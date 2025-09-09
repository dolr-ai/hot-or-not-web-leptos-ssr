use super::types::TournamentInfo;
use chrono::Utc;
use leptos::prelude::*;
use leptos_use::{use_interval, UseIntervalReturn};

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
            <div class="relative p-4 pr-40">
                <div class="flex-1">
                    // Prize pool line
                    <div class="flex items-center gap-1.5 mb-1">
                        <span class="text-xl font-bold text-white">"Win upto "</span>
                        <span class="text-xl font-bold text-[#FFEF00]">
                            {if tournament.prize_token == "CKBTC" {
                                format!("${}", format_with_commas(tournament.prize_pool as u64))
                            } else {
                                format_with_commas(tournament.prize_pool as u64)
                            }}
                        </span>
                        {if tournament.prize_token != "CKBTC" {
                            view! {
                                <img src="/img/yral/yral-token.webp" alt="" class="w-6 h-6" />
                            }.into_any()
                        } else {
                            view! { <img src="/img/hotornot/bitcoin.svg" alt="" class="w-6 h-6" /> }.into_any()
                        }}
                    </div>

                    // Today text
                    <div class="text-lg font-bold text-white mb-2">
                        "Today!"
                    </div>

                    // Subtitle
                    <p class="text-[10px] font-normal text-white mb-3">
                        "Be on top 10 of the leaderboard to win!"
                    </p>

                    // Contest countdown badge
                    <div class="inline-flex items-center gap-1.5 bg-neutral-900 rounded-full px-2 py-1">
                        <span class="text-neutral-400 text-[10px] font-normal">
                            "Contest ends on:"
                        </span>
                        <span class="text-neutral-50 text-[10px] font-medium">{move || time_remaining.get()}</span>
                    </div>
                </div>
            </div>

            // Gift box graphic positioned at bottom-right
            <div class="absolute bottom-0 right-4">
                <img src={if tournament.prize_token == "CKBTC" {
                    "/img/leaderboard/gift-box-btc-header.svg"
                } else {
                    "/img/leaderboard/gift-box-header.svg"
                }} alt="Gift Box" class="w-44 h-36" />
            </div>
        </div>
    }
}
