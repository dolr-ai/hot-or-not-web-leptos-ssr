use component::leaderboard::{
    history_api::fetch_tournament_history, history_card::TournamentHistoryCard,
};
use component::title::TitleText;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LeaderboardHistory() -> impl IntoView {
    let navigate = use_navigate();

    // Fetch tournament history
    let history_resource =
        LocalResource::new(move || async move { fetch_tournament_history(20).await });

    view! {
        <div class="min-h-screen bg-black text-white">
            // Header
            <TitleText>
                <div class="flex items-center justify-between w-full px-4">
                    <button
                        on:click={let navigate = navigate.clone(); move |_| navigate("/leaderboard", Default::default())}
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                        </svg>
                    </button>
                    <span class="text-xl font-bold">History</span>
                    <div class="w-10"></div> // Spacer for centering
                </div>
            </TitleText>

            // Main content
            <div class="container mx-auto px-4 pt-6 pb-24 max-w-4xl">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center py-12">
                        <div class="animate-spin h-8 w-8 border-t-2 border-pink-500 rounded-full"></div>
                    </div>
                }>
                    {move || {
                        match history_resource.get() {
                            Some(Ok(response)) => {
                                if response.tournaments.is_empty() {
                                    view! {
                                        <div class="text-center py-12 text-gray-400">
                                            "No tournament history available"
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-4">
                                            <For
                                                each=move || response.tournaments.clone()
                                                key=|tournament| tournament.id.clone()
                                                children=move |tournament| {
                                                    view! {
                                                        <TournamentHistoryCard tournament=tournament />
                                                    }
                                                }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            }
                            Some(Err(e)) => {
                                view! {
                                    <div class="text-center py-12 text-red-400">
                                        "Failed to load tournament history: " {e}
                                    </div>
                                }.into_any()
                            }
                            None => {
                                view! {
                                    <div class="flex justify-center py-12">
                                        <div class="animate-spin h-8 w-8 border-t-2 border-pink-500 rounded-full"></div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
