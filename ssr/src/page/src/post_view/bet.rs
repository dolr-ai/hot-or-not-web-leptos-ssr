use crate::post_view::BetEligiblePostCtx;
use component::{
    bullet_loader::BulletLoader, canisters_prov::AuthCansProvider, hn_icons::*, spinner::SpinnerFit,
};
use leptos::{either::Either, prelude::*};
use leptos_icons::*;
use leptos_use::use_interval_fn;
use state::canisters::{authenticated_canisters, unauth_canisters};
use utils::{send_wrap, time::to_hh_mm_ss, try_or_redirect_opt};
use web_time::Duration;
use yral_canisters_client::individual_user_template::BettingStatus;
use yral_canisters_common::{
    utils::{
        posts::PostDetails,
        vote::{VoteDetails, VoteKind, VoteOutcome},
    },
    Canisters,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum CoinState {
    C50,
    C100,
    C200,
}

impl CoinState {
    fn wrapping_next(self) -> Self {
        match self {
            CoinState::C50 => CoinState::C100,
            CoinState::C100 => CoinState::C200,
            CoinState::C200 => CoinState::C50,
        }
    }

    fn wrapping_prev(self) -> Self {
        match self {
            CoinState::C50 => CoinState::C200,
            CoinState::C100 => CoinState::C50,
            CoinState::C200 => CoinState::C100,
        }
    }
}

impl From<CoinState> for u64 {
    fn from(coin: CoinState) -> u64 {
        match coin {
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
            on:click=move |_| bet_direction.set(Some(kind))
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
    coin: RwSignal<CoinState>,
    bet_direction: RwSignal<Option<VoteKind>>,
    refetch_bet: Trigger,
) -> impl IntoView {
    let place_bet_action = Action::new(
        move |(canisters, bet_direction, bet_amount): &(Canisters<true>, VoteKind, u64)| {
            let post_can_id = post.canister_id;
            let post_id = post.post_id;
            let cans = canisters.clone();
            let bet_amount = *bet_amount;
            let bet_direction = *bet_direction;
            send_wrap(async move {
                match cans
                    .vote_on_post(bet_amount, bet_direction, post_id, post_can_id)
                    .await
                {
                    Ok(_) => Some(()),
                    Err(e) => {
                        log::error!("{e}");
                        None
                    }
                }
            })
        },
    );
    let place_bet_res = place_bet_action.value();
    Effect::new(move |_| {
        if place_bet_res().flatten().is_some() {
            refetch_bet.notify();
        }
    });
    let running = place_bet_action.pending();

    let BetEligiblePostCtx { can_place_bet } = expect_context();

    Effect::new(move |_| {
        if !running.get() {
            can_place_bet.set(true)
        } else {
            can_place_bet.set(false)
        }
    });

    view! {
        <AuthCansProvider let:canisters>

            {
                Effect::new(move |_| {
                    let Some(bet_direction) = bet_direction() else {
                        return;
                    };
                    let bet_amount = coin.get_untracked().into();
                    place_bet_action.dispatch((canisters.clone(), bet_direction, bet_amount));
                });
            }

        </AuthCansProvider>

        <div class="flex justify-center w-full touch-manipulation">
            <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_next())>
                <Icon attr:class="justify-self-end text-2xl text-white" icon=icondata::AiUpOutlined />
            </button>
        </div>
        <div class="flex flex-row gap-6 justify-center items-center w-full touch-manipulation">
            <HNButton disabled=running bet_direction kind=VoteKind::Hot />
            <button disabled=running on:click=move |_| coin.update(|c| *c = c.wrapping_next())>
                <CoinStateView
                    disabled=running
                    class="w-12 h-12 md:w-14 md:h-14 lg:w-16 lg:h-16 drop-shadow-lg"
                    coin
                />

            </button>
            <HNButton disabled=running bet_direction kind=VoteKind::Not />
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
        <button class="py-2 px-4 w-full text-sm font-bold text-black bg-white rounded-sm">
            <Icon attr:class="fill-white" style="" icon=icondata::RiTrophyFinanceFill />

            "You Lost"
        </button>
    }
}

#[component]
fn HNWonLost(participation: VoteDetails) -> impl IntoView {
    let won = matches!(participation.outcome, VoteOutcome::Won(_));
    let bet_amount = participation.vote_amount;
    let coin = match bet_amount {
        50 => CoinState::C50,
        100 => CoinState::C100,
        200 => CoinState::C200,
        amt => {
            log::warn!("Invalid bet amount: {amt}, using fallback");
            CoinState::C50
        }
    };
    let is_hot = matches!(participation.vote_kind, VoteKind::Hot);
    let hn_icon = if is_hot { HotIcon } else { NotIcon };

    view! {
        <div class="flex gap-6 justify-center items-center p-4 w-full bg-transparent rounded-xl shadow-sm">
            <div class="relative flex-shrink-0 drop-shadow-lg">
                <CoinStateView class="w-14 h-14 md:w-16 md:h-16" coin />
                <Icon attr:class="absolute -bottom-0.5 -right-3 w-7 h-7 md:w-9 md:h-9" icon=hn_icon />

            </div>

            // <!-- Text and Badge Column -->
            <div class="flex flex-col gap-2 w-full md:w-1/2 lg:w-1/3">
                // <!-- Result Text -->
                <div class="p-1 text-sm leading-snug text-white rounded-full">
                    <p>You staked {bet_amount}tokens on {if is_hot { "Hot" } else { "Not" }}.</p>
                    <p>
                        {if let Some(reward) = participation.reward() {
                            format!("You received {reward} tokens.")
                        } else {
                            format!("You lost {bet_amount} tokens.")
                        }}
                    </p>

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
fn BetTimer(post: PostDetails, participation: VoteDetails, refetch_bet: Trigger) -> impl IntoView {
    let bet_duration = participation.vote_duration().as_secs();
    let time_remaining = RwSignal::new(participation.time_remaining(post.created_at));
    _ = use_interval_fn(
        move || {
            time_remaining.try_update(|t| *t = t.saturating_sub(Duration::from_secs(1)));
            _ = refetch_bet;
            // if time_remaining.try_get_untracked() == Some(Duration::ZERO) {
            //     refetch_bet.notify();
            // }
        },
        1000,
    );

    let percentage = Memo::new(move |_| {
        let remaining_secs = time_remaining().as_secs();
        100 - ((remaining_secs * 100) / bet_duration).min(100)
    });
    let gradient = move || {
        let perc = percentage();
        format!("background: linear-gradient(to right, rgb(var(--color-primary-600)) {perc}%, #00000020 0 {}%);", 100 - perc)
    };

    view! {
        <div
            class="flex flex-row gap-1 justify-end items-center py-px w-full text-base text-white rounded-full md:text-lg pe-4"
            style=gradient
        >

            <Icon icon=icondata::AiClockCircleFilled />
            <span>{move || to_hh_mm_ss(time_remaining())}</span>
        </div>
    }
}

#[component]
fn HNAwaitingResults(
    post: PostDetails,
    participation: VoteDetails,
    refetch_bet: Trigger,
) -> impl IntoView {
    let is_hot = matches!(participation.vote_kind, VoteKind::Hot);
    let bet_direction_text = if is_hot { "Hot" } else { "Not" };
    let hn_icon = if is_hot { HotIcon } else { NotIcon };

    let bet_amount = participation.vote_amount;
    let coin = match bet_amount {
        50 => CoinState::C50,
        100 => CoinState::C100,
        200 => CoinState::C200,
        amt => {
            log::warn!("Invalid bet amount: {amt}, using fallback");
            CoinState::C50
        }
    };

    view! {
        <div class="flex flex-col gap-1 items-center p-4 w-full shadow-sm">
            <div class="flex flex-row gap-4 justify-center items-end w-full">
                <div class="relative flex-shrink-0 drop-shadow-lg">
                    <Icon attr:class="w-12 h-12 md:w-14 md:h-14 lg:w-16 lg:h-16" icon=hn_icon />
                    <CoinStateView
                        class="absolute bottom-0 -right-3 w-7 h-7 md:w-9 md:h-9 lg:w-11 lg:h-11"
                        coin
                    />

                </div>
                <div class="w-1/2 md:w-1/3 lg:w-1/4">
                    <BetTimer post refetch_bet participation />
                </div>
            </div>
            <p class="p-1 text-center text-white rounded-full bg-black/15 ps-2">
                You staked {bet_amount}tokens on {bet_direction_text}.
                Result is still pending.

            </p>
        </div>
    }
}

#[component]
pub fn HNUserParticipation(
    post: PostDetails,
    participation: VoteDetails,
    refetch_bet: Trigger,
) -> impl IntoView {
    view! {
        {match participation.outcome {
            VoteOutcome::AwaitingResult => {
                view! { <HNAwaitingResults post refetch_bet participation /> }.into_any()
            }
            VoteOutcome::Won(_) => {
                view! { <HNWonLost participation /> }.into_any()
            }
            VoteOutcome::Draw(_) => {
                view! { "Draw" }.into_any()
            }
            VoteOutcome::Lost => {
                view! { <HNWonLost participation /> }.into_any()
            }
        }
            .into_view()}
        <ShadowBg />
    }
}

#[component]
fn MaybeHNButtons(
    post: PostDetails,
    bet_direction: RwSignal<Option<VoteKind>>,
    coin: RwSignal<CoinState>,
    refetch_bet: Trigger,
) -> impl IntoView {
    let post = StoredValue::new(post);
    let is_betting_enabled: Resource<Option<bool>> = Resource::new(
        move || (),
        move |_| {
            let post = post.get_value();
            send_wrap(async move {
                let canisters = unauth_canisters();
                let user = canisters.individual_user(post.canister_id).await;
                let res = user
                    .get_hot_or_not_bet_details_for_this_post(post.post_id)
                    .await
                    .ok()?;
                Some(matches!(res, BettingStatus::BettingOpen { .. }))
            })
        },
    );
    let BetEligiblePostCtx { can_place_bet } = expect_context();

    view! {
        <Suspense fallback=LoaderWithShadowBg>
            {move || {
                is_betting_enabled.get()
                    .and_then(|enabled| {
                        if !enabled.unwrap_or_default() {
                            can_place_bet.set(false);
                            return None;
                        }
                        Some(
                            view! {
                                <HNButtonOverlay
                                    post=post.get_value()
                                    bet_direction
                                    coin
                                    refetch_bet
                                />
                            },
                        )
                    })
            }}

        </Suspense>
    }
    .into_any()
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
            class="absolute bottom-0 left-0 h-2/5 w-dvw -z-[1]"
            style="background: linear-gradient(to bottom, #00000000 0%, #00000099 45%, #000000a8 100%, #000000cc 100%, #000000a8 100%);"
        ></div>
    }
}

#[component]
pub fn HNGameOverlay(post: PostDetails) -> impl IntoView {
    let bet_direction = RwSignal::new(None::<VoteKind>);
    let coin = RwSignal::new(CoinState::C50);

    let refetch_bet = Trigger::new();
    let post = StoredValue::new(post);

    // let create_bet_participation_outcome = move |canisters: Canisters<true>| {
    //     // TODO: leptos 0.7, switch to `create_resource`
    //     LocalResource::new(
    //         // MockPartialEq is necessary
    //         // See: https://github.com/leptos-rs/leptos/issues/2661
    //         move || {
    //             refetch_bet.track();
    //             let cans = canisters.clone();
    //             async move {
    //                 let post = post.get_value();
    //                 let user = cans.authenticated_user().await;
    //                 let bet_participation = user
    //                     .get_individual_hot_or_not_bet_placed_by_this_profile(
    //                         post.canister_id,
    //                         post.post_id,
    //                     )
    //                     .await?;
    //                 Ok::<_, ServerFnError>(bet_participation.map(VoteDetails::from))
    //             }
    //         },
    //     )
    // };

    let create_bet_participation_outcome = Resource::new(
        move || (),
        move |_| {
            refetch_bet.track();
            send_wrap(async move {
                let cans = authenticated_canisters().await?;
                let cans = Canisters::from_wire(cans, expect_context())?;
                let post = post.get_value();
                let user = cans.authenticated_user().await;
                let bet_participation =
                    send_wrap(user.get_individual_hot_or_not_bet_placed_by_this_profile(
                        post.canister_id,
                        post.post_id,
                    ))
                    .await?;
                Ok::<_, ServerFnError>(bet_participation.map(VoteDetails::from))
            })
        },
    );
    view! {
        <Suspense fallback=LoaderWithShadowBg>

            {
                move || {
                    create_bet_participation_outcome.get()
                    .and_then(|res| {
                        let participation = try_or_redirect_opt!(res.as_ref());
                        let post = post.get_value();
                        Some(
                            if let Some(participation) = participation {
                                view! {
                                    <HNUserParticipation post refetch_bet participation=participation.clone() />
                                }.into_any()
                            } else {
                                view! {
                                    <MaybeHNButtons post bet_direction coin refetch_bet />
                                }.into_any()
                            },
                        )
                    })
                    .unwrap_or_else(|| view! { <LoaderWithShadowBg /> }.into_any())
                }

            }

        </Suspense>
    }
}
