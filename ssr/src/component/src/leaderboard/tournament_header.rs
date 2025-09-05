use super::types::TournamentInfo;
use chrono::Utc;
use leptos::prelude::*;
use leptos_use::{use_interval, UseIntervalReturn};
use utils::timezone::format_tournament_date_with_fallback;

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

#[component]
pub fn TournamentHeader(tournament: TournamentInfo) -> impl IntoView {
    // Calculate time remaining
    let (time_remaining, set_time_remaining) = signal(String::new());

    // Update countdown every second
    let end_time = tournament.end_time;

    // Calculate initial time remaining
    let calculate_time_remaining = move || {
        let now = Utc::now().timestamp();
        let remaining = end_time - now;

        if remaining <= 0 {
            "Tournament Ended".to_string()
        } else {
            let days = remaining / 86400;
            let hours = (remaining % 86400) / 3600;
            let minutes = (remaining % 3600) / 60;
            let seconds = remaining % 60;

            if days > 0 {
                format!("{days}d {hours}h {minutes}m")
            } else if hours > 0 {
                format!("{hours}h {minutes}m {seconds}s")
            } else if minutes > 0 {
                format!("{minutes}m {seconds}s")
            } else {
                format!("{seconds}s")
            }
        }
    };

    // Set initial value
    set_time_remaining.set(calculate_time_remaining());

    // Start interval timer to update every second
    let UseIntervalReturn { counter, .. } = use_interval(1000);

    // Update countdown on each interval tick
    Effect::new(move |_| {
        counter.get(); // Subscribe to counter changes
        set_time_remaining.set(calculate_time_remaining());
    });

    view! {
        <div class="relative w-full rounded-lg overflow-hidden mb-6 min-h-[140px]"
            style="background-image: url('/img/leaderboard/header-bg.svg'); background-size: cover; background-position: center;">
            // Optional gradient overlay for better text readability
            <div class="absolute inset-0 bg-gradient-to-r from-black/20 to-transparent"></div>

            // Content
            <div class="relative p-4 md:p-6 flex items-center justify-between">
                <div class="flex-1">
                    <h2 class="text-xl md:text-2xl font-bold text-white mb-1 flex items-center flex-wrap">
                        <span>"Win upto "</span>
                        <span class="text-[#FFC33A] mx-1">{format_with_commas(tournament.prize_pool as u64)}</span>
                        <img src="/img/yral/yral-token.webp" alt="" class="w-5 h-5 md:w-6 md:h-6 mr-1" />
                        <span>"Today!"</span>
                    </h2>
                    <p class="text-white/80 text-xs md:text-sm">
                        "Top the leaderboard to win!"
                    </p>

                    // Contest countdown badge
                    <div class="mt-4 inline-flex items-center gap-1.5 bg-neutral-900 rounded-full px-2 py-1">
                        <span class="text-neutral-400 text-[10px] font-normal">
                            "Time remaining:"
                        </span>
                        <span class="text-neutral-50 text-[10px] font-medium">{move || time_remaining.get()}</span>
                    </div>
                </div>

                // Gift box graphic with floating coins
                <div class="relative">
                    <img src="/img/leaderboard/gift-box-header.svg" alt="Gift Box" class="w-32 h-32" />
                </div>
            </div>
        </div>
    }
}
