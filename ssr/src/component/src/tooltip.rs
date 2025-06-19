use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_icons::*;
/// a dumb tooltip. Can't specify direction, customize content, make it stick with a close button, etc.
#[component]
pub fn Tooltip(
    #[prop(into)] icon: icondata_core::Icon,
    #[prop(into)] title: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    let _ = title;
    view! {
        <div class="relative group">
            <div class="grid place-items-center rounded-full cursor-pointer tooltip-target bg-neutral-800 size-[22px]">
                <Icon attr:class="size-[22px]" icon=icon />
            </div>
            <div class="absolute top-0 left-1/2 z-50 p-4 mt-8 ml-2 w-max rounded-md opacity-0 duration-150 -translate-x-1/2 pointer-events-none group-hover:opacity-100 max-w-[65vw] bg-primary-200 text-primary-800 md:max-w-[400px]">
                <div class="absolute right-1/2 bottom-full mr-1 w-0 h-0 border-r-4 border-b-4 border-l-4 border-l-transparent border-r-transparent border-b-primary-200"></div>
                <h2 class="font-bold">{title}</h2>
                <div>{description}</div>
            </div>
        </div>
    }
}
