use crate::{
    icpump::ProcessedTokenListResponse,
    pumpdump::{
        components::{
            button::{DumpButton, PumpButton},
            slider::BullBearSlider,
        },
        GameResult, RunningGameRes, ShowOnboarding,
    },
};
use component::icons::chevron_right_icon::ChevronRightIcon;
use leptos::html;
use leptos::{either::Either, prelude::*};
use leptos_icons::*;

#[component]
pub fn PlayingCard() -> impl IntoView {
    let token: ProcessedTokenListResponse = expect_context();
    let show_onboarding: ShowOnboarding = expect_context();
    let game_res: RunningGameRes = expect_context();
    // let running_data: RwSignal<Option<GameRunningData>> = expect_context();
    let winning_pot = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return "--".to_string();
        };
        ctx.with_running_data(|data| data.winning_pot)
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "--".into())
    };

    let token_link = token.token_details.link.clone();

    let player_count = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return "--".to_string();
        };
        ctx.with_running_data(|data| data.player_count.to_string())
            .unwrap_or_else(|| "--".into())
    };

    let audio_ref = NodeRef::<html::Audio>::new();

    view! {
        <div
            class="bg-neutral-900 flip-card transition-all absolute inset-0 h-full shrink-0 rounded-2xl flex flex-col gap-4 pt-14 pb-5 px-5 overflow-hidden"
        >
            <div class="flex flex-col gap-6 w-full items-center">
                <img class="size-32 rounded-sm" alt="Avatar" src=token.token_details.logo />
                <a href=token_link class="flex items-center gap-1">
                    <div class="font-bold text-lg">{token.token_details.token_name}</div>
                    <Icon icon=ChevronRightIcon attr:class="w-5 h-5 -mb-px" />
                </a>
                <div class="bg-[#212121] shrink-0 rounded-full relative w-full h-11 overflow-hidden">
                    <div
                        class="w-full animate-slide-up top-14 flex items-center gap-2 justify-between absolute inset-0 py-2 pl-4 pr-2"
                    >
                        <div class="flex items-center gap-1">
                            <div class="text-neutral-400 text-xs">Winning Pot:</div>
                            <img src="/img/pumpdump/cents.webp" alt="Coin" class="size-5" />
                            <div class="text-neutral-200 font-bold">{winning_pot} Cents</div>
                        </div>
                        <button
                            on:click=move |_| show_onboarding.show()
                            class="bg-black text-neutral-400 font-bold hover:bg-black/35 rounded-full text-xl w-7 h-7 flex items-center justify-center leading-none"
                        >
                            ?
                        </button>
                    </div>
                    <div
                        style="--animation-delay:5s;"
                        class="w-full top-14 animate-slide-up flex items-center gap-1 absolute inset-0 py-2 pl-4 pr-2"
                    >
                        <img src="/img/pumpdump/player.webp" alt="Coin" class="w-5 h-5" />
                        <div class="text-neutral-200 font-bold">{player_count}</div>
                        <div class="text-neutral-400 text-xs">players are playing - join the action!</div>
                    </div>
                </div>
            </div>
            <div class="flex select-none flex-col gap-4 h-34 w-full">
                <BullBearSlider />
                <div
                    class="flex relative items-center gap-6 justify-center w-full"
                >
                    <audio node_ref=audio_ref preload="auto" src="/img/pumpdump/tap.mp3"/>
                    <DumpButton audio_ref />

                    <PumpButton audio_ref />
                </div>
            </div>
        </div>
    }
}

#[component]
fn WonCard(win_amount: u128) -> impl IntoView {
    let game_res: RunningGameRes = expect_context();
    let loading_data = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return None;
        };
        Some(ctx.loading_data())
    };
    let pending = move || {
        let Some(loading) = loading_data() else {
            return true;
        };
        loading.get()
    };

    let on_click = move |_| {
        let Some(Ok(ctx)) = game_res.get() else {
            return;
        };

        ctx.reload_running_data.dispatch(());
    };

    // TODO: add confetti animation
    view! {
        <div
            style="background-size: cover; background-position: center; background-image: url('/img/pumpdump/onboarding-bg.webp');"
            class="rounded-2xl overflow-hidden flip-card card-flipped absolute inset-0 h-full w-full shrink-0 items-center justify-center flex flex-col gap-7 pt-14 pb-5 px-5"
        >
            <img src="/img/pumpdump/trophy.webp" alt="Trophy" class="w-32 h-[7.6rem] translate-x-3" />
            <div class="flex flex-col gap-4 items-center">
                <div class="font-semibold text-xl">Victory is yours!</div>
                <div class="text-neutral-400 text-center">
                    {"Your strategy paid off! The tide shifted to your side, and you've won big. 💰"}
                </div>
                <div
                    class="bg-[#212121] w-full px-4 py-2 rounded-full flex items-center justify-center gap-2"
                >
                    <span class="text-neutral-400 text-xs">You have won:</span>
                    <img src="/img/pumpdump/cents.webp" alt="Coin" class="w-5 h-5" />
                    <span class="text-neutral-200 font-bold">{win_amount} Cents</span>
                </div>
            </div>
            <button
                disabled=pending
                on:click=on_click
                class="w-full px-5 py-3 rounded-lg flex items-center transition-all justify-center gap-8 font-kumbh font-bold"
                style:background="linear-gradient(73deg, #DA539C 0%, #E2017B 33%, #5F0938 100%)"
            >
                <Show when=pending fallback=|| "Start playing again">
                    {"Starting another round..."}
                </Show>
            </button>
        </div>
    }
}

#[component]
fn LostCard() -> impl IntoView {
    let game_res: RunningGameRes = expect_context();
    let loading_data = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return None;
        };
        Some(ctx.loading_data())
    };
    let pending = move || {
        let Some(loading) = loading_data() else {
            return true;
        };
        loading.get()
    };

    let on_click = move |_| {
        let Some(Ok(ctx)) = game_res.get() else {
            return;
        };

        ctx.reload_running_data.dispatch(());
    };

    view! {
        <div
            style="background: radial-gradient(100% 100% at -14% 74%, rgba(46, 124, 246, 0.16) 0%, rgba(23, 23, 23, 1) 100%);"
            class="rounded-2xl flip-card card-flipped absolute inset-0 h-full w-full shrink-0 items-center justify-center flex flex-col gap-7 pt-14 pb-5 px-5"
        >
            <img src="/img/pumpdump/sadface.webp" alt="Sad face emoji" class="h-36 w-36" />
            <div class="flex flex-col gap-4 items-center">
                <div class="font-semibold text-xl">The Tide Turned Against You!</div>
                <div class="text-neutral-400 text-center">
                    The other side took the lead this time, but every vote brings you closer to your next win.
                    Stay in the game!
                </div>
            </div>
            <button
                disabled=pending
                on:click=on_click
                class="w-full px-5 py-3 rounded-lg flex items-center transition-all justify-center gap-8 font-kumbh font-bold"
                style:background="linear-gradient(73deg, #DA539C 0%, #E2017B 33%, #5F0938 100%)"
            >
                <Show when=pending fallback=|| "Keep Playing">
                    {"Starting another round..."}
                </Show>
            </button>
        </div>
    }
}

#[component]
pub fn ResultDeclaredCard(result: GameResult) -> impl IntoView {
    match result {
        GameResult::Loss { .. } => Either::Left(view! {
            <LostCard />
        }),
        GameResult::Win { amount } => Either::Right(view! {
            <WonCard win_amount=amount/>
        }),
    }
}
