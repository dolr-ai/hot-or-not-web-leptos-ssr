use crate::token::context::IcpumpSunsetPopupCtx;
use component::popup::Popup;
use leptos::prelude::*;

#[component]
pub fn IcpumpSunsetPopup() -> impl IntoView {
    let icpump_sunset_popup_ctx = use_context::<IcpumpSunsetPopupCtx>().unwrap();

    view! {
        <Popup show={icpump_sunset_popup_ctx.show}>
            <div class="flex flex-col gap-4 text-center">
                <h1 class="text-4xl font-bold">"👋"</h1>
                <div class="text-xl">
                "icpump.fun is being sunset soon."
                </div>
                <div class="text-neutral-500">
                "All created tokens will be dissolved in case of any concerns please reach out to us on" <a href="https://t.me/HotOrNot_app" target="_blank" class="underline">Telegram</a>
                </div>
            </div>
        </Popup>
    }
}
