use crate::token::context::IcpumpSunsetPopupCtx;
use component::popup::Popup;
use leptos::prelude::*;

#[component]
pub fn IcpumpSunsetPopup() -> impl IntoView {
    let icpump_sunset_popup_ctx = use_context::<IcpumpSunsetPopupCtx>().unwrap();

    view! {
        <Popup show={icpump_sunset_popup_ctx.show}>
            <div class="flex flex-col gap-4 text-center">
                <h1 style="font-size: 3.5rem" class="font-bold pt-24">"👋"</h1>
                <div class="text-3xl font-semibold text-neutral-100">
                    "ICPump.fun is being sunset soon."
                </div>
                <div class="text-neutral-200 text-xl">
                    "All created tokens will be dissolved. In case of any concerns, "<br/>"please reach out to us on " <a href="https://t.me/HotOrNot_app" target="_blank" class="underline">Telegram</a>
                </div>
            </div>
        </Popup>
    }
}
