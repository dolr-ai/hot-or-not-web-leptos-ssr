use crate::{buttons::HighlightedButton, overlay::ShadowOverlay};
use codee::string::FromToStringCodec;
use consts::NSFW_TOGGLE_STORE;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_use::storage::use_local_storage;

#[component]
pub fn NsfwUnlockPopup(show: RwSignal<bool>) -> impl IntoView {
    let agreed = RwSignal::new(true);
    let (nsfw_enabled, set_nsfw_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
    let check_show = Signal::derive(move || show.get() && !nsfw_enabled.get_untracked());

    view! {
        <ShadowOverlay show=check_show>
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div
                    class="relative overflow-hidden h-fit max-w-md w-full pt-16 rounded-md bg-neutral-950"
                    style="background-image: url('/img/yral/nsfw_nudge.png'); background-size: cover; background-position: center;"
                >
                    <div class="absolute inset-0 z-[1]"></div>

                    <button
                        on:click=move |_| show.set(false)
                        class="text-white rounded-full flex items-center justify-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>

                    <div class="flex z-[2] relative flex-col items-center gap-6 text-white justify-center p-12">
                        <div class="text-center text-2xl font-semibold">
                            "Adults Only — Unlock the Fun"
                        </div>

                        <div class="text-center text-sm text-neutral-300 leading-relaxed">
                            "Missing out on spicy video?"<br />
                            "Enable NSFW to view — 18+ only."
                        </div>

                        <label class="flex items-center gap-2 text-neutral-200 text-sm">
                            <input
                                type="checkbox"
                                class="accent-pink-500 w-5 h-5"
                                bind:checked=agreed
                            />
                            <span>
                                "I’m 18+ and agree to all "
                                <a href="/privacy-policy" class="underline">"content policy"</a>
                            </span>
                        </label>
                        {
                            move || {
                                let disabled = !agreed.get();
                                view!{
                                    <HighlightedButton
                                        alt_style=true
                                        disabled=disabled
                                        on_click=move || {
                                            if agreed.get() {
                                                set_nsfw_enabled.set(true);
                                                let window = window();
                                                let _ = window
                                                    .location()
                                                    .set_href(&format!("/?nsfw={}", nsfw_enabled.get_untracked()));
                                            }
                                        }
                                    >
                                        "Unlock 18+ Content"
                                    </HighlightedButton>
                                }
                            }
                        }
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
