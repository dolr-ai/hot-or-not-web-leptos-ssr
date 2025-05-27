use leptos::prelude::*;

use crate::pumpdump::RunningGameRes;

#[component]
pub fn BullBearSlider() -> impl IntoView {
    let game_res: RunningGameRes = expect_context();
    let position = move || {
        let Some(Ok(ctx)) = game_res.get() else {
            return 39f64;
        };

        let ratio = ctx
            .with_running_data(|d| (d.dumps as f64 + 1.0) / (d.pumps as f64 + 1.0))
            .unwrap_or(1f64);
        if ratio == 1f64 {
            39f64
        } else {
            78f64.min(0f64.max(ratio * 78f64 / (ratio + 1f64)))
        }
    };

    let is_bear_attacking = Memo::new(move |prev_state| {
        let Some(Ok(ctx)) = game_res.get() else {
            return (None, 0u64, 0u64);
        };

        let Some((new_dumps, new_pumps)) = ctx.with_running_data(|d| (d.dumps, d.pumps)) else {
            return (None, 0u64, 0u64);
        };
        // state was reset
        if new_dumps == 0 && new_pumps == 0 {
            return (None, 0u64, 0u64);
        }

        let (prev, prev_dumps, prev_pumps) = prev_state.copied().unwrap_or((None, 0u64, 0u64));

        if new_dumps > prev_dumps {
            return (Some(true), new_dumps, new_pumps);
        } else if new_pumps > prev_pumps {
            return (Some(false), new_dumps, new_pumps);
        }

        (prev, prev_dumps, prev_pumps)
    });
    let anim_classes = Signal::derive(move || {
        let (Some(is_bear_attacking), _, _) = is_bear_attacking() else {
            return ("", "");
        };

        if is_bear_attacking {
            ("animate-push-right", "animate-shake")
        } else {
            ("animate-shake", "animate-push-left")
        }
    });

    view! {
        <div class="py-5 w-full">
            <div
                style="background: linear-gradient(90deg, #3D8EFF 0%, #390059 51.5%, #E2017B 100%);"
                class="relative ring-4 ring-[#212121] rounded-full w-full h-2"
            >
                <div
                    class="flex absolute inset-0 transition-all duration-700 ease-in-out gap-1 items-center"
                    style:left=move || format!("{}%", position())
                >
                    <img
                        style="filter: drop-shadow( -3px 3px 2px rgba(0, 0, 0, .7));"
                        src="/img/pumpdump/bear.webp"
                        alt="Bear"
                        class=move || format!("h-6 {}", anim_classes.with(|c| c.0))
                    />
                    <img
                        style="filter: drop-shadow( 3px 3px 2px rgba(0, 0, 0, .7));"
                        src="/img/pumpdump/bull.webp"
                        alt="Bull"
                        class=move || format!("h-7 {}", anim_classes.with(|c| c.1))
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn MockBullBearSlider() -> impl IntoView {
    view! {
        <div class="py-5 w-full">
            <div
                style="background: linear-gradient(90deg, #3D8EFF 0%, #390059 51.5%, #E2017B 100%);"
                class="relative ring-4 ring-[#212121] rounded-full w-full h-2"
            >
                <div
                    class="flex absolute inset-0 transition-all duration-700 ease-in-out gap-1 items-center"
                    style:left="39%"
                >
                    <img
                        style="filter: drop-shadow( -3px 3px 2px rgba(0, 0, 0, .7));"
                        src="/img/pumpdump/bear.webp"
                        alt="Bear"
                        class="h-6 push-right shake"
                    />
                    <img
                        style="filter: drop-shadow( 3px 3px 2px rgba(0, 0, 0, .7));"
                        src="/img/pumpdump/bull.webp"
                        alt="Bull"
                        class="h-7 push-left shake"
                    />
                </div>
            </div>
        </div>
    }
}
