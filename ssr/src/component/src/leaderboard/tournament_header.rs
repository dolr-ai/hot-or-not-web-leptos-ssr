use leptos::prelude::*;
use super::types::TournamentInfo;
use chrono::{DateTime, Datelike, Utc};

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
                let tz_str = tournament.client_timezone.as_ref()
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
                
                format!("{}{} {}, {} {}", 
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
                
                format!("{}{} {}, {} UTC", 
                    day, 
                    suffix, 
                    dt.format("%B"),
                    dt.format("%I:%M %p")
                )
            })
            .unwrap_or_else(|| "Unknown".to_string())
    };
    
    view! {
        <div class="relative w-full rounded-lg overflow-hidden mb-6">
            // Gradient background
            <div class="absolute inset-0 bg-gradient-to-r from-pink-600 to-purple-600 opacity-90"></div>
            
            // Content
            <div class="relative p-6 flex items-center justify-between">
                <div class="flex-1">
                    <h2 class="text-2xl font-bold text-white mb-2">
                        {format!("Win upto {} {} This Week!", 
                            tournament.prize_pool as u64,
                            tournament.prize_token
                        )}
                        <img src="/img/yral/yral-token.webp" alt="" class="w-6 h-6 ml-2 inline-block" />
                    </h2>
                    <p class="text-white/90 text-sm">
                        "Top the leaderboard this week to win!"
                    </p>
                    
                    // Contest end badge
                    <div class="mt-4 inline-flex items-center bg-white/20 backdrop-blur-sm rounded-full px-4 py-2">
                        <span class="text-white text-sm font-medium">
                            "Contest ends on "
                            <span class="font-bold">{end_date}</span>
                        </span>
                    </div>
                </div>
                
                // Gift box graphic (placeholder for now)
                <div class="hidden md:block w-32 h-32">
                    <svg viewBox="0 0 128 128" class="w-full h-full">
                        <rect x="20" y="50" width="88" height="60" fill="#e91e63" rx="4"/>
                        <rect x="20" y="45" width="88" height="15" fill="#f06292" rx="4"/>
                        <rect x="58" y="30" width="12" height="80" fill="#ffc107"/>
                        <rect x="10" y="45" width="108" height="12" fill="#ffc107"/>
                        <circle cx="64" cy="51" r="8" fill="#fff59d"/>
                    </svg>
                </div>
            </div>
        </div>
    }
}