use leptos::prelude::*;

pub mod universal_modal;

pub use universal_modal::{icons, ButtonConfig, ButtonStyle, UniversalModal};

#[component]
pub fn ModalOverlay(#[prop(into)] show: Signal<bool>, children: Children) -> impl IntoView {
    view! {
        <div
            class=move || format!(
                "fixed inset-0 z-50 flex items-center justify-center transition-all duration-300 {}",
                if show.get() {
                    "opacity-100 visible bg-black/50 backdrop-blur-sm"
                } else {
                    "opacity-0 invisible"
                }
            )
        >
            <div
                class=move || format!(
                    "relative transform transition-all duration-300 {}",
                    if show.get() {
                        "scale-100 opacity-100"
                    } else {
                        "scale-95 opacity-0"
                    }
                )
            >
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn CloseButton(on_close: impl Fn() + 'static + Copy + Send + Sync) -> impl IntoView {
    view! {
        <button
            class="absolute top-4 right-4 w-8 h-8 flex items-center justify-center rounded-full bg-gray-600/50 hover:bg-gray-600/70 transition-colors z-10"
            on:click=move |_| on_close()
        >
            <svg
                class="w-5 h-5 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
            >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
        </button>
    }
}
