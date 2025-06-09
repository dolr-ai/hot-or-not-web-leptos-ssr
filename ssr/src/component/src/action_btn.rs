use leptos::prelude::*;

#[component]
pub fn ActionButton(
    href: String,
    label: String,
    children: Children,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <a
            aria-disabled=move || disabled().to_string()
            href=href
            class=move || {
                format!(
                    "flex flex-col gap-1 justify-center items-center text-xs transition-colors {}",
                    if !disabled.get() {
                        "group-hover:text-white text-neutral-300"
                    } else {
                        "text-neutral-600 pointer-events-none"
                    },
                )
            }
        >
            <div class="w-4.5 h-4.5 flex items-center justify-center">
                {children()}
            </div>

            <div class="text-[0.625rem] font-medium leading-4">{label}</div>
        </a>
    }
}

#[component]
pub fn ActionButtonLink(
    label: String,
    children: Children,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <button
            disabled=disabled
            class="flex flex-col gap-1 justify-center items-center text-xs transition-colors enabled:group-hover:text-white enabled:text-neutral-300 disabled:group-hover:cursor-default disabled:text-neutral-600"
        >
            <div class="w-4.5 h-4.5 flex items-center justify-center">
                {children()}
            </div>

            <div>{label}</div>
        </button>
    }
}
