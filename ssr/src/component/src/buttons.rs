use leptos::prelude::*;

#[component]
pub fn HighlightedButton(
    children: Children,
    on_click: impl Fn() + 'static,
    #[prop(optional)] classes: String,
    #[prop(optional)] alt_style: bool,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let on_click = move |_| on_click();
    view! {
        <button
            on:click=on_click
            disabled=disabled
            class=format!(
                "w-full px-5 py-3 rounded-lg flex items-center transition-all justify-center gap-8 font-kumbh font-bold hover:opacity-90 {}",
                classes,
            )
            style=if alt_style {
                "background: linear-gradient(73deg, #FFF 0%, #FFF 1000%)"
            } else {
                "background: linear-gradient(190.27deg, #FF6DC4 8%, #F7007C 38.79%, #690039 78.48%);"
            }
        >
            <div class=move || {
                if alt_style {
                    "bg-linear-to-r from-[#FF78C1] via-[#E2017B] to-[#5F0938] inline-block text-transparent bg-clip-text"
                } else {
                    "text-white"
                }
            }>{children()}</div>
        </button>
    }
}

#[component]
pub fn HighlightedLinkButton(
    children: Children,
    href: String,
    #[prop(optional)] classes: String,
    #[prop(optional)] alt_style: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] target: String,
) -> impl IntoView {
    view! {
        <a
            href=href
            target=target
            aria_disabled=disabled
            class=format!(
                "w-full px-5 py-3 rounded-lg flex items-center transition-all justify-center gap-8 font-kumbh font-bold hover:opacity-90 {}",
                classes,
            )
            style=if alt_style {
                "background: linear-gradient(73deg, #FFF 0%, #FFF 1000%)"
            } else {
                "background: linear-gradient(190.27deg, #FF6DC4 8%, #F7007C 38.79%, #690039 78.48%);"
            }
        >
            <div class=move || {
                if alt_style {
                    "bg-linear-to-r from-[#FF78C1] via-[#E2017B] to-[#5F0938] inline-block text-transparent bg-clip-text"
                } else {
                    "text-white"
                }
            }>{children()}</div>

        </a>
    }
}

#[component]
pub fn SecondaryHighlightedLinkButton(
    children: Children,
    href: String,
    #[prop(optional)] classes: String,
    #[prop(optional)] alt_style: Signal<bool>,
) -> impl IntoView {
    view! {
        <a
            href=href
            class=move || {
                format!(
                    "rounded-full border border-white text-sm font-bold font-kumbh px-5 py-2 {} {}",
                    if alt_style.get() {
                        "bg-transparent text-white hover:bg-white/10 active:bg-white/5"
                    } else {
                        "bg-white text-black"
                    },
                    classes,
                )
            }
        >
            {children()}
        </a>
    }
}

#[component]
pub fn SecondaryHighlightedButton(
    children: Children,
    disabled: Signal<bool>,
    alt_style: Signal<bool>,
    classes: String,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let on_click = move |_| on_click();
    view! {
        <button
            disabled=move || disabled.get()
            on:click=on_click
            class=move || {
                format!(
                    "rounded-full border border-white text-sm font-bold font-kumbh px-5 py-2 {} {}",
                    if alt_style.get() {
                        "bg-transparent text-white hover:bg-white/10 active:bg-white/5"
                    } else {
                        "bg-white text-black"
                    },
                    classes,
                )
            }
        >

            {children()}
        </button>
    }
}

#[component]
pub fn GradientButton(
    children: Children,
    disabled: Signal<bool>,
    #[prop(into)] classes: String,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let on_click = move |_| on_click();

    view! {
        <button
            class=(
                [
                    "pointer-events-none",
                    "text-primary-300",
                    "bg-brand-gradient-disabled",
                    "cursor-disabled",
                ],
                move || disabled(),
            )
            class=(["text-neutral-50", "bg-brand-gradient"], move || !disabled())
            class=format!("rounded-lg px-5 py-2 text-sm text-center font-bold {}", classes)
            on:click=on_click
        >
            {children()}
        </button>
    }
}

#[component]
pub fn GradientLinkButton(
    children: Children,
    #[prop(into)] href: String,
    #[prop(optional, into)] classes: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    view! {
        <a
            class=(
                [
                    "pointer-events-none",
                    "text-primary-300",
                    "bg-brand-gradient-disabled",
                    "cursor-disabled",
                ],
                disabled,
            )
            class=(["text-neutral-50", "bg-brand-gradient"], !disabled)
            class=format!("rounded-lg px-5 py-2 text-sm text-center font-bold {}", classes)
            href=href
        >
            {children()}
        </a>
    }
}

#[component]
pub fn GradientLinkText(
    children: Children,
    #[prop(into)] href: String,
    #[prop(optional, into)] classes: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    view! {
        <a
            class=(
                ["pointer-events-none", "bg-brand-gradient-disabled", "cursor-disabled"],
                disabled,
            )
            class=(["bg-brand-gradient"], !disabled)
            class=format!(
                "bg-clip-text text-transparent rounded-lg px-5 py-2 text-sm text-center font-bold {}",
                classes,
            )
            href=href
        >
            {children()}
        </a>
    }
}
