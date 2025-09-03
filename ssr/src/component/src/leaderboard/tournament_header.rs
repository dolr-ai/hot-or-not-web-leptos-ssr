use super::types::TournamentInfo;
use chrono::{DateTime, Datelike, Utc};
use leptos::prelude::*;

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
    let (_time_remaining, set_time_remaining) = signal(String::new());

    // Update countdown every minute
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

            if days > 0 {
                format!("{}d {}h {}m", days, hours, minutes)
            } else if hours > 0 {
                format!("{}h {}m", hours, minutes)
            } else {
                format!("{}m", minutes)
            }
        }
    };

    // Set initial value
    set_time_remaining.set(calculate_time_remaining());

    // Format end date - use client_end_time if available, otherwise fallback to UTC
    let end_date = if let Some(client_time) = &tournament.client_end_time {
        // Parse ISO 8601 string and format for display
        DateTime::parse_from_rfc3339(client_time)
            .map(|dt| {
                // Extract timezone abbreviation if available
                let tz_str = tournament
                    .client_timezone
                    .as_ref()
                    .and_then(|tz| tz.split('/').last())
                    .unwrap_or("Local Time");

                // Format with day suffix handling
                let day = dt.day();
                let suffix = match day {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };

                format!(
                    "{}{} {}, {} {}",
                    day,
                    suffix,
                    dt.format("%B"),
                    dt.format("%I:%M %p"),
                    tz_str
                )
            })
            .unwrap_or_else(|_| {
                // If parsing fails, use the raw string
                client_time.clone()
            })
    } else {
        // Fallback to UTC formatting
        DateTime::from_timestamp(tournament.end_time, 0)
            .map(|dt| {
                let day = dt.day();
                let suffix = match day {
                    1 | 21 | 31 => "st",
                    2 | 22 => "nd",
                    3 | 23 => "rd",
                    _ => "th",
                };

                format!(
                    "{}{} {}, {} UTC",
                    day,
                    suffix,
                    dt.format("%B"),
                    dt.format("%I:%M %p")
                )
            })
            .unwrap_or_else(|| "Unknown".to_string())
    };

    view! {
        <div class="relative w-full rounded-lg overflow-hidden mb-6 bg-gradient-to-br from-[#4A0E2E] via-[#3D1B47] to-[#1A0B2E] min-h-[140px]">
            // Background pattern overlay (simulating the Figma design pattern)
            <div class="absolute inset-0 opacity-30"
                style="background-image: radial-gradient(circle at 20% 50%, rgba(226, 1, 123, 0.3) 0%, transparent 50%),
                       radial-gradient(circle at 80% 80%, rgba(205, 41, 255, 0.2) 0%, transparent 50%),
                       radial-gradient(circle at 40% 80%, rgba(226, 1, 123, 0.2) 0%, transparent 50%);">
            </div>

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

                    // Contest end badge
                    <div class="mt-4 inline-flex items-center gap-1.5 bg-neutral-900 rounded-full px-2 py-1">
                        <span class="text-neutral-400 text-[10px] font-normal">
                            "Contest ends on:"
                        </span>
                        <span class="text-neutral-50 text-[10px] font-medium">{end_date}</span>
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
