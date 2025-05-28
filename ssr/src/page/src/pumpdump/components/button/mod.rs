mod particles;

use crate::pumpdump::ProcessedTokenListResponse;
use crate::pumpdump::{PlayerDataRes, RunningGameRes};
use leptos::{html::Audio, prelude::*, tachys::dom::window};
use particles::{FireBubbles, SkullBubbles};
use state::canisters::auth_state;
use utils::event_streaming::events::TokenPumpedDumped;
use yral_pump_n_dump_common::GameDirection;

fn non_visual_feedback(audio_ref: NodeRef<Audio>) {
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
            nav.vibrate_with_duration(5);
        } else {
            log::debug!("browser does not support vibrate");
        }
        let Some(audio) = audio_ref.get() else {
            return;
        };
        audio.set_current_time(0.);
        audio.set_volume(0.5);
        _ = audio.play();
    }
}

#[component]
pub fn DumpButton(audio_ref: NodeRef<Audio>) -> impl IntoView {
    let game_res: RunningGameRes = expect_context();
    let player_data: PlayerDataRes = expect_context();
    let token: ProcessedTokenListResponse = expect_context();
    let has_no_balance = move || {
        player_data
            .read
            .get()
            .is_some_and(|d| d.is_ok_and(|d| d.get().wallet_balance == 0))
    };
    let counter = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return "-".to_string();
        };
        ctx.with_running_data(|v| v.dumps.to_string())
            .unwrap_or_else(|| "-".into())
    };

    let spawn_bubbles = RwSignal::new(0u32);

    let press_count = RwSignal::new(0u32);

    let auth = auth_state();
    let event_ctx = auth.event_ctx();

    let send_event = leptos_use::use_debounce_fn(
        move || {
            let count = press_count.get();
            if count > 0 {
                let token_details = token.token_details.clone();
                let token_root = token.root;
                let press_count_value = count;

                TokenPumpedDumped.send_event(
                    event_ctx,
                    token_details.token_name,
                    token_root,
                    "dump".to_string(),
                    press_count_value,
                );

                // Reset the counter after sending
                press_count.set(0);
            }
        },
        1000.0, // 1 second debounce
    );

    let onclick = move |_| {
        non_visual_feedback(audio_ref);
        spawn_bubbles.update(|b| *b += 1);
        let Some(Ok(ctx)) = game_res.get() else {
            return;
        };
        ctx.send_bet(GameDirection::Dump);

        press_count.update(|count| *count += 1);
        send_event();
    };

    view! {
        <div class="relative">
            <div class="absolute -top-24 h-36 left-0">
                <SkullBubbles spawn_bubbles/>
            </div>
            <button
                aria-label="Vibration"
                on:click=onclick
                disabled=has_no_balance
                class="dump-button transition duration-300 disabled:grayscale rounded-[28px] transition-all duration-150 ring-4 group text-white ring-white/25 gap-2 min-w-36 p-3 flex flex-col items-center justify-center touch-none"
            >
                <div class="text-xl font-bold">DUMP</div>
                <div class="bg-[#4683DC] rounded-full w-12 h-3 relative">
                    <div
                        class="w-full h-full relative overflow-hidden font-bold text-xs items-center flex justify-center"
                    >
                            <span
                                class="absolute inset-0 flex items-center justify-center"
                            >
                                {counter}
                            </span>
                    </div>
                    <img
                        src="/img/pumpdump/skull.webp"
                        class="absolute w-6 h-6 -left-3 -top-1/2 transition group-active:saturate-150 group-active:scale-110 group-active:rotate-12"
                        alt="DUMP"
                    />
                </div>
            </button>
        </div>
    }
}

#[component]
pub fn MockDumpButton() -> impl IntoView {
    view! {
        <button
            aria-label="Vibration"
            class="dump-button rounded-[28px] transition-all duration-150 ring-4 group text-white ring-white/25 gap-2 min-w-36 p-3 flex flex-col items-center justify-center"
        >
            <div class="text-xl font-bold">DUMP</div>
            <div class="bg-[#4683DC] rounded-full w-12 h-3 relative">
                <div
                    class="w-full h-full relative overflow-hidden font-bold text-xs items-center flex justify-center"
                >
                        <span
                            class="absolute inset-0 flex items-center justify-center"
                        >
                            0
                        </span>
                </div>
                <img
                    src="/img/pumpdump/skull.webp"
                    class="absolute w-6 h-6 -left-3 -top-1/2 transition group-active:saturate-150 group-active:scale-110 group-active:rotate-12"
                    alt="DUMP"
                />
            </div>
        </button>
    }
}

#[component]
pub fn PumpButton(audio_ref: NodeRef<Audio>) -> impl IntoView {
    let game_res: RunningGameRes = expect_context();
    let player_data: PlayerDataRes = expect_context();
    let token: ProcessedTokenListResponse = expect_context();
    let has_no_balance = move || {
        player_data
            .read
            .get()
            .is_some_and(|d| d.is_ok_and(|d| d.get().wallet_balance == 0))
    };
    let counter = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return "-".to_string();
        };
        ctx.with_running_data(|v| v.pumps.to_string())
            .unwrap_or_else(|| "-".into())
    };

    let spawn_bubbles = RwSignal::new(0u32);
    let press_count = RwSignal::new(0u32);

    let auth = auth_state();
    let event_ctx = auth.event_ctx();

    let send_event = leptos_use::use_debounce_fn(
        move || {
            let count = press_count.get();
            if count > 0 {
                let token_details = token.token_details.clone();
                let token_root = token.root;
                let press_count_value = count;

                TokenPumpedDumped.send_event(
                    event_ctx,
                    token_details.token_name,
                    token_root,
                    "pump".to_string(),
                    press_count_value,
                );

                // Reset the counter after sending
                press_count.set(0);
            }
        },
        1000.0, // 1 second debounce
    );

    let onclick = move |_| {
        non_visual_feedback(audio_ref);
        spawn_bubbles.update(|b| *b += 1);
        let Some(Ok(ctx)) = game_res.get() else {
            return;
        };
        ctx.send_bet(GameDirection::Pump);

        press_count.update(|count| *count += 1);
        send_event();
    };

    view! {
        <div class="relative">
            <div class="absolute -top-24 h-36 right-0">
                <FireBubbles spawn_bubbles/>
            </div>
            <button
                aria-label="Vibration"
                on:click=onclick
                disabled=has_no_balance
                class="pump-button transition duration-300 disabled:grayscale rounded-[28px] transition-all duration-150 ring-4 group text-white ring-white/25 gap-2 min-w-36 p-3 flex flex-col items-center justify-center touch-none"
            >
                <div class="text-xl font-bold">PUMP</div>
                <div class="bg-[#E2027B] rounded-full w-12 h-3 relative">
                    <div
                        class="w-full h-full relative overflow-hidden font-bold text-xs items-center flex justify-center"
                    >
                        <span
                            class="absolute inset-0 flex items-center justify-center"
                        >
                            {counter}
                        </span>
                    </div>
                    <img
                        src="/img/pumpdump/fire.webp"
                        class="absolute w-6 h-6 -left-3 -top-1/2 transition group-active:saturate-150 group-active:scale-110 group-active:-rotate-12"
                        alt="PUMP"
                    />
                </div>
            </button>
        </div>
    }
}

#[component]
pub fn MockPumpButton() -> impl IntoView {
    view! {
        <button
            aria-label="Vibration"
            class="pump-button rounded-[28px] transition-all duration-150 ring-4 group text-white ring-white/25 gap-2 min-w-36 p-3 flex flex-col items-center justify-center"
        >
            <div class="text-xl font-bold">PUMP</div>
            <div class="bg-[#E2027B] rounded-full w-12 h-3 relative">
                <div
                    class="w-full h-full relative overflow-hidden font-bold text-xs items-center flex justify-center"
                >
                    <span
                        class="absolute inset-0 flex items-center justify-center"
                    >
                        0
                    </span>
                </div>
                <img
                    src="/img/pumpdump/fire.webp"
                    class="absolute w-6 h-6 -left-3 -top-1/2 transition group-active:saturate-150 group-active:scale-110 group-active:-rotate-12"
                    alt="PUMP"
                />
            </div>
        </button>
    }
}
