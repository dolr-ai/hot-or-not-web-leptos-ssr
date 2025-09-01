use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_navigate;
use state::canisters::auth_state;
use utils::send_wrap;

use super::{api::fetch_user_rank_from_api, RankUpdateCounter};

/// Reusable rank badge view component
#[component]
fn RankBadgeView(
    /// Text to display in the badge (e.g., "...", "#2")
    rank_text: String,
) -> impl IntoView {
    view! {
        <div
            class="relative cursor-pointer animate-fade-in"
            on:click=move |_| {
                let navigate = use_navigate();
                navigate("/leaderboard", Default::default());
            }
        >
            <div class="relative group">
                // Trophy Icon with golden color
                <div class="w-10 h-10 flex items-center justify-center">
                    <Icon
                        attr:class="w-full h-full text-yellow-400 drop-shadow-lg
                                   group-hover:scale-110 transition-transform duration-200"
                        icon=icondata::RiTrophyFinanceFill
                    />
                </div>
                
                // Rank Badge
                <div class="absolute -bottom-1 -right-1 bg-[#f14331] text-white
                            rounded-lg px-1.5 py-0.5 text-xs font-bold min-w-[28px] 
                            text-center border-2 border-white shadow-md
                            group-hover:scale-110 transition-transform duration-200">
                    {rank_text}
                </div>
                
                // Tooltip on hover
                <div class="absolute bottom-full right-0 mb-2 opacity-0 group-hover:opacity-100
                            transition-opacity duration-200 pointer-events-none">
                    <div class="bg-black/80 text-white text-xs rounded px-2 py-1 whitespace-nowrap">
                        "Your Tournament Rank"
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn RankBadge() -> impl IntoView {
    let auth = auth_state();

    // Get update counter from context
    let rank_update_count = use_context::<RwSignal<RankUpdateCounter>>()
        .unwrap_or_else(|| RwSignal::new(RankUpdateCounter(0)));

    // Use Resource to fetch rank - tracks the counter for updates
    let rank_resource = Resource::new(
        move || rank_update_count.get().0,
        move |_| {
            send_wrap(async move {
                // Get user principal
                let principal = auth.user_principal.await.ok()?;
                
                leptos::logging::log!("RankBadge: Fetching rank for principal: {}", principal);
                
                // Fetch rank from API
                match fetch_user_rank_from_api(principal).await {
                    Ok(rank) => {
                        leptos::logging::log!("RankBadge: Fetched rank: {:?}", rank);
                        rank
                    }
                    Err(e) => {
                        leptos::logging::error!("RankBadge: Failed to fetch user rank: {}", e);
                        None
                    }
                }
            })
        },
    );
    
    view! {
        <Suspense
            fallback=move || view! {
                // Show loading state with "..."
                <RankBadgeView rank_text="...".to_string() />
            }
        >
            {move || {
                rank_resource.get().map(|rank_option| {
                    rank_option.map(|rank| {
                        // Show badge with actual rank
                        view! {
                            <RankBadgeView rank_text=format!("#{}", rank) />
                        }
                    })
                }).flatten()
            }}
        </Suspense>
    }
}
