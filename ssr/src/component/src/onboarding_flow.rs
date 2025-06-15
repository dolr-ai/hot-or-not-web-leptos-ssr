use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn OnboardingPopUp(onboard_on_click: WriteSignal<bool>) -> impl IntoView {
    let onboarding_page_no = RwSignal::new(1);

    let style = move || {
        if onboarding_page_no.get() == 2 {
            "background: radial-gradient(circle at 50% calc(100% - 148px), transparent 40px, rgba(0, 0, 0, 0.7) 37px);"
        } else if onboarding_page_no.get() == 3 {
            "background: radial-gradient(circle at calc(50% - 90px) calc(100% - 130px), transparent 56px, rgba(0, 0, 0, 0.7) 51px);"
        } else if onboarding_page_no.get() == 4 {
            "background: radial-gradient(circle at calc(50% + 90px) calc(100% - 130px), transparent 56px, rgba(0, 0, 0, 0.7) 51px);"
        } else {
            ""
        }
    };

    view! {
        <div
            class="flex relative z-10 flex-col justify-center w-full h-full bg-black bg-opacity-70"
            style=style
        >
            <Show when=move || { onboarding_page_no.get() == 1 }>
                <OnboardingTopDecorator />
                <div class="flex flex-row justify-center">
                    <div class="flex relative flex-col gap-y-36 justify-center w-9/12 sm:w-4/12">
                        <div class="relative self-center">
                            <p class="w-56 text-2xl font-bold leading-normal text-center text-white">
                                A new Hot or Not game experience awaits you
                            </p>
                            <img
                                class="absolute -left-6 top-8 w-5 h-5"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="absolute -left-2 -top-6 w-4 h-4"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="absolute -top-2 left-6 w-3 h-3"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="absolute -top-2 -right-6 w-6 h-6"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="absolute -top-1 right-2 w-2 h-2"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="absolute bottom-4 -right-5 w-2 h-2"
                                src="/img/common/decorator/star.svg"
                            />
                        </div>
                        <div class="flex flex-col gap-y-4 items-center">
                            <button
                                class="self-center py-2 w-full text-base font-semibold text-center text-white rounded-full md:py-3 md:text-xl bg-primary-600 max-w-80"
                                on:click=move |_| onboarding_page_no.set(2)
                            >
                                Start Tutorial
                            </button>
                            <button
                                class="font-sans text-base font-medium leading-normal text-center text-white"
                                on:click=move |_| onboard_on_click.set(true)
                            >
                                Maybe Later
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || { onboarding_page_no.get() == 2 }>
                <OnboardingTopCross onboard_on_click />
                <OnboardingContent
                    header_text="Select your bet amount"
                    body_text="Select your bet (50, 100, or 200) by tapping the coin or arrows"
                    onboarding_page_no
                />
            </Show>

            <Show when=move || { onboarding_page_no.get() == 3 }>
                <OnboardingTopCross onboard_on_click />
                <OnboardingContent
                    header_text="Place your first bet"
                    body_text="Do you think the video will be popular? Click 'Hot' and place your bet"
                    onboarding_page_no
                />
            </Show>

            <Show when=move || { onboarding_page_no.get() == 4 }>
                <OnboardingTopCross onboard_on_click />
                <OnboardingContent
                    header_text="Place your first bet"
                    body_text="If you think video won't be popular, click 'Not' and place your bet"
                    onboarding_page_no
                />
            </Show>

            <Show when=move || { onboarding_page_no.get() == 5 }>
                <OnboardingTopDecorator />
                <div class="flex flex-row justify-center">
                    <div class="flex relative flex-col justify-center w-9/12 sm:w-4/12">
                        <div class="self-center">
                            <p class="w-56 text-2xl font-bold leading-normal text-center text-white">
                                "There's even more"
                            </p>
                        </div>
                        <div class="flex flex-col gap-y-3 justify-center mt-12">
                            <div class="self-center">
                                <img src="/img/common/decorator/buy_coin.svg" />
                            </div>
                            <div class="self-center">
                                <p class="text-sm font-medium leading-normal text-center text-white">
                                    Refer and Earn Cents
                                </p>
                            </div>
                        </div>
                        <div class="flex flex-col gap-y-3 justify-center mt-12">
                            <div class="self-center">
                                <img src="/img/common/decorator/prizes.svg" />
                            </div>
                            <div class="self-center">
                                <p class="text-sm font-medium leading-normal text-center text-white">
                                    Play and earn
                                </p>
                            </div>
                        </div>
                        <button
                            class="self-center py-3 mt-24 w-80 text-lg font-bold text-center text-white rounded-full md:py-4 md:text-xl bg-primary-600"
                            on:click=move |_| onboard_on_click.set(true)
                        >
                            "Let's make some money"
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }.into_any()
}

#[component]
pub fn OnboardingTopDecorator() -> impl IntoView {
    view! {
        <div class="flex top-0 justify-center w-full">
            <div class="absolute top-0 left-0">
                <img src="/img/common/decorator/decore-left.svg" />
            </div>
            <div class="absolute top-0 right-0">
                <img src="/img/common/decorator/decore-right.svg" />
            </div>
        </div>
    }
}

#[component]
pub fn OnboardingTopCross(onboard_on_click: WriteSignal<bool>) -> impl IntoView {
    view! {
        <div class="flex top-0 justify-center w-full">
            <div class="absolute right-[16.1px] top-[19px]">
                <button
                    class="text-white bg-transparent bg-opacity-70"
                    on:click=move |_| onboard_on_click.set(true)
                >
                    <Icon attr:class="w-[24px] h-[24px]" icon=icondata::ChCross />
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn OnboardingContent(
    header_text: &'static str,
    body_text: &'static str,
    onboarding_page_no: RwSignal<i32>,
) -> impl IntoView {
    view! {
        <div class="flex flex-row justify-center">
            <div class="flex relative flex-col justify-center w-9/12 sm:w-4/12">
                <div class="flex relative flex-col gap-y-9 justify-center items-center">
                    <div class="flex flex-col gap-y-2 justify-center items-center">
                        <div class="self-center">
                            <p class="-mt-3 w-72 text-2xl font-bold leading-normal text-center text-white">
                                {header_text}
                            </p>
                        </div>
                        <div class="self-center px-2">
                            <p class="w-64 font-sans text-sm font-medium leading-5 text-center text-white">
                                {body_text}
                            </p>
                        </div>
                    </div>
                    <div class="flex flex-col gap-y-4 justify-center items-center">
                        <button
                            class="z-20 self-center py-2 w-40 text-base font-semibold text-center text-white rounded-full md:py-3 md:text-xl bg-primary-600 max-w-30"
                            on:click=move |_| onboarding_page_no.update(|page| *page += 1)
                        >
                            Next
                        </button>
                        <button
                            class="z-20 font-sans text-lg font-semibold leading-normal text-center text-white sm:text-base"
                            on:click=move |_| onboarding_page_no.update(|page| *page -= 1)
                        >
                            Previous
                        </button>
                    </div>
                    <Show when=move || { onboarding_page_no.get() == 2 }>
                        <img
                            src="/img/common/decorator/coin_arrow.svg"
                            class="absolute mt-48 -ml-56 sm:-ml-64 h-[30vh] hot-left-arrow sm:mt:64"
                        />
                    </Show>
                    <Show when=move || { onboarding_page_no.get() == 3 }>
                        <img
                            src="/img/common/decorator/hot_arrow.svg"
                            class="absolute mt-48 -ml-60 sm:mt-64 sm:-ml-72 h-[33vh] hot-left-arrow"
                        />
                    </Show>
                    <Show when=move || { onboarding_page_no.get() == 4 }>
                        <img
                            src="/img/common/decorator/not_arrow.svg"
                            class="absolute mt-48 ml-60 sm:mt-64 sm:ml-72 h-[33vh] hot-left-arrow"
                        />
                    </Show>
                </div>
            </div>
        </div>
    }.into_any()
}
