use component::{airdrop_logo::AirdropLogo, social::*, title::TitleText};
use leptos::prelude::*;

#[component]
pub fn Airdrop() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center pb-12 w-screen h-screen text-white bg-black">
            <TitleText>
                <div class="pt-4 pb-8 font-bold text-md">Airdrop</div>
            </TitleText>
            <div class="px-16 pb-8 max-w-80 sm:max-h-80!">
                <AirdropLogo />
            </div>
            <div class="flex flex-col gap-4 items-center py-4 px-16 w-full max-w-md">
                <div class="text-2xl font-bold text-center uppercase">
                    Airdrop Registration has Ended
                </div>
                <div class="text-lg text-center">
                    Thank you for your interest! We are no longer accepting new
                    registrations. If you have already claimed the airdrop, please
                    login to see your status.
                </div>
                <button class="py-2 px-4 w-full text-xl text-white rounded-full bg-primary-600">
                    Login
                </button>
                <div class="flex flex-row gap-4 pt-4">
                    <Telegram />
                    <Discord />
                    <Twitter />
                </div>

            </div>
            <span class="text-md text-white/50">
                For more queries, you can get in touch with us on our socials
            </span>
        </div>
    }
}
