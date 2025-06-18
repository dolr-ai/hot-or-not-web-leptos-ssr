use component::coming_soon::ComingSoonGraphic;
use leptos::prelude::*;
use leptos_icons::*;

#[component]
pub fn Leaderboard() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4 justify-center items-center bg-black w-dvw h-dvh">
            <Icon attr:class="w-36 h-36" icon=ComingSoonGraphic />
            <span class="text-xl text-white">Coming Soon</span>
        </div>
    }
}
