mod server_impl;

use component::{bullet_loader::BulletLoader, hn_icons::*, spinner::SpinnerFit};
use hon_worker_common::limits::{CoinState, BET_COIN_ENABLED_STATES, DEFAULT_BET_COIN_STATE};
use hon_worker_common::{sign_vote_request, GameInfo, GameResult, WORKER_URL};
use ic_agent::Identity;
use leptos::html::Audio;
use leptos::{either::Either, prelude::*};
use leptos_icons::*;
use server_impl::vote_with_cents_on_post;
use state::canisters::auth_state;
use utils::try_or_redirect_opt;
use utils::{mixpanel::mixpanel_events::*, send_wrap};
use yral_canisters_common::utils::{
    posts::PostDetails, token::balance::TokenBalance, vote::VoteKind,
};

trait CoinStateExt {
    fn wrapping_next(self) -> Self;
    fn wrapping_prev(self) -> Self;
}

impl CoinStateExt for CoinState {
    fn wrapping_next(self) -> Self {
        let current_index = BET_COIN_ENABLED_STATES.iter().position(|&x| x == self);
        match current_index {
            Some(idx) => {
                let next_idx = (idx + 1) % BET_COIN_ENABLED_STATES.len();
                BET_COIN_ENABLED_STATES[next_idx]
            }
            None => DEFAULT_BET_COIN_STATE,
        }
    }

    fn wrapping_prev(self) -> Self {
        let current_index = BET_COIN_ENABLED_STATES.iter().position(|&x| x == self);
        match current_index {
            Some(idx) => {
                let prev_idx = if idx == 0 {
                    BET_COIN_ENABLED_STATES.len() - 1
                } else {
                    idx - 1
                };
                BET_COIN_ENABLED_STATES[prev_idx]
            }
            None => DEFAULT_BET_COIN_STATE,
        }
    }
}

trait ToCents {
    fn to_cents(&self) -> u64;
}

impl ToCents for CoinState {
    fn to_cents(&self) -> u64 {
        match self {
            CoinState::C10 => 10,
            CoinState::C20 => 20,
            CoinState::C50 => 50,
            CoinState::C100 => 100,
            CoinState::C200 => 200,
        }
    }
}

#[component]
fn CoinStateView(
    #[prop(into)] coin: Signal<CoinState>,
    #[prop(into)] class: String,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let icon = Signal::derive(move || match coin() {
        CoinState::C10 => C10Icon,
        CoinState::C20 => C20Icon,
        CoinState::C50 => C50Icon,
        CoinState::C100 => C100Icon,
        CoinState::C200 => C200Icon,
    });

    view! {
        <div class:grayscale=disabled>
            <Icon attr:class=class icon />
        </div>
    }
}

#[component]
fn HNButton(
    bet_direction: RwSignal<Option<VoteKind>>,
    kind: VoteKind,
    #[prop(into)] disabled: Signal<bool>,
    place_bet_action: Action<VoteKind, Option<()>>,
) -> impl IntoView {
    let grayscale = Memo::new(move |_| bet_direction() != Some(kind) && disabled());
    let show_spinner = move || disabled() && bet_direction() == Some(kind);
    let icon = if kind == VoteKind::Hot {
        HotIcon
    } else {
        NotIcon
    };

    view! {
        <button
            class="w-14 h-14 md:w-16 md:h-16 md:w-18 lg:h-18"
            class=("grayscale", grayscale)
            disabled=disabled
            on:click=move |_| {bet_direction.set(Some(kind)); place_bet_action.dispatch(kind);}
        >
            <Show when=move || !show_spinner() fallback=SpinnerFit>
                <Icon attr:class="w-full h-full drop-shadow-lg" icon=icon />
            </Show>
        </button>
    }
}

#[component]
fn HNButtonOverlay(
    post: PostDetails,
    prev_post: Option<PostDetails>,
    coin: RwSignal<CoinState>,
    bet_direction: RwSignal<Option<VoteKind>>,
    refetch_bet: Trigger,
    audio_ref: NodeRef<Audio>,
) -> impl IntoView {
    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    fn play_win_sound_and_vibrate(audio_ref: NodeRef<Audio>, won: bool) {
        #[cfg(not(feature = "hydrate"))]
        {
            _ = audio_ref;
        }
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsValue;
            use web_sys::js_sys::Reflect;

            let window = window();
            let nav = window.navigator();
            if Reflect::has(&nav, &JsValue::from_str("vibrate")).unwrap_or_default() {
                nav.vibrate_with_duration(200);
            } else {
                log::debug!("browser does not support vibrate");
            }
            let Some(audio) = audio_ref.get_untracked() else {
                return;
            };
            if won {
                audio.set_current_time(0.);
                audio.set_volume(0.5);
                _ = audio.play();
            }
        }
    }

    let place_bet_action: Action<VoteKind, Option<()>> =
        Action::new(move |bet_direction: &VoteKind| {
            let post_canister = post.canister_id;
            let post_id = post.post_id;
            let bet_amount: u64 = coin.get_untracked().to_cents();
            let bet_direction = *bet_direction;
            let req = hon_worker_common::VoteRequest {
                post_canister,
                post_id,
                vote_amount: bet_amount as u128,
                direction: bet_direction.into(),
            };
            let prev_post = prev_post.as_ref().map(|p| (p.canister_id, p.post_id));

            let post_mix = post.clone();
            send_wrap(async move {
                let cans = auth.auth_cans(expect_context()).await.ok()?;
                let identity = cans.identity();
                let sender = identity.sender().unwrap();
                let sig = sign_vote_request(identity, req.clone()).ok()?;

                let res = vote_with_cents_on_post(sender, req, sig, prev_post).await;
                refetch_bet.notify();
                match res {
                    Ok(res) => {
                        let is_logged_in = is_connected.get_untracked();
                        let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
                        let game_conclusion = match res.game_result {
                            GameResult::Win { .. } => GameConclusion::Win,
                            GameResult::Loss { .. } => GameConclusion::Loss,
                        };
                        let win_loss_amount = match res.game_result.clone() {
                            GameResult::Win { win_amt } => {
                                TokenBalance::new((win_amt + bet_amount).into(), 0).humanize()
                            }
                            GameResult::Loss { lose_amt } => {
                                TokenBalance::new((lose_amt + 0u64).into(), 0).humanize()
                            }
                        };
                        MixPanelEvent::track_game_played(MixpanelGamePlayedProps {
                            is_nsfw: post_mix.is_nsfw,
                            user_id: global.user_id,
                            visitor_id: global.visitor_id,
                            is_logged_in: global.is_logged_in,
                            canister_id: global.canister_id,
                            is_nsfw_enabled: global.is_nsfw_enabled,
                            game_type: MixpanelPostGameType::HotOrNot,
                            option_chosen: bet_direction.into(),
                            publisher_user_id: post_mix.poster_principal.to_text(),
                            video_id: post_mix.uid.clone(),
                            view_count: post_mix.views,
                            like_count: post_mix.likes,
                            stake_amount: bet_amount,
                            is_game_enabled: true,
                            stake_type: StakeType::Sats,
                            conclusion: game_conclusion,
                            won_loss_amount: win_loss_amount,
                            creator_commision_percentage: crate::consts::CREATOR_COMMISION_PERCENT,
                        });
                        play_win_sound_and_vibrate(
                            audio_ref,
                            matches!(res.game_result, GameResult::Win { .. }),
                        );
                        Some(())
                    }
                    Err(e) => {
                        log::error!("{e}");
                        None
                    }
                }
            })
        });

    let running = place_bet_action.pending();

    view! {
        <div class="flex justify-center w-full touch-manipulation">
            <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_next())>
                <Icon
                    attr:class="justify-self-end text-2xl text-white"
                    icon=icondata::AiUpOutlined
                />
            </button>
        </div>
        <div class="flex flex-row gap-6 justify-center items-center w-full touch-manipulation">
            <HNButton disabled=running bet_direction kind=VoteKind::Hot place_bet_action />
            <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_next())>
                <CoinStateView
                    disabled=running
                    class="w-12 h-12 md:w-14 md:h-14 lg:w-16 lg:h-16 drop-shadow-lg"
                    coin
                />
            </button>
            <HNButton disabled=running bet_direction kind=VoteKind::Not place_bet_action />
        </div>
        // Bottom row: Hot <down arrow> Not
        // most of the CSS is for alignment with above icons
        <div class="flex gap-6 justify-center items-center pt-2 w-full text-base font-medium text-center md:text-lg lg:text-xl touch-manipulation">
            <p class="w-14 md:w-16 lg:w-18">Hot</p>
            <div class="flex justify-center w-12 md:w-14 lg:w-16">
                <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_prev())>
                    <Icon attr:class="text-2xl text-white" icon=icondata::AiDownOutlined />
                </button>
            </div>
            <p class="w-14 md:w-16 lg:w-18">Not</p>
        </div>
        <ShadowBg />
    }
}

#[component]
fn WinBadge() -> impl IntoView {
    view! {
        <button class="py-2 px-4 w-full text-sm font-bold text-white rounded-sm bg-primary-600">

            <div class="flex justify-center items-center">
                <span class="">
                    <Icon attr:class="fill-white" style="" icon=icondata::RiTrophyFinanceFill />
                </span>
                <span class="ml-2">"You Won"</span>
            </div>
        </button>
    }
}

#[component]
fn LostBadge() -> impl IntoView {
    view! {
        <button class="py-2 px-4 w-full text-sm font-bold bg-white rounded-sm text-primary-600">

            <div class="flex justify-center items-center">
                <span class="">
                    <Icon attr:class="fill-white" style="" icon=icondata::LuThumbsDown />
                </span>
                <span class="ml-2">"You Lost"</span>
            </div>
        </button>
    }
}

#[component]
fn HNWonLost(game_result: GameResult, vote_amount: u64) -> impl IntoView {
    let won = matches!(game_result, GameResult::Win { .. });
    let creator_reward = (vote_amount * crate::consts::CREATOR_COMMISION_PERCENT) / 100;
    let message = match game_result {
        GameResult::Win { win_amt } => format!(
            "You received {} SATS, {} SATS went to the creator.",
            TokenBalance::new((win_amt + vote_amount).into(), 0).humanize(),
            creator_reward
        ),
        GameResult::Loss { lose_amt } => format!(
            "You lost {} SATS.",
            TokenBalance::new(lose_amt.into(), 0).humanize()
        ),
    };
    let bet_amount = vote_amount;
    let coin = match bet_amount {
        10 => CoinState::C10,
        20 => CoinState::C20,
        50 => CoinState::C50,
        100 => CoinState::C100,
        200 => CoinState::C200,
        amt => {
            log::warn!("Invalid bet amount: {amt}, using fallback");
            CoinState::C50
        }
    };

    view! {
        <div class="flex gap-6 justify-center items-center p-4 w-full bg-transparent rounded-xl shadow-sm">
            <div class="relative shrink-0 drop-shadow-lg">
                <CoinStateView class="w-14 h-14 md:w-16 md:h-16" coin />
            </div>

            // <!-- Text and Badge Column -->
            <div class="flex flex-col gap-2 w-full md:w-1/2 lg:w-1/3">
                // <!-- Result Text -->
                <div class="p-1 text-sm leading-snug text-white rounded-full">
                    <p>{message}</p>

                </div>
                {if won {
                    Either::Left(view! { <WinBadge /> })
                } else {
                    Either::Right(view! { <LostBadge /> })
                }}

            </div>

        </div>
    }
}

#[component]
pub fn HNUserParticipation(
    post: PostDetails,
    participation: GameInfo,
    refetch_bet: Trigger,
) -> impl IntoView {
    let (_, _) = (post, refetch_bet); // not sure if i will need these later
    let (vote_amount, game_result) = match participation {
        GameInfo::CreatorReward(..) => unreachable!(
            "When a game result is accessed, backend should never return creator reward"
        ),
        GameInfo::Vote {
            vote_amount,
            game_result,
        } => (vote_amount, game_result),
    };
    let vote_amount: u64 = vote_amount
        .try_into()
        .expect("We only allow voting with 200 max, so this is alright");

    view! {
        <HNWonLost game_result vote_amount />
        <ShadowBg />
    }
}

#[component]
fn LoaderWithShadowBg() -> impl IntoView {
    view! {
        <BulletLoader />
        <ShadowBg />
    }
}

#[component]
fn ShadowBg() -> impl IntoView {
    view! {
        <div
            class="absolute bottom-0 left-0 h-2/5 w-dvw -z-1"
            style="background: linear-gradient(to bottom, #00000000 0%, #00000099 45%, #000000a8 100%, #000000cc 100%, #000000a8 100%);"
        ></div>
    }
}

#[component]
pub fn HNGameOverlay(
    post: PostDetails,
    prev_post: Option<PostDetails>,
    win_audio_ref: NodeRef<Audio>,
) -> impl IntoView {
    let bet_direction = RwSignal::new(None::<VoteKind>);
    let coin = RwSignal::new(DEFAULT_BET_COIN_STATE);

    let refetch_bet = Trigger::new();
    let post = StoredValue::new(post);

    let auth = auth_state();
    let create_game_info = auth.derive_resource(
        move || refetch_bet.track(),
        move |cans, _| {
            send_wrap(async move {
                let post = post.get_value();
                let game_info = cans
                    .fetch_game_with_sats_info(
                        reqwest::Url::parse(WORKER_URL).unwrap(),
                        (post.canister_id, post.post_id).into(),
                    )
                    .await?;
                Ok::<_, ServerFnError>(game_info)
            })
        },
    );

    view! {
        <Suspense fallback=LoaderWithShadowBg>

            {move || {
                create_game_info
                    .get()
                    .and_then(|res| {
                        let participation = try_or_redirect_opt!(res.as_ref());
                        let post = post.get_value();
                        Some(
                            if let Some(participation) = participation {
                                view! {
                                    <HNUserParticipation
                                        post
                                        refetch_bet
                                        participation=participation.clone()
                                    />
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <HNButtonOverlay
                                        post
                                        prev_post=prev_post.clone()
                                        bet_direction
                                        coin
                                        refetch_bet
                                        audio_ref=win_audio_ref
                                    />
                                }
                                    .into_any()
                            },
                        )
                    })
                    .unwrap_or_else(|| view! { <LoaderWithShadowBg /> }.into_any())
            }}

        </Suspense>
    }
}
