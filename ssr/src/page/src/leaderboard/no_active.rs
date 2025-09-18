use component::buttons::HighlightedButton;
use component::title::TitleText;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NoActiveTournament() -> impl IntoView {
    let navigate = use_navigate();
    let navigate_back = navigate.clone();
    let navigate_history = navigate.clone();
    let navigate_play = navigate.clone();

    view! {
        <div class="min-h-screen bg-black text-white">
            // Header
            <TitleText>
                <div class="flex items-center justify-between w-full px-4">
                    <button
                        class="p-2"
                        on:click=move |_| navigate_back("/", Default::default())
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                        </svg>
                    </button>
                    <span class="text-xl font-bold">Leaderboard</span>
                    <button
                        class="text-pink-500 text-sm font-medium"
                        on:click=move |_| navigate_history("/leaderboard/history", Default::default())
                    >
                        "View History"
                    </button>
                </div>
            </TitleText>

            // Content
            <div class="flex items-center justify-center px-4 min-h-[calc(100vh-80px)]">
                <div class="max-w-md w-full flex flex-col items-center text-center">
                // Icon
                <div class="mb-8">
                    <img
                        src="/img/leaderboard/no-active.svg"
                        alt="No active tournament"
                        class="w-32 h-32 md:w-40 md:h-40"
                    />
                </div>

                // Heading
                <h1 class="text-2xl md:text-3xl font-bold mb-4 text-white">
                    "No Active Tournament"
                </h1>

                // Description
                <p class="text-gray-400 text-base md:text-lg mb-8 leading-relaxed">
                    "There's no tournament running right now. Check back soon for the next competition and your chance to win rewards!"
                </p>

                // Play Games button with pink gradient
                <div class="w-full max-w-xs">
                    <HighlightedButton
                        on_click=move || navigate_play("/", Default::default())
                        classes="text-lg".to_string()
                    >
                        "Play Games"
                    </HighlightedButton>
                </div>
                </div>
            </div>
        </div>
    }
}
