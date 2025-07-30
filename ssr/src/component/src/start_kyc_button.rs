use leptos::prelude::*;
use leptos_icons::*;

#[component]
pub fn StartVerificationButton(show_popup: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| show_popup.set(true)
            class="flex w-full items-center justify-between p-2 rounded-xl border border-yellow-800 bg-neutral-900 text-left hover:bg-yellow-900/10 transition-all"
        >
            <div class="flex flex-col gap-1">
                <div class="flex items-center gap-2 font-semibold">
                    <img src="/img/kyc/kyc-shield.svg" />
                    <span>Start verification</span>
                </div>
                <div class="text-sm text-neutral-400">
                    Unlock higher limits & faster withdrawals
                </div>
            </div>

            <div class="text-yellow-800">
                <Icon icon=icondata::AiRightOutlined />
            </div>
        </button>
    }
}
