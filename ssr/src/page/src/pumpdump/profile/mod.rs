use std::convert::Infallible;

use crate::pumpdump::{convert_e8s_to_cents, GameResult};
use candid::Principal;
use component::{
    back_btn::BackButton, infinite_scroller::InfiniteScroller, skeleton::Skeleton, title::TitleText,
};
use consts::PUMP_AND_DUMP_WORKER_URL;
use futures::{stream::FuturesOrdered, StreamExt};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use state::canisters::authenticated_canisters;
use utils::send_wrap;
use yral_canisters_client::{
    individual_user_template::IndividualUserTemplate, sns_ledger::MetadataValue,
    sns_root::ListSnsCanistersArg,
};
use yral_canisters_common::{
    cursored_data::{CursoredDataProvider, KeyedData, PageEntry},
    utils::profile::{propic_from_principal, ProfileDetails},
    Canisters,
};
use yral_pump_n_dump_common::rest::{CompletedGameInfo, UncommittedGameInfo, UncommittedGamesRes};

use super::GameState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ProfileData {
    pub(super) user: ProfileDetails,
}

impl ProfileData {
    pub(super) async fn load(
        user: ProfileDetails,
        _ind_user: IndividualUserTemplate<'_>,
    ) -> Result<Self, String> {
        Ok(Self { user })
    }
}

#[derive(Debug, Clone)]
struct GameplayHistoryItem {
    logo: String,
    root: Principal,
    owner_principal: Principal,
    owner_pfp: String,
    state: GameState,
}

impl KeyedData for GameplayHistoryItem {
    type Key = Principal;

    fn key(&self) -> Self::Key {
        self.root
    }
}
type GameplayHistory = Vec<GameplayHistoryItem>;

fn compute_result(info: impl Into<CompletedGameInfo>) -> GameResult {
    let info = info.into();

    let reward = convert_e8s_to_cents(info.reward);
    let spent = info.pumps as u128 + info.dumps as u128;

    if spent > reward {
        GameResult::Loss {
            amount: spent - reward,
        }
    } else {
        GameResult::Win {
            amount: reward - spent,
        }
    }
}

// TODO: switch to using in-house `InfiniteScroller` for 0.7 migration
async fn load_history(
    cans: Canisters<true>,
    start_idx: u64,
    end_idx: u64,
) -> Result<(GameplayHistory, bool), String> {
    // to test without playing games
    // #[cfg(any(feature = "local-lib", feature = "local-bin"))]
    // {
    //     let history = vec![GameplayHistoryItem {
    //         logo: "https://picsum.photos/200".into(),
    //         owner_pfp: "https://picsum.photos/200?rand=1".into(),
    //         owner_principal: Principal::anonymous(),
    //         root: Principal::anonymous(),
    //         state: GameState::ResultDeclared(GameResult::Win { amount: 100 }),
    //     }];

    //     return Ok((history, true));
    // }

    let (items, request_more) = if start_idx == 0 {
        // First page includes uncommitted games
        (load_uncommitted_games(&cans).await?, true)
    } else {
        // Subsequent pages load from chain
        let items = load_from_chain(&cans, start_idx - 1, end_idx - start_idx).await?;
        let request_more = !items.is_empty();
        (items, request_more)
    };

    let token_infos = items
        .into_iter()
        .map(async |item| {
            let token_root = item.token_root();

            let owner_and_pfp_fut = async {
                let token_owner = cans.get_token_owner(token_root).await.ok().flatten();
                let (token_owner_principal, token_owner_canister) = token_owner
                    .map(|o| (o.principal_id, Some(o.canister_id)))
                    .unwrap_or_else(|| (Principal::anonymous(), None));

                let pfp = if let Some(canister) = token_owner_canister {
                    cans.individual_user(canister)
                        .await
                        .get_profile_details()
                        .await
                        .map(|details| {
                            let details = ProfileDetails::from(details);
                            details.profile_pic_or_random()
                        })
                        .ok()
                } else {
                    None
                };
                let pfp = pfp.unwrap_or_else(|| propic_from_principal(token_owner_principal));
                (token_owner_principal, pfp)
            };

            let token_logo_fut = async {
                let ledger = cans
                    .sns_root(token_root)
                    .await
                    .list_sns_canisters(ListSnsCanistersArg {})
                    .await
                    .ok()
                    .and_then(|l| l.ledger);
                let Some(ledger) = ledger else {
                    return propic_from_principal(token_root);
                };

                cans.sns_ledger(ledger)
                    .await
                    .icrc_1_metadata()
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .find_map(|(k, v)| {
                        if k != "icrc1:logo" {
                            return None;
                        }
                        let MetadataValue::Text(logo) = v else {
                            return None;
                        };
                        Some(logo)
                    })
                    .unwrap_or_else(|| propic_from_principal(token_root))
            };

            let ((owner_principal, owner_pfp), logo) =
                futures::join!(owner_and_pfp_fut, token_logo_fut);

            GameplayHistoryItem {
                logo,
                root: token_root,
                owner_principal,
                owner_pfp,
                state: match item {
                    UncommittedGameInfo::Completed(item) => {
                        GameState::ResultDeclared(compute_result(item))
                    }
                    UncommittedGameInfo::Pending { .. } => GameState::Playing,
                },
            }
        })
        .collect::<FuturesOrdered<_>>();
    let processed_items = token_infos.collect().await;

    Ok((processed_items, request_more))
}

async fn load_uncommitted_games(cans: &Canisters<true>) -> Result<UncommittedGamesRes, String> {
    let uncommitted_games = PUMP_AND_DUMP_WORKER_URL
        .join(&format!("/uncommitted_games/{}", cans.user_canister()))
        .expect("url to be valid");

    let uncommitted_games: UncommittedGamesRes = reqwest::get(uncommitted_games)
        .await
        .map_err(|err| format!("Coulnd't load bets: {err}"))?
        .json()
        .await
        .map_err(|err| format!("Couldn't parse bets out of repsonse: {err}"))?;

    Ok(uncommitted_games)
}

async fn load_from_chain(
    cans: &Canisters<true>,
    start_index: u64,
    limit: u64,
) -> Result<UncommittedGamesRes, String> {
    let items = cans
        .individual_user(cans.user_canister())
        .await
        .played_game_info_with_pagination_cursor(start_index, limit)
        .await
        .map_err(|err| format!("Couldn't load gameplay history: {err}"))?;
    let items = match items {
        yral_canisters_client::individual_user_template::Result15::Ok(res) => res,
        yral_canisters_client::individual_user_template::Result15::Err(err) => {
            return match err.as_str() {
                "ReachedEndOfItemsList" => {
                    return Ok(Default::default());
                }
                _ => Err(format!("Couldn't load played games: {err}")),
            };
        }
    };
    let items = items
        .into_iter()
        .map(|item| UncommittedGameInfo::Completed(item.into()))
        .collect();
    Ok(items)
}

#[component]
fn Stat(_stat: u64, #[prop(into)] _info: String) -> impl IntoView {
    view! {
        <div class="flex flex-1 flex-col items-center text-white space-y-0.5">
            <span class="font-bold text-xl text-neutral-50">{_stat}</span>
            <span class="text-md text-neutral-400">{_info}</span>
        </div>
    }
}

#[component]
fn ProfileDataSection(#[prop(into)] profile_data: ProfileData) -> impl IntoView {
    let username_or_principal = profile_data.user.username_or_principal();
    let profile_pic = profile_data.user.profile_pic_or_random();
    let display_name = profile_data.user.display_name_or_fallback();

    view! {
        <div class="grid grid-cols-1 gap-5 justify-normal justify-items-center w-full">
            <div class="flex flex-row w-11/12 justify-center">
                <div class="flex flex-col justify-center items-center gap-4">
                    <img
                        class="h-24 w-24 rounded-full"
                        alt=username_or_principal.clone()
                        src=profile_pic
                    />
                    <div class="flex flex-col text-center items-center gap-4">
                        <span class="text-md text-white font-bold w-full">{display_name}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn GameplayHistorySkeleton() -> impl IntoView {
    view! {
        <Skeleton class="text-neutral-800 [--shimmer:#363636] max-w-32 max-h-40 w-32 h-40 rounded-md" />
    }
}

#[component]
fn GameplayHistoryCard(#[prop(into)] details: GameplayHistoryItem) -> impl IntoView {
    let state = details.state;
    let href = {
        let root = details.root;
        let (state_label, amount) = match state {
            GameState::ResultDeclared(result) => match result {
                GameResult::Win { amount } => ("win", Some(amount)),
                GameResult::Loss { amount } => ("loss", Some(amount)),
            },
            GameState::Playing => ("pending", None),
        };

        match amount {
            Some(amount) => format!("/?root={root}&state={state_label}&amount={amount}"),
            None => format!("/?root={root}&state={state_label}"),
        }
    };
    view! {
        <a href=href>
            <div class="rounded-md overflow-hidden relative max-w-32 max-h-40 w-32 h-40">
                <div class="absolute z-1 inset-x-0 h-1/3 bg-linear-to-b from-black/50 to-transparent"></div>
                <div class="absolute z-2 flex top-2 items-center gap-1 px-2">
                    <img
                        src=details.owner_pfp
                        alt="Profile name"
                        class="w-4 h-4 shrink-0 object-cover rounded-full"
                    />
                    <span class="text-xs font-medium line-clamp-1">
                        {details.owner_principal.to_string()}
                    </span>
                </div>
                <img
                    src=details.logo
                    class="w-full bg-white/5 h-28 object-cover"
                    alt="Coin title"
                />
                <Show
                    when=move || !state.is_running()
                    fallback=move || {
                        view! {
                            <div
                                style="background: linear-gradient(248.46deg, rgba(61, 142, 255, 0.4) 16.67%, rgba(57, 0, 89, 0.4) 52.62%, rgba(226, 1, 123, 0.4) 87.36%);"
                                class="text-xs font-semibold justify-center py-4 flex items-center text-[#FAFAFA]"
                            >
                                Pending
                            </div>
                        }
                    }
                >
                    <div
                        class="text-xs font-semibold py-1 text-center"
                        class=(["bg-primary-800", "text-white"], state.has_won())
                        class=(["text-neutral-400", "bg-[#212121]"], state.has_lost())
                    >
                        {if state.has_lost() { "You Lost" } else { "You Won" }}
                    </div>
                    <div
                        class="text-xs font-semibold py-1 text-center"
                        class=(["bg-primary-600", "text-white"], state.has_won())
                        class=(["bg-neutral-600", "text-neutral-400"], state.has_lost())
                    >
                        {state.winnings().or(state.lossings())}
                        Cents
                    </div>
                </Show>
            </div>
        </a>
    }
}
#[component]
pub fn PndProfilePage() -> impl IntoView {
    let auth_cans = authenticated_canisters();
    let fetch_profile_data: Resource<std::result::Result<ProfileData, ServerFnError>> =
        Resource::new(
            move || (),
            move |_| async move {
                let cans_wire = authenticated_canisters().await?;
                let user = cans_wire.profile_details.clone();
                let canisters = Canisters::from_wire(cans_wire.clone(), expect_context())?;
                let ind_user =
                    send_wrap(canisters.individual_user(canisters.user_canister())).await;
                send_wrap(ProfileData::load(user, ind_user))
                    .await
                    .map_err(|e| ServerFnError::new(e.to_string()))
            },
        );
    view! {
        <div class="min-h-screen w-full flex flex-col text-white pt-2 pb-12 bg-black items-center">
            <div id="back-nav" class="flex flex-col items-center w-full gap-20 pb-16">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between">
                        <BackButton fallback="/" />
                        <span class="font-bold text-2xl">Profile</span>
                        <div></div>
                    </div>
                </TitleText>
            </div>
            <Suspense>
            {
                move || {
                    let profile_data_res = fetch_profile_data.get();
                    match profile_data_res {
                        Some(Ok(profile_data)) => Some(view! {
                            <ProfileDataSection profile_data=profile_data />
                        }),
                        _ => None
                    }
                }
            }
            </Suspense>

            <div class="w-11/12 flex justify-center">
                <div class="flex flex-wrap gap-4 justify-center pt-8 pb-16">
                    <Suspense>
                        {
                            Suspend::new(async move {
                                // cant use auth cans in the cursoreddata impl expect context fails
                                let cans_wire = auth_cans.await;
                                let cans_wire = cans_wire.unwrap();
                                let canisters = Canisters::from_wire(cans_wire, expect_context()).unwrap();
                                let provider = GameplayHistoryProvider(canisters);

                                view!{
                                    <InfiniteScroller
                                    provider
                                    fetch_count=6
                                    children=move |item, _ref| {
                                        view! {
                                            <div node_ref=_ref.unwrap_or_default()>
                                                <GameplayHistoryCard details=item />
                                            </div>
                                        }.into_any()
                                    }
                                    custom_loader=move || {
                                        view! {
                                            <For each=move || 0..6 key=|&idx| idx let:_>
                                                <GameplayHistorySkeleton />
                                            </For>
                                        }.into_any()
                                    }
                                />
                                }
                            })
                        }
                </Suspense>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone)]
struct GameplayHistoryProvider(Canisters<true>);

impl CursoredDataProvider for GameplayHistoryProvider {
    type Data = GameplayHistoryItem;
    type Error = Infallible;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> std::result::Result<yral_canisters_common::cursored_data::PageEntry<Self::Data>, Self::Error>
    {
        let (items, request_more) = load_history(self.0.clone(), start as u64, end as u64)
            .await
            .unwrap();
        Ok(PageEntry {
            data: items,
            end: request_more,
        })
    }
}
