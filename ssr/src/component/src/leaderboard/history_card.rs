use super::history_types::TournamentHistoryEntry;
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

// Helper function to format timestamps to readable dates
fn format_date(timestamp: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
    datetime.format("%b %d").to_string()
}

#[component]
pub fn TournamentHistoryCard(tournament: TournamentHistoryEntry) -> impl IntoView {
    let navigate = use_navigate();

    // Format date range
    let start_date = format_date(tournament.start_time);
    // let end_date = format_date(tournament.end_time);
    // let date_range = format!("{} – {}", start_date, end_date);

    // Click handler for View Leaderboard button
    let tournament_id = tournament.id.clone();
    let on_click = move |_| {
        navigate(
            &format!("/leaderboard/tournament/{tournament_id}"),
            Default::default(),
        );
    };

    view! {
        <div
            class="relative rounded-lg p-6 transition-all hover:scale-[1.02]"
            style="background: linear-gradient(135deg, rgba(241, 67, 49, 0.1) 0%, rgba(226, 1, 123, 0.1) 50%, rgba(105, 0, 57, 0.1) 100%); border: 1px solid rgba(255, 255, 255, 0.1);">
            // Content wrapper
            <div class="flex flex-col gap-3">
                // First line - Prize pool (full width)
                <div class="flex items-center gap-2">
                    <span class="text-white text-lg font-semibold">
                        "Upto"
                    </span>
                    <span class="text-[#FFEF00] text-lg font-semibold">
                        {tournament.prize_pool.to_string()}
                    </span>
                    // YRAL token icon
                    <img src="/img/yral/yral-token.webp" alt="" class="w-5 h-5" />
                    <span class="text-white/80 text-sm">
                        "Shared by top 10 winners"
                    </span>
                </div>

                // Second section - Content and Trophy side by side
                <div class="flex items-start justify-between">
                    // Left side - Date, winner, button
                    <div class="flex flex-col gap-3 flex-1">
                        // Date range
                        <div class="text-white/60 text-sm font-medium">
                            {start_date}
                        </div>

                        // Winner info
                        <div class="text-white/50 text-xs">
                            <span class="font-semibold">"Winner:"</span>
                            <span class="text-[#E2017B] ml-1">"@"{tournament.winner.username}</span>
                            <span class="mx-2">"•"</span>
                            <span>{tournament.total_participants}" participants"</span>
                        </div>

                        // View Leaderboard button
                        <button
                            class="bg-gradient-to-r from-[#F14331] to-[#E2017B] text-white px-6 py-2 rounded-lg text-sm font-semibold hover:opacity-90 transition-opacity w-fit mt-2"
                            on:click=on_click
                        >
                            "View Leaderboard"
                        </button>
                    </div>

                    // Right side - Trophy illustration
                    <div
                        class="relative w-24 h-24 flex items-center justify-center"
                    >
                        <img
                            src="/img/leaderboard/trophy.svg"
                            alt="Trophy"
                            class="w-16 h-16 object-contain drop-shadow-xl"
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}
