use leptos::prelude::*;
use leptos::web_sys::MouseEvent;

use crate::connect::ConnectLogin;

#[component]
pub fn FeedPopUp<F: Fn(MouseEvent) + 'static>(
    on_dismiss: F,
    header_text: &'static str,
    body_text: &'static str,
    login_text: &'static str,
) -> impl IntoView {
    view! {
        <div
            class="h-full w-full absolute bg-black opacity-90 z-50 flex flex-col justify-center"
            on:click=on_dismiss
        >
            <div class="flex flex-row justify-center" on:click=move |e| e.stop_propagation()>
                <div class="flex flex-col justify-center w-9/12 sm:w-4/12 relative">
                    <img
                        class="h-28 w-28 absolute -left-4 -top-10"
                        src="/img/common/coins/coin-topleft.svg"
                    />
                    <img
                        class="h-18 w-18 absolute -right-2 -top-14"
                        src="/img/common/coins/coin-topright.svg"
                    />
                    <img
                        class="h-18 w-18 absolute -bottom-14 -left-8"
                        src="/img/common/coins/coin-bottomleft.svg"
                    />
                    <img
                        class="h-18 w-18 absolute -bottom-12 -right-2"
                        src="/img/common/coins/coin-bottomright.svg"
                    />
                    <span class="text-white text-3xl text-center text-bold p-2 whitespace-pre-line">
                        {header_text}
                    </span>
                    <span class="text-white text-center p-2 pb-4">{body_text}</span>
                    <div class="flex justify-center">
                        <div class="w-7/12 sm:w-4/12 z-[60]">
                            <ConnectLogin login_text=login_text cta_location="feed_popup" />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
