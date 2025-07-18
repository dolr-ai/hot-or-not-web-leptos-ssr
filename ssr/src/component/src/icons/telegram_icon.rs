use leptos::prelude::*;

#[component]
pub fn TelegramIcon(
    #[prop(optional, default = "w-full h-full".to_string())] classes: String,
) -> impl IntoView {
    view! {
        <svg
            class=format!("{}", classes)
            viewBox="0 0 24 25"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path fill-rule="evenodd" clip-rule="evenodd" d="M24 12.5a12 12 0 1 1-24 0 12 12 0 0 1 24 0ZM12.4 9.4A571.4 571.4 0 0 0 4.5 13c0 .4.5.5 1 .7l.3.1a9 9 0 0 0 1.9.5c.4 0 .8-.2 1.3-.5a124 124 0 0 1 5.3-3.3v.2c0 .2-1.8 1.8-2.7 2.7a32.7 32.7 0 0 0-.8.8c-.6.5-1 1 0 1.6a94.9 94.9 0 0 1 2.7 1.8l.4.3c.5.4 1 .7 1.5.6.3 0 .6-.3.8-1.2a142.2 142.2 0 0 0 1.3-9.1l-.1-.3a.8.8 0 0 0-.5-.2c-.4 0-1.1.3-4.5 1.7Z" fill="currentColor"/>
        </svg>
    }
}
