mod server_impl;

use candid::Principal;
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use component::login_modal::LoginModal;
use component::login_nudge_popup::LoginNudgePopup;
use component::{bullet_loader::BulletLoader, hn_icons::*, show_any::ShowAny, spinner::SpinnerFit};
use consts::auth::REFRESH_MAX_AGE;
use consts::{
    UserOnboardingStore, AUTH_JOURNEY_PAGE, USER_ONBOARDING_STORE_KEY, WALLET_BALANCE_STORE_KEY,
};
use global_constants::{
    CoinState, CREATOR_COMMISSION_PERCENT, DEFAULT_BET_COIN_FOR_LOGGED_IN,
    DEFAULT_BET_COIN_FOR_LOGGED_OUT,
};
use hon_worker_common::{
    sign_vote_request_v3, GameInfo, GameInfoReqV3, GameResult, GameResultV2, VoteRequestV3,
    VoteResV2, WORKER_URL,
};
use ic_agent::Identity;
use leptos::html::Audio;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_use::storage::use_local_storage;
use leptos_use::{use_cookie_with_options, use_timeout_fn, UseCookieOptions, UseTimeoutFnReturn};
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use server_impl::vote_with_cents_on_post;
use state::canisters::auth_state;
use state::hn_bet_state::{HnBetState, VideoComparisonResult};
use utils::try_or_redirect_opt;
use utils::{mixpanel::mixpanel_events::*, send_wrap};
use yral_canisters_common::utils::{
    posts::PostDetails,
    token::balance::TokenBalance,
    token::load_sats_balance,
    vote::{fetch_game_with_sats_info_v3, VoteKind},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteAPIRes {
    pub game_result: VoteResV2,
    pub video_comparison_result: VideoComparisonResult,
}

#[component]
fn CoinStateView(
    #[prop(into)] coin: Signal<CoinState>,
    #[prop(into)] class: String,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] is_connected: Signal<bool>,
) -> impl IntoView {
    let icon = Signal::derive(move || match coin() {
        CoinState::C10 => C10Icon,
        CoinState::C1 => C1Icon,
        CoinState::C5 => C5Icon,
        CoinState::C20 => C20Icon,
        CoinState::C50 => C50Icon,
        CoinState::C100 => C100Icon,
        CoinState::C200 => C200Icon,
    });

    let disabled = Signal::derive(move || {
        disabled.get() || (!is_connected.get() && coin.get() != CoinState::C1)
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
        "/img/hotornot/hot-icon.svg"
    } else {
        "/img/hotornot/not-icon.svg"
    };

    view! {
        <button
            class="size-14 md:size-16 drop-shadow-[0_4px_6px_rgba(0,0,0,0.28)]"
            class=("grayscale", grayscale)
            disabled=disabled
            on:click=move |_| {bet_direction.set(Some(kind)); place_bet_action.dispatch(kind);}
        >
            <Show when=move || !show_spinner() fallback=SpinnerFit>
                <img src=icon alt="Icons..." class="w-full h-full"
                    loading="eager"
                />
            </Show>
        </button>
    }
}

async fn fetch_and_update_balance(
    auth: &state::canisters::AuthState,
    set_wallet_balance_store: WriteSignal<u64>,
) -> Option<u64> {
    let cans = auth.auth_cans().await.ok()?;
    let user_principal = cans.user_principal();
    let balance_info = load_sats_balance(user_principal).await.ok()?;
    let balance = balance_info.balance.to_u64().unwrap_or(25);
    set_wallet_balance_store.set(balance);
    HnBetState::set_balance(balance);
    Some(balance)
}

#[component]
fn HNButtonOverlay(
    post: PostDetails,
    prev_post: Option<(Principal, u64)>,
    coin: RwSignal<CoinState>,
    bet_direction: RwSignal<Option<VoteKind>>,
    refetch_bet: Trigger,
    audio_ref: NodeRef<Audio>,
    show_low_balance_popup: RwSignal<bool>,
) -> impl IntoView {
    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();
    let ev_ctx = auth.event_ctx();

    let (wallet_balance_store, set_wallet_balance_store, _) =
        use_local_storage::<u64, FromToStringCodec>(WALLET_BALANCE_STORE_KEY);

    // Track previous user principal to detect account switches
    let prev_principal = StoredValue::new(None::<Principal>);

    // Clear wallet balance when user authentication changes
    Effect::new(move |_| {
        // Track the user principal to detect account switches
        if let Some(Ok(current_principal)) = auth.user_principal.get() {
            let previous = prev_principal.get_value();
            if previous.is_some() && previous != Some(current_principal) {
                // User has switched accounts, clear cached balance
                set_wallet_balance_store.set(0);
                HnBetState::set_balance(0);
            }
            prev_principal.set_value(Some(current_principal));
        }
    });

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

    let show_login_nudge = RwSignal::new(false);
    let show_login_popup = RwSignal::new(false);

    let default_bet_coin = if is_connected.get_untracked() {
        DEFAULT_BET_COIN_FOR_LOGGED_IN
    } else {
        DEFAULT_BET_COIN_FOR_LOGGED_OUT
    };

    let check_show_login_nudge = move || {
        if !is_connected.get_untracked() && coin.get_untracked() != default_bet_coin {
            show_login_nudge.set(true);
            Err(())
        } else {
            Ok(())
        }
    };

    let place_bet_action: Action<VoteKind, Option<()>> =
        Action::new(move |bet_direction: &VoteKind| {
            let post_canister = post.canister_id;
            let post_id = post.post_id;
            let bet_amount: u64 = coin.get_untracked().to_cents();
            let bet_direction = *bet_direction;

            // Create the original VoteRequest for the server function
            let req = hon_worker_common::VoteRequest {
                post_canister,
                post_id,
                vote_amount: bet_amount as u128,
                direction: bet_direction.into(),
            };
            // Create VoteRequestV3 for signing
            let req_v3 = VoteRequestV3 {
                publisher_principal: post.poster_principal,
                post_id,
                vote_amount: bet_amount as u128,
                direction: bet_direction.into(),
            };

            let post_mix = post.clone();
            send_wrap(async move {
                let res = check_show_login_nudge();
                if res.is_err() {
                    return None;
                }
                // The auth.auth_cans() call will fetch the current canisters from the resource
                // which tracks identity changes and updates automatically
                let cans = auth.auth_cans().await.ok()?;

                let is_logged_in = is_connected.get_untracked();
                let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
                MixPanelEvent::track_game_clicked(
                    global,
                    post_mix.poster_principal.to_text(),
                    post_mix.likes,
                    post_mix.views,
                    true,
                    post_mix.uid.clone(),
                    MixpanelPostGameType::HotOrNot,
                    bet_direction.into(),
                    bet_amount,
                    StakeType::Sats,
                    post.is_nsfw,
                );

                // Check balance and refetch if insufficient
                let current_balance = wallet_balance_store.get_untracked();
                if bet_amount > current_balance {
                    fetch_and_update_balance(&auth, set_wallet_balance_store).await;

                    let current_balance = wallet_balance_store.get_untracked();

                    if bet_amount > current_balance {
                        show_low_balance_popup.set(true);
                        return None;
                    }
                }

                let identity = cans.identity();
                let sender = identity.sender().unwrap();
                let sig = sign_vote_request_v3(identity, req_v3).ok()?;

                let res = vote_with_cents_on_post(sender, req, sig, prev_post).await;
                refetch_bet.notify();
                match res {
                    Ok(res) => {
                        let is_logged_in = is_connected.get_untracked();
                        let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
                        let game_conclusion = match res.game_result.game_result {
                            GameResultV2::Win { .. } => GameConclusion::Win,
                            GameResultV2::Loss { .. } => GameConclusion::Loss,
                        };
                        let win_loss_amount = match res.game_result.game_result.clone() {
                            GameResultV2::Win {
                                win_amt,
                                updated_balance: _,
                            } => TokenBalance::new((win_amt + bet_amount).into(), 0).humanize(),
                            GameResultV2::Loss {
                                lose_amt,
                                updated_balance: _,
                            } => TokenBalance::new((lose_amt + 0u64).into(), 0).humanize(),
                        };

                        HnBetState::set(post_mix.uid.clone(), res.video_comparison_result);

                        let (wallet_balance_store, set_wallet_balance_store, _) =
                            use_local_storage::<u64, FromToStringCodec>(WALLET_BALANCE_STORE_KEY);
                        let current_balance = wallet_balance_store.get_untracked() - bet_amount;

                        let balance = match res.game_result.game_result.clone() {
                            GameResultV2::Win {
                                win_amt: _,
                                updated_balance,
                            } => updated_balance.to_u64().unwrap_or(current_balance),
                            GameResultV2::Loss {
                                lose_amt: _,
                                updated_balance,
                            } => updated_balance.to_u64().unwrap_or(current_balance),
                        };
                        HnBetState::set_balance(balance);
                        set_wallet_balance_store.set(balance);

                        MixPanelEvent::track_game_played(
                            global,
                            post_mix.uid.clone(),
                            post_mix.poster_principal.to_text(),
                            MixpanelPostGameType::HotOrNot,
                            bet_amount,
                            StakeType::Sats,
                            bet_direction.into(),
                            post_mix.likes,
                            post_mix.views,
                            true,
                            game_conclusion,
                            win_loss_amount,
                            CREATOR_COMMISSION_PERCENT,
                            post.is_nsfw,
                        );
                        play_win_sound_and_vibrate(
                            audio_ref,
                            matches!(res.game_result.game_result, GameResultV2::Win { .. }),
                        );

                        // Trigger rank update after successful vote
                        if let Some(rank_update_count) =
                            use_context::<RwSignal<component::leaderboard::RankUpdateCounter>>()
                        {
                            rank_update_count.update(|c| c.0 += 1);
                        }

                        Some(())
                    }
                    Err(e) => {
                        let error_msg = format!("{e}");

                        // Only show low balance popup for insufficient funds errors
                        if error_msg.contains("InsufficientFunds") {
                            show_low_balance_popup.set(true);
                        }

                        log::error!("{error_msg}");
                        None
                    }
                }
            })
        });

    let running = place_bet_action.pending();

    let was_connected = RwSignal::new(is_connected.get_untracked());

    let (auth_journey_page, _) = use_cookie_with_options::<BottomNavigationCategory, JsonSerdeCodec>(
        AUTH_JOURNEY_PAGE,
        UseCookieOptions::default()
            .path("/")
            .max_age(REFRESH_MAX_AGE.as_millis() as i64),
    );

    let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
        move |_| {
            let _ = window().location().set_href("/");
        },
        50.0,
    );

    Effect::new(move |_| {
        let auth_journey_page_cookie = auth_journey_page.get();
        if !was_connected.get() && is_connected.get() && auth_journey_page_cookie.is_none() {
            start(());
        }
    });

    view! {
        <div class="flex justify-center w-full touch-manipulation">
            <button disabled=running  on:click=move |_| coin.update(|c| *c = c.wrapping_next())
              class="disabled:grayscale disabled:opacity-60 disabled:cursor-not-allowed"
            >
                <img
                    class="w-6 h-6 md:w-7 md:h-7"
                    src="/img/hotornot/hn_arrow_up.svg"
                />
            </button>
        </div>
        <LoginNudgePopup show=show_login_nudge show_login_popup  ev_ctx coin/>
        <LoginModal show=show_login_popup redirect_to=None reload_window=true/>
        <div class="flex flex-row gap-6 justify-center items-center w-full touch-manipulation">
            <HNButton disabled=running bet_direction kind=VoteKind::Hot place_bet_action />
            <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_next())>
                <CoinStateView
                    disabled=running
                    class="w-12 h-12 md:w-14 md:h-14 lg:w-16 lg:h-16 drop-shadow-lg"
                    coin
                    is_connected
                />
            </button>
            <HNButton disabled=running bet_direction kind=VoteKind::Not place_bet_action />
        </div>
        // Bottom row: Hot <down arrow> Not
        // most of the CSS is for alignment with above icons
        <div class="flex gap-6 justify-center items-center pt-2 w-full text-base font-medium text-center md:text-lg lg:text-xl touch-manipulation">
            <p class="w-14 md:w-16 lg:w-18">Hot</p>
            <div class="flex justify-center w-12 md:w-14 lg:w-16">
                <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_prev())
                    class="disabled:grayscale disabled:opacity-60 disabled:cursor-not-allowed"
                >
                    <img
                        src="/img/hotornot/hn_arrow_down.svg"
                    />
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
fn HNWonLost(
    game_result: GameResult,
    vote_amount: u64,
    bet_direction: RwSignal<Option<VoteKind>>,
    show_tutorial: RwSignal<bool>,
    video_uid: String,
    post: PostDetails,
) -> impl IntoView {
    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();
    let event_ctx = auth.event_ctx();
    let won = matches!(game_result, GameResult::Win { .. });
    let creator_reward_rounded =
        ((vote_amount * CREATOR_COMMISSION_PERCENT) as f64 / 100.0).ceil() as u64;
    let bet_direction_text = match bet_direction.get() {
        Some(VoteKind::Hot) => "Hot",
        Some(VoteKind::Not) => "Not",
        None => "",
    };
    let creator_reward_text = if creator_reward_rounded > 0 {
        format!(", creator gets {creator_reward_rounded} YRAL")
    } else {
        String::new()
    };
    let (line1, line2) = match game_result.clone() {
        GameResult::Win { win_amt } => {
            let total_win = TokenBalance::new((win_amt + vote_amount).into(), 0).humanize();
            if bet_direction_text.is_empty() {
                (
                    format!("You won {total_win} YRAL",),
                    "Tap ? to see how it works".into(),
                )
            } else {
                (
                    format!("You voted \"{bet_direction_text}\" - Spot on!"),
                    format!("You won {total_win} YRAL{creator_reward_text}",),
                )
            }
        }
        GameResult::Loss { lose_amt } => {
            let total_loss = TokenBalance::new(lose_amt.into(), 0).humanize();
            if bet_direction_text.is_empty() {
                (
                    format!("You lost {total_loss} YRAL"),
                    "Tap ? to see how it works".into(),
                )
            } else {
                (
                    format!("You voted \"{bet_direction_text}\" - wrong vote."),
                    format!("You lost {total_loss} YRAL{creator_reward_text}"),
                )
            }
        }
    };

    let bet_amount = vote_amount;
    let coin = match bet_amount {
        1 => CoinState::C1,
        5 => CoinState::C5,
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

    let (onboarding_store, _, _) =
        use_local_storage::<UserOnboardingStore, JsonSerdeCodec>(USER_ONBOARDING_STORE_KEY);
    let show_help_ping = RwSignal::new(true);

    Effect::new(move |_| {
        if onboarding_store.get_untracked().has_seen_hon_bet_help {
            show_help_ping.set(false);
        }
    });

    let show_ping = move || show_help_ping.get() && !won;

    let conclusion = match game_result {
        GameResult::Win { .. } => GameConclusion::Win,
        GameResult::Loss { .. } => GameConclusion::Loss,
    };

    let conclusion_cloned = conclusion.clone();
    let vote_amount_cloned = vote_amount;

    let tutorial_action = Action::new(move |_| {
        let video_id = video_uid.clone();
        let conclusion = conclusion_cloned.clone();
        let vote_amount = vote_amount_cloned;
        async move {
            if let Some(global) = MixpanelGlobalProps::from_ev_ctx(event_ctx) {
                MixPanelEvent::track_how_to_play_clicked(
                    global,
                    video_id,
                    MixpanelPostGameType::HotOrNot,
                    vote_amount,
                    StakeType::Sats,
                    bet_direction
                        .get_untracked()
                        .unwrap_or(VoteKind::Hot)
                        .into(),
                    conclusion,
                );
            }
        }
    });

    view! {
        <div class="flex w-full flex-col gap-3 py-2">
            <div class="flex gap-2 justify-center items-center w-full">
                <div class="relative shrink-0 drop-shadow-lg">
                    <CoinStateView class="w-14 h-14 md:w-16 md:h-16" coin is_connected />
                </div>
                <div class="flex-1 p-1 text-xs md:text-sm font-semibold leading-snug text-white rounded-full">
                    {line1}<br/>
                    {line2}
                </div>
                <button
                class="relative shrink-0 cursor-pointer"
                on:click=move |_| {
                        show_help_ping.set(false);
                        show_tutorial.set(true);
                        tutorial_action.dispatch(());
                    }>
                    <img src="/img/hotornot/question-mark.svg" class="h-8 w-8" />
                    <ShowAny when=move || show_ping()>
                        <span class="absolute top-1 right-1 ping rounded-full w-2 h-2 bg-[#F14331] text-[#F14331]"></span>
                    </ShowAny>
                </button>
            </div>
            {move || HnBetState::get(post.uid.clone()).map(|bet_res| {
                    view! {
                        <VideoScoreComparison
                            current_score=bet_res.current_video_score
                            previous_score=bet_res.previous_video_score
                            won
                            show_score=coin==CoinState::C1
                        />
                    }})
            }
            <TotalBalance won />
        </div>
    }
}

#[component]
fn TotalBalance(won: bool) -> impl IntoView {
    let (wallet_balance_store, _, _) =
        use_local_storage::<u64, FromToStringCodec>(WALLET_BALANCE_STORE_KEY);

    let total_balance_text = move || {
        let balance = HnBetState::get_balance().unwrap_or(0);
        format!("Total balance: {balance} YRAL")
    };

    Effect::new(move |_| {
        if HnBetState::get_balance().is_none() {
            let balance = wallet_balance_store.get();
            HnBetState::set_balance(balance);
        }
    });

    view! {
        <Show when=move|| HnBetState::get_balance().is_some()>
            <div class=format!("flex items-center text-sm font-semibold justify-center p-2 rounded-full {}", if won { "bg-[#158F5C]" } else { "bg-[#F14331]" })>
                {total_balance_text}
            </div>
        </Show>
    }
}

#[component]
fn VideoScoreComparison(
    current_score: f32,
    previous_score: f32,
    won: bool,
    show_score: bool,
) -> impl IntoView {
    let is_current_higher = current_score > previous_score;
    let comparison_symbol = if is_current_higher { ">" } else { "<" };
    let comparison_color = if won {
        "text-green-500"
    } else {
        "text-[#F14331]"
    };

    let mut current_score_int = current_score.round() as u32;
    let mut previous_score_int = previous_score.round() as u32;

    if current_score_int == previous_score_int {
        if won && is_current_higher {
            current_score_int += 1;
        } else {
            previous_score_int += 1;
        }
    }

    view! {
        <div class="flex justify-center items-center gap-2 bg-black/40 rounded-full px-2 py-2 text-white text-sm font-semibold">
            <div class="flex gap-2 items-center text-start">
                <Show when=move||show_score><span class="text-md">{current_score_int}</span></Show>
                <span class="text-xs">Current Video Score</span>
            </div>

            <span class=format!("text-lg font-bold {}", comparison_color)>
                {comparison_symbol}
            </span>

            <div class="flex gap-2 items-center text-start">
                <Show when=move||show_score><span class="text-md">{previous_score_int}</span></Show>
                <span class="text-xs">Previous Video Score</span>
            </div>
        </div>
    }
}

#[component]
pub fn HNUserParticipation(
    post: PostDetails,
    participation: GameInfo,
    refetch_bet: Trigger,
    bet_direction: RwSignal<Option<VoteKind>>,
    show_tutorial: RwSignal<bool>,
) -> impl IntoView {
    // let (_, _) = (post, refetch_bet); // not sure if i will need these later
    let _ = refetch_bet; // not sure if i will need these later
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
    let video_uid = post.uid.clone();

    let post = post.clone();

    view! {
        <HNWonLost game_result vote_amount bet_direction show_tutorial video_uid post />
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
    prev_post: Option<(Principal, u64)>,
    win_audio_ref: NodeRef<Audio>,
    show_tutorial: RwSignal<bool>,
    show_low_balance_popup: RwSignal<bool>,
) -> impl IntoView {
    let bet_direction = RwSignal::new(None::<VoteKind>);

    let refetch_bet = Trigger::new();
    let post = StoredValue::new(post);

    let auth = auth_state();

    let coin: RwSignal<CoinState> = use_context().unwrap_or_else(|| {
        RwSignal::new(if auth.is_logged_in_with_oauth().get_untracked() {
            DEFAULT_BET_COIN_FOR_LOGGED_IN
        } else {
            DEFAULT_BET_COIN_FOR_LOGGED_OUT
        })
    });

    let user_principal = auth.user_principal;
    let create_game_info = Resource::new(
        move || refetch_bet.track(),
        move |_| {
            refetch_bet.track();
            send_wrap(async move {
                let principal = user_principal.await?;

                let post = post.get_value();
                let game_info_req = GameInfoReqV3 {
                    publisher_principal: post.poster_principal,
                    post_id: post.post_id,
                };
                let game_info = fetch_game_with_sats_info_v3(
                    principal,
                    reqwest::Url::parse(WORKER_URL).unwrap(),
                    game_info_req,
                )
                .await
                .map_err(|err| ServerFnError::new(format!("{err:#?}")))?;

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
                                        bet_direction
                                        show_tutorial
                                    />
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <HNButtonOverlay
                                        post
                                        prev_post
                                        bet_direction
                                        coin
                                        refetch_bet
                                        audio_ref=win_audio_ref
                                        show_low_balance_popup
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
