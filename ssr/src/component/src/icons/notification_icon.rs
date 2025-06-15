use leptos::prelude::*;

#[component]
pub fn NotificationIcon(
    #[prop(into, optional, default = "w-full h-full".into())] class: String,
    #[prop(optional)] show_dot: bool,
) -> impl IntoView {
    view! {
        <svg
            class=format!("text-neutral-200 {}", class)
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M20.5 17H4L5.15542 14.6892C5.71084 13.5783 6 12.3534 6 11.1115V8C6.16667 6.33333 7.6 3 12 3C16.4 3 17.8333 6.33333 18 8V10.7056C18 12.205 18.4214 13.6742 19.216 14.9456L20.5 17Z"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linejoin="round"
            />
            <path
                d="M12 21C13.6569 21 15 20.1046 15 19H9C9 20.1046 10.3431 21 12 21Z"
                fill="currentColor"
            />
            <circle
                cx="17"
                cy="5"
                r="3"
                fill="#E2017B"
                opacity=move || (if show_dot { "1" } else { "0" }).to_string()
            />
        </svg>
    }
}
