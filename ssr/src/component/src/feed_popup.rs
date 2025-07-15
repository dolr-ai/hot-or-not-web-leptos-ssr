use leptos::prelude::*;
use leptos::web_sys::MouseEvent;

use crate::connect::ConnectLogin;

#[component]
pub fn FeedPopUp<F: Fn(MouseEvent) + 'static>(
    on_dismiss: F,
    header_text: String,
    body_text: String,
    login_text: &'static str,
) -> impl IntoView {
    view! {
        <div
            class="flex absolute z-50 flex-col justify-center w-full h-full bg-black opacity-90"
            on:click=on_dismiss
        >
            <div class="flex flex-row justify-center" on:click=move |e| e.stop_propagation()>
                <div class="flex relative flex-col justify-center w-9/12 sm:w-4/12">
                    <img
                        class="absolute -left-4 -top-10 w-28 h-28"
                        src="/img/common/coins/coin-topleft.svg"
                    />
                    <img
                        class="absolute -right-2 -top-14 h-18 w-18"
                        src="/img/common/coins/coin-topright.svg"
                    />
                    <img
                        class="absolute -left-8 -bottom-14 h-18 w-18"
                        src="/img/common/coins/coin-bottomleft.svg"
                    />
                    <img
                        class="absolute -right-2 -bottom-12 h-18 w-18"
                        src="/img/common/coins/coin-bottomright.svg"
                    />
                    <span class="p-2 text-3xl text-center text-white whitespace-pre-line text-bold">
                        {header_text}
                    </span>
                    <span class="p-2 pb-4 text-center text-white">{body_text}</span>
                    <div class="flex justify-center">
                        <div class="w-7/12 sm:w-4/12 z-60">
                            <ConnectLogin login_text=login_text cta_location="feed_popup" />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
