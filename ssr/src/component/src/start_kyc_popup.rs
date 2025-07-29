use crate::{buttons::HighlightedButton, overlay::ShadowOverlay};
use leptos::prelude::*;
use leptos_icons::*;
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};
use state::kyc_state::{KycState, KycStatus};

#[component]
pub fn StartKycPopup(show: RwSignal<bool>) -> impl IntoView {
    view! {
        <ShadowOverlay show=show>
        {
            move || match KycState::get().get() {
                KycStatus::Pending => view! { <StartKyc show=show/> }.into_any(),
                KycStatus::InProgress => view! { <VerificationStatusPopup show=show is_verified=RwSignal::new(false) /> }.into_any(),
                KycStatus::Verified => view! { <VerificationStatusPopup show=show is_verified=RwSignal::new(true) /> }.into_any(),
            }
        }
        </ShadowOverlay>
    }
}

#[component]
pub fn StartKyc(show: RwSignal<bool>) -> impl IntoView {
    view! {
            <div class="w-full h-full flex items-center justify-center">
                <div class="overflow-hidden h-fit max-w-md items-center cursor-auto bg-[#171717] rounded-md w-full relative">
                    <button
                        on:click=move |_| show.set(false)
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>

                    <div class="flex z-[2] relative flex-col items-center gap-5 text-white justify-center p-10 pb-8">
                        <div class="relative">
                            <div class="absolute inset-0 rounded-full bg-neutral-800/50 size-32 m-auto" />
                            <img src="/img/kyc/kyc_mobile.svg" class="relative h-36 z-10" alt="Verify Icon" />
                        </div>

                        <div class="text-center text-xl font-semibold mt-2">Verify to unlock higher limits</div>

                        <div class="w-full text-center items-start text-sm text-neutral-300 flex flex-col gap-1">
                            <span>Get access to:</span>
                            <span class="px-2">"• Daily withdrawals: 50 – 1000 SATS"</span>
                            <span class="px-2">"• Faster withdrawals & added security"</span>
                        </div>

                        <div class="w-full my-4 bg-neutral-800 text-sm text-blue-300 border border-blue-500 p-2 rounded-md text-center">
                            <div class="flex items-center justify-center gap-2">
                                <Icon icon=icondata::BsInfoCircle />
                                Unverified users withdraw only 50 SATS / day.
                            </div>
                        </div>

                        <HighlightedButton
                            alt_style=false
                            disabled=false
                            on_click=move || {
                                KycState::toggle();
                            }
                        >
                            "Start Verification"
                        </HighlightedButton>
                    </div>
                </div>
            </div>
    }
}

#[component]
fn VerificationStatusPopup(show: RwSignal<bool>, is_verified: RwSignal<bool>) -> impl IntoView {
    let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
        move |_: ()| {
            if !is_verified.get() {
                KycState::toggle();
            }
        },
        3000.0,
    );

    Effect::new(move || {
        if !is_verified.get() {
            start(());
        }
    });

    view! {
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div class="overflow-hidden h-fit max-w-md items-center pt-16 cursor-auto bg-[#121212] rounded-md w-full relative">
                    <Show when=move||is_verified.get()>
                        <button
                            on:click=move |_| show.set(false)
                            class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                        >
                            <Icon icon=icondata::ChCross />
                        </button>
                    </Show>
                    <div class="flex z-[2] relative flex-col items-center gap-5 text-white justify-center p-10 pb-8 text-center">
                        // Image
                        <div class="relative">
                            <img
                                class="h-32"
                                src=move || {
                                    if is_verified.get() {
                                        "/img/kyc/kyc_done.svg"
                                    } else {
                                        "/img/kyc/kyc_pending.svg"
                                    }
                                }
                            />
                        </div>

                        // Title
                        <div class="text-xl font-semibold mt-2">
                            {move || if is_verified.get() {
                                view! { <span>"✅ Verified!"</span> }.into_any()
                            } else {
                                view! { <span>Verification Under Process!</span> }.into_any()
                            }}
                        </div>

                        // Body content
                        <div class="text-sm text-neutral-300 flex flex-col gap-2 leading-snug">
                            {move || if is_verified.get() {
                                view! {
                                    <div>
                                        <span>"You’re all set. You can now go back to your wallet to withdraw SATS."</span>
                                        <span class="mt-2">"New Withdrawal Limit: Min 50 SATS"</span>
                                        <span>"Max 1000 SATS per day"</span>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div>
                                        <span>"We’re redirecting you to our verification partner to complete verification."</span>
                                        <span>"Please don’t close or refresh this tab."</span>
                                    </div>
                                }.into_any()
                            }}
                        </div>

                        <Show when=move || is_verified.get()>
                            <HighlightedButton
                                alt_style=false
                                disabled=false
                                on_click=move || {
                                    show.set(false);
                                }
                            >
                                Withdraw to BTC
                            </HighlightedButton>
                        </Show>

                    </div>
                </div>
            </div>
    }
}
