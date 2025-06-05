use component::coming_soon::ComingSoonGraphic;
use leptos::prelude::*;
use leptos_icons::*;

#[component]
pub fn Leaderboard() -> impl IntoView {
    view! {
        <div class="flex flex-col bg-black items-center gap-4 justify-center w-dvw h-dvh">
            <Icon attr:class="w-36 h-36" icon=ComingSoonGraphic />
            <span class="text-white text-xl">Coming Soon</span>
        </div>
    }
}
