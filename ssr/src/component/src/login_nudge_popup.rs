use leptos::prelude::*;
use leptos_icons::*;

use crate::{buttons::HighlightedButton, login_icons::*, overlay::ShadowOverlay};

#[component]
pub fn LoginNudgePopup(show: RwSignal<bool>, show_login_popup: RwSignal<bool>) -> impl IntoView {
    view! {
        <ShadowOverlay show=show>
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div class="overflow-hidden h-fit max-w-md items-center pt-16 cursor-auto bg-neutral-950 rounded-md w-full relative">
                    <div
                        style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                        class="absolute z-[1] -left-1/2 bottom-1/3 size-[32rem]" >
                    </div>

                    <button
                        on:click=move |_| {
                            show.set(false);
                        }
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>

                    <div class="flex z-[2] relative flex-col items-center gap-6 text-white justify-center p-12">
                        <img src="/img/login/unlock.png" class="h-24" alt="Unlock Icon" />

                        <div class="text-center text-2xl font-semibold">Unlock Higher Bets</div>

                        <div class="text-center text-sm text-neutral-300">
                            "You're just 1 step away from high-stake fun."<br />
                            Log in to unlock 5 YRAL bets!
                        </div>

                        <div class="flex flex-col px-8 items-start gap-4 w-full max-w-xs text-sm">
                            <span class="text-center text-sm text-neutral-300">Why log in?</span>

                            <div class="flex items-center gap-2 text-neutral-200">
                                <Icon icon=Crown />
                                <span>Higher bets, higher rewards</span>
                            </div>
                            <div class="flex items-center gap-2 text-neutral-200">
                                <Icon icon=Dollar />
                                <span>Enable withdrawals</span>
                            </div>
                            <div class="flex items-center gap-2 text-neutral-200">
                                <Icon icon=Airdrop />
                                <span>Daily DOLR / YRAL airdrops</span>
                            </div>
                        </div>


                        <HighlightedButton
                            alt_style=false
                            disabled=false
                            on_click=move || {
                                show.set(false);
                                show_login_popup.set(true);
                            }
                        >
                            "Login Now"
                        </HighlightedButton>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
