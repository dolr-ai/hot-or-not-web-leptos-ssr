use super::history_types::TournamentHistoryEntry;
use crate::buttons::GradientButton;
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
    let on_click = move || {
        navigate(
            &format!("/leaderboard/tournament/{tournament_id}"),
            Default::default(),
        );
    };

    // Disabled state for button (always false for this use case)
    let disabled = Signal::derive(|| false);

    view! {
        <div
            class="relative rounded-lg p-4 transition-all hover:scale-[1.02]"
            style="background: linear-gradient(135deg, rgba(241, 67, 49, 0.1) 0%, rgba(226, 1, 123, 0.1) 50%, rgba(105, 0, 57, 0.1) 100%); border: 1px solid rgba(255, 255, 255, 0.1);">
            // Content wrapper
            <div class="flex flex-col gap-3">
                // First row - Prize pool line (full width)
                <div class="flex items-center gap-2">
                    <span class={if tournament.prize_token == "CKBTC" { "text-white text-sm font-bold" } else { "text-white text-base font-bold" }}>
                        "Upto"
                    </span>
                    <span class={if tournament.prize_token == "CKBTC" { "text-[#FFEF00] text-sm font-bold" } else { "text-[#FFEF00] text-base font-bold" }}>
                        {if tournament.prize_token == "CKBTC" {
                            format!("${} BITCOIN", tournament.prize_pool as u64)
                        } else {
                            tournament.prize_pool.to_string()
                        }}
                    </span>
                    {if tournament.prize_token != "CKBTC" {
                        view! {
                            <img src="/img/yral/yral-token.webp" alt="" class="w-[18px] h-[18px]" />
                        }.into_any()
                    } else {
                        view! {
                            <img src="/img/hotornot/bitcoin.svg" alt="" class="w-[18px] h-[18px]" />
                        }.into_any()
                    }}
                    <span class={if tournament.prize_token == "CKBTC" { "text-white text-sm font-bold" } else { "text-white text-base font-bold" }}>
                        {format!("Shared by top {} winners", tournament.num_winners.unwrap_or(10))}
                    </span>
                </div>

                // Second row - Content and Trophy side by side
                <div class="flex items-start justify-between gap-4">
                    // Left side - Date, winner, button
                    <div class="flex flex-col gap-2.5 flex-1">
                        // Date and participants line
                        <div class="text-white text-xs">
                            {start_date}
                            <span class="mx-1">"•"</span>
                            {tournament.total_participants}" participants"
                        </div>

                        // Winner line
                        <div class="text-white text-xs">
                            "Winner: @"{tournament.winner.username}
                        </div>

                        // View Leaderboard button
                        <GradientButton
                            on_click=on_click
                            disabled=disabled
                            classes="w-[180px] text-nowrap".to_string()
                        >
                            "View Leaderboard"
                        </GradientButton>
                    </div>

                    // Right side - Trophy illustration
                    <div class="relative flex-shrink-0">
                        <img
                            src="/img/leaderboard/trophy.svg"
                            alt="Trophy"
                            class="w-[88px] h-[88px] object-contain"
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}
