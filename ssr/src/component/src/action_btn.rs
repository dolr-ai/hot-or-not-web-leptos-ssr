use leptos::prelude::*;

#[component]
pub fn ActionButtonLink(
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
            <div class="flex justify-center items-center w-4.5 h-4.5">{children()}</div>

            <div class="font-medium leading-4 text-[0.625rem]">{label}</div>
        </a>
    }
}

#[component]
pub fn ActionButton(
    label: String,
    children: Children,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <button
            disabled=disabled
            class="flex flex-col gap-1 justify-center items-center text-xs transition-colors enabled:group-hover:text-white enabled:text-neutral-300 disabled:cursor-default cursor-pointer disabled:text-neutral-600"
        >
            <div class="flex justify-center items-center w-4.5 h-4.5">{children()}</div>

            <div>{label}</div>
        </button>
    }
}
