use super::types::TournamentInfo;
use leptos::prelude::*;

/// Common leaderboard table header component
#[component]
pub fn LeaderboardTableHeader(
    /// Tournament info to get the metric display name
    tournament_info: Signal<Option<TournamentInfo>>,
) -> impl IntoView {
    view! {
        <div class="sticky top-[72px] z-30 flex items-center justify-between px-4 py-2 border-b border-white/10 bg-black">
            <div class="w-[60px]">
                <span class="text-xs text-neutral-400 font-medium">Rank</span>
            </div>
            <div class="flex-1 text-left">
                <span class="text-xs text-neutral-400 font-medium">Username</span>
            </div>
            <div class="w-[81px] text-right">
                <span class="text-xs text-neutral-400 font-medium">
                    {move || tournament_info.get().map(|t| t.metric_display_name.clone()).unwrap_or_else(|| "Score".to_string())}
                </span>
            </div>
            <div class="w-[80px] flex justify-end">
                <span class="text-xs text-neutral-400 font-medium">Prize</span>
            </div>
        </div>
    }
}
