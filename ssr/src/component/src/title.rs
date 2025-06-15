use leptos::prelude::*;

#[component]
pub fn TitleText(
    /// `children` takes the `Children` type
    /// this is an alias for `Box<dyn FnOnce() -> Fragment>`
    #[prop(default = true)]
    justify_center: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class="sticky top-0 z-50 items-center p-4 w-full text-white bg-transparent"
            class:justify-center=justify_center
            class:flex=justify_center
        >
            {children()}
        </span>
    }
}
