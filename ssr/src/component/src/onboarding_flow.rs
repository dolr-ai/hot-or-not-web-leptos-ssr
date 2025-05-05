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
            class="h-full w-full bg-black bg-opacity-70 z-10 flex flex-col justify-center relative"
            style=style
        >
            <Show when=move || { onboarding_page_no.get() == 1 }>
                <OnboardingTopDecorator />
                <div class="flex flex-row justify-center">
                    <div class="flex flex-col justify-center w-9/12 sm:w-4/12 relative gap-y-36">
                        <div class="relative self-center">
                            <p class="text-white text-center font-bold w-56 text-2xl leading-normal">
                                A new Hot or Not game experience awaits you
                            </p>
                            <img
                                class="-left-6 top-8 h-5 w-5 absolute"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="-left-2 -top-6 h-4 w-4 absolute"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="left-6 -top-2 h-3 w-3 absolute"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="-right-6 -top-2 h-6 w-6 absolute"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="right-2 -top-1 h-2 w-2 absolute"
                                src="/img/common/decorator/star.svg"
                            />
                            <img
                                class="-right-5 bottom-4 h-2 w-2 absolute"
                                src="/img/common/decorator/star.svg"
                            />
                        </div>
                        <div class="flex flex-col items-center gap-y-4">
                            <button
                                class="self-center font-semibold rounded-full bg-primary-600 py-2 md:py-3 w-full max-w-80 text-center text-base md:text-xl text-white"
                                on:click=move |_| onboarding_page_no.set(2)
                            >
                                Start Tutorial
                            </button>
                            <button
                                class="text-white text-center font-medium text-base leading-normal font-sans"
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
                    <div class="flex flex-col justify-center w-9/12 sm:w-4/12 relative">
                        <div class="self-center">
                            <p class="text-white text-center font-bold w-56 text-2xl leading-normal">
                                "There's even more"
                            </p>
                        </div>
                        <div class="flex flex-col justify-center gap-y-3 mt-12">
                            <div class="self-center">
                                <img src="/img/common/decorator/buy_coin.svg" />
                            </div>
                            <div class="self-center">
                                <p class="text-white text-center font-medium text-sm leading-normal">
                                    Refer and Earn Cents
                                </p>
                            </div>
                        </div>
                        <div class="flex flex-col justify-center gap-y-3 mt-12">
                            <div class="self-center">
                                <img src="/img/common/decorator/prizes.svg" />
                            </div>
                            <div class="self-center">
                                <p class="text-white text-center font-medium text-sm leading-normal">
                                    Play and earn
                                </p>
                            </div>
                        </div>
                        <button
                            class="font-bold rounded-full bg-primary-600 py-3 md:py-4 w-80 mt-24 self-center text-center text-lg md:text-xl text-white"
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
        <div class="top-0 w-full flex justify-center">
            <div class="absolute left-0 top-0">
                <img src="/img/common/decorator/decore-left.svg" />
            </div>
            <div class="absolute right-0 top-0">
                <img src="/img/common/decorator/decore-right.svg" />
            </div>
        </div>
    }
}

#[component]
pub fn OnboardingTopCross(onboard_on_click: WriteSignal<bool>) -> impl IntoView {
    view! {
        <div class="top-0 w-full flex justify-center">
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
            <div class="flex flex-col justify-center w-9/12 sm:w-4/12 relative">
                <div class="relative flex flex-col justify-center items-center gap-y-9">
                    <div class="flex flex-col gap-y-2 justify-center items-center">
                        <div class="self-center">
                            <p class="text-white text-center font-bold w-72 text-2xl leading-normal -mt-3">
                                {header_text}
                            </p>
                        </div>
                        <div class="self-center px-2">
                            <p class="text-white text-center font-medium w-64 text-sm leading-5 font-sans">
                                {body_text}
                            </p>
                        </div>
                    </div>
                    <div class="flex flex-col gap-y-4 justify-center items-center">
                        <button
                            class="self-center font-semibold rounded-full bg-primary-600 z-20 py-2 md:py-3 w-40 max-w-30 text-center text-base md:text-xl text-white"
                            on:click=move |_| onboarding_page_no.update(|page| *page += 1)
                        >
                            Next
                        </button>
                        <button
                            class="text-white text-center font-semibold text-lg sm:text-base z-20 leading-normal font-sans"
                            on:click=move |_| onboarding_page_no.update(|page| *page -= 1)
                        >
                            Previous
                        </button>
                    </div>
                    <Show when=move || { onboarding_page_no.get() == 2 }>
                        <img
                            src="/img/common/decorator/coin_arrow.svg"
                            class="absolute h-[30vh] hot-left-arrow -ml-56 sm:-ml-64 mt-48 sm:mt:64"
                        />
                    </Show>
                    <Show when=move || { onboarding_page_no.get() == 3 }>
                        <img
                            src="/img/common/decorator/hot_arrow.svg"
                            class="absolute h-[33vh] hot-left-arrow -ml-60 sm:-ml-72 mt-48 sm:mt-64"
                        />
                    </Show>
                    <Show when=move || { onboarding_page_no.get() == 4 }>
                        <img
                            src="/img/common/decorator/not_arrow.svg"
                            class="absolute h-[33vh] hot-left-arrow ml-60 sm:ml-72 mt-48 sm:mt-64"
                        />
                    </Show>
                </div>
            </div>
        </div>
    }.into_any()
}
