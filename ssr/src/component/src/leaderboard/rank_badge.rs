use super::UserRank;
use leptos::prelude::*;

/// Reusable rank badge view component
#[component]
fn RankBadgeView(
    /// Text to display in the badge (e.g., "...", "#2", "NA")
    rank_text: String,
    /// Whether the tournament is active
    #[prop(default = true)]
    is_active: bool,
) -> impl IntoView {
    view! {
        <a
            href="/leaderboard"
            class="relative cursor-pointer animate-fade-in block"
            style={if !is_active { "filter: grayscale(100%) opacity(60%)" } else { "" }}
        >
            <div class="relative group">
                // Trophy Icon from SVG
                <div class="w-10 h-10 flex items-center justify-center">
                    <img
                        src="/img/leaderboard/trophy-feed.svg"
                        alt="Trophy"
                        class="w-full h-full drop-shadow-lg
                               group-hover:scale-110 transition-transform duration-200"
                    />
                </div>

                // Rank Badge positioned at the center of the trophy horizontally, at the base
                // Only show the badge if tournament is active
                <Show when=move || is_active>
                    <div class="absolute -bottom-2 left-1/2 -translate-x-1/2 bg-[#F14331] text-white
                                rounded-[8px] px-1.5 py-0.5 text-xs font-bold min-w-[32px]
                                text-center border-[3px] border-white shadow-md
                                group-hover:scale-110 transition-transform duration-200">
                        {rank_text.clone()}
                    </div>
                </Show>

                // Tooltip on hover
                <div class="absolute bottom-full right-0 mb-2 opacity-0 group-hover:opacity-100
                            transition-opacity duration-200 pointer-events-none">
                    <div class="bg-black/80 text-white text-xs rounded px-2 py-1 whitespace-nowrap">
                        {if is_active { "Your Tournament Rank" } else { "Tournament Inactive" }}
                    </div>
                </div>
            </div>
        </a>
    }
}

/// Global rank badge that uses the single global LocalResource
#[component]
pub fn GlobalRankBadge() -> impl IntoView {
    let global_rank_resource =
        use_context::<LocalResource<Result<UserRank, leptos::prelude::ServerFnError>>>()
            .expect("Global rank LocalResource should be provided");

    view! {
        <Transition
            fallback=move || {
            view! {
                // Initial loading state
                <RankBadgeView rank_text="N/A".to_string() is_active=false />
            }
        }
        >
            {move || {
                global_rank_resource.get().map(|result| {
                    let user_rank = result.unwrap_or(UserRank {
                        rank: None,
                        tournament_status: None,
                    });

                    // Check if tournament is active
                    let is_active = user_rank.tournament_status.as_ref()
                        .map(|s| s == "active")
                        .unwrap_or(false);

                    // Display rank or NA based on tournament status
                    let rank_text = if is_active {
                        match user_rank.rank {
                            Some(rank) => format!("#{rank}"),
                            None => "N/A".to_string(),
                        }
                    } else {
                        "N/A".to_string()
                    };

                    view! {
                        <RankBadgeView rank_text is_active />
                    }
                })
            }}
        </Transition>
    }
}
