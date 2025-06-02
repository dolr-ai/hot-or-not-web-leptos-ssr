use component::{airdrop_logo::AirdropLogo, social::*, title::TitleText};
use leptos::prelude::*;

#[component]
pub fn Airdrop() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center h-screen w-screen pb-12 text-white bg-black">
            <TitleText>
                <div class="pt-4 pb-8 text-md font-bold">Airdrop</div>
            </TitleText>
            <div class="max-w-80 px-16 sm:max-h-80! pb-8">
                <AirdropLogo />
            </div>
            <div class="flex flex-col w-full max-w-md px-16 py-4 gap-4 items-center">
                <div class="text-center text-2xl font-bold uppercase">
                    Airdrop Registration has Ended
                </div>
                <div class="text-center text-lg">
                    Thank you for your interest! We are no longer accepting new
                    registrations. If you have already claimed the airdrop, please
                    login to see your status.
                </div>
                <button class="rounded-full w-full bg-primary-600 text-white text-xl px-4 py-2">
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
