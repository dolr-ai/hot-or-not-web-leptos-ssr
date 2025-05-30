use leptos::{component, either::Either, prelude::*};
use leptos_icons::Icon;

use crate::pumpdump::{
    components::{
        button::{MockDumpButton, MockPumpButton},
        slider::MockBullBearSlider,
    },
    ShowOnboarding,
};

#[component]
pub fn OnboardingPopup() -> impl IntoView {
    let (step, set_step) = signal(0);
    let show_onboarding: ShowOnboarding = expect_context();
    view! {
        <div class="fade-in fixed inset-0 bg-black/50 flex py-16 justify-center z-50 p-4">
            <div
                style="background-size: cover; background-position: left; background-image: url('/img/pumpdump/onboarding-bg.webp');"
                class="rounded-2xl max-w-md flex flex-col h-134 justify-center text-white gap-8 items-center pt-8 pb-5 px-8 absolute top-1/2 -translate-y-1/2 origin-center [@media(max-height:680px)]:scale-[.8]"
            >
                <div
                    class="absolute flex items-center top-4 px-4 inset-x-0"
                    class=("justify-end", move || step.get() == 0)
                    class=("justify-between", move || step.get() == 1)
                >
                {move || (step.get() == 1).then(|| view! {
                    <button on:click=move |_| set_step.set(0) class="text-neutral-400">
                        <Icon attr:class="size-5" icon=icondata::FiChevronLeft />
                    </button>
                })}
                    <button
                        on:click=move |_| show_onboarding.hide()
                        class="p-1 flex items-center justify-center bg-neutral-600 rounded-full"
                    >
                        <Icon attr:class="size-3" icon=icondata::IoClose />
                    </button>
                </div>
                {move || if step.get() == 0 {
                    Either::Left(view! {
                        <img src="/img/pumpdump/logo.webp" alt="Logo" class="h-32 pt-8" />
                        <div class="flex flex-col gap-5 items-center">
                            <div class="font-bold text-xl">Shape the Future of Tokens!</div>
                            <div class="text-base text-center">
                                Your vote decides the fate of the tokens. Ride the waves of Pump and Dump and vote to
                                make the tides shift to snatch up with reward pool.
                            </div>
                            <div class="flex gap-0.5 text-sm">
                                <img src="/img/pumpdump/cents.webp" alt="Coin" class="w-5 h-5" />
                                <div>1 Cent = 1 vote</div>
                            </div>
                        </div>
                        <div class="flex w-full justify-end pt-20 items-center gap-1">
                            <button
                                on:click=move |_| set_step.set(1)
                                class="appearance-none text-xl font-semibold">Next</button
                            >
                            <Icon attr:class="size-6 -mb-0.5" icon=icondata::FiChevronRight />
                        </div>
                    }.into_any())
                } else {
                    Either::Right(view! {
                        <div class="flex flex-col text-sm gap-5 items-center text-center">
                            <div class="font-bold text-xl">How it works?</div>
                            <div class="flex gap-2 justify-between items-center">
                                <div class="flex-1 text-xs text-left">
                                    <div class="text-white">Step 1</div>
                                    <div class="text-neutral-400">
                                        Vote for the Tide - Pump or Dump. Predict the next shift in momentum.
                                    </div>
                                </div>
                                <div class="flex-1 relative py-12">
                                    <div class="scale-[0.6] h-full w-full">
                                        <div class="absolute bottom-0 -left-8 z-2">
                                            <MockPumpButton />
                                        </div>
                                        <div class="absolute -top-2 -right-6 z-1">
                                            <MockDumpButton />
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="flex flex-row-reverse gap-2 justify-between items-center">
                                <div class="flex-1 text-xs text-right">
                                    <div class="text-white">Step 2</div>
                                    <div class="text-neutral-400">
                                        The battle for dominance begins here, keep voting as each vote influences the tide
                                    </div>
                                </div>
                                <div class="flex-1 relative py-6">
                                    <div class="scale-[0.8] -translate-x-3 h-full w-full">
                                        <MockBullBearSlider />
                                    </div>
                                </div>
                            </div>
                            <div class="flex gap-2 justify-between items-center">
                                <div class="flex-1 text-xs text-left">
                                    <div class="text-white">Step 3</div>
                                    <div class="text-neutal-400">
                                        Claim your rewards when the tide turns and overtakes the majority.
                                    </div>
                                </div>
                                <div class="flex-1 flex items-center justify-center relative py-6 pl-8">
                                    <img src="/img/pumpdump/trophy.webp" alt="Trophy" class="h-20 w-22" />
                                </div>
                            </div>
                        </div>
                        <button
                            on:click=move |_| show_onboarding.hide()
                            class="w-full px-5 py-3 rounded-lg flex items-center transition-all justify-center gap-8 font-kumbh font-bold"
                            style:background="linear-gradient(73deg, #DA539C 0%, #E2017B 33%, #5F0938 100%)"
                        >Ok, got it!</button>
                    }.into_any())
                }}
            </div>
        </div>
    }
}
