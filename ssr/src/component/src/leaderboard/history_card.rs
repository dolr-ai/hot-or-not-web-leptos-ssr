use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use super::history_types::TournamentHistoryEntry;
use chrono::{DateTime, Utc};

// Helper function to format timestamps to readable dates
fn format_date(timestamp: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| Utc::now().into());
    datetime.format("%b %d").to_string()
}

#[component]
pub fn TournamentHistoryCard(
    tournament: TournamentHistoryEntry,
) -> impl IntoView {
    let navigate = use_navigate();
    
    // Format date range
    let start_date = format_date(tournament.start_time);
    let end_date = format_date(tournament.end_time);
    let date_range = format!("{} – {}", start_date, end_date);
    
    // Click handler for View Leaderboard button
    let tournament_id = tournament.id.clone();
    let on_click = move |_| {
        navigate(&format!("/leaderboard/tournament/{}", tournament_id), Default::default());
    };
    
    view! {
        <div class="relative bg-black border border-[#212121] rounded-lg p-4 hover:bg-white/5 transition-colors">
            // Content wrapper
            <div class="flex items-start justify-between">
                // Left side content
                <div class="flex flex-col gap-2.5 flex-1">
                    // Prize pool row
                    <div class="flex items-center gap-2.5">
                        <div class="flex items-center gap-1.5">
                            <span class="text-[#FFEF00] text-base font-bold">
                                {tournament.prize_pool.to_string()}
                            </span>
                            // Coin emoji
                            <span class="text-yellow-500">{"🪙"}</span>
                        </div>
                        <span class="text-white text-base font-bold">
                            "Shared by top 10 winners"
                        </span>
                    </div>
                    
                    // Date range
                    <div class="text-white/70 text-xs">
                        {date_range}
                    </div>
                    
                    // Winner info (subtle display)
                    <div class="text-white/50 text-xs">
                        "Winner: @"{tournament.winner.username}" • "
                        {tournament.total_participants}" participants"
                    </div>
                    
                    // View Leaderboard button
                    <button 
                        class="bg-[#E2017B] text-white px-5 py-1.5 rounded-lg text-sm font-semibold hover:bg-[#E2017B]/90 transition-colors w-fit"
                        on:click=on_click
                    >
                        "View Leaderboard"
                    </button>
                </div>
                
                // Right side - Trophy illustration
                <div class="w-24 h-20 flex items-center justify-center opacity-50">
                    // Trophy emoji as placeholder (can be replaced with actual image)
                    <span class="text-5xl">{"🏆"}</span>
                </div>
            </div>
        </div>
    }
}