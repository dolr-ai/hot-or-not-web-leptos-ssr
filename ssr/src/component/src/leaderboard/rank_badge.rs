use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::*;
use leptos_router::hooks::use_navigate;
use state::canisters::auth_state;
use utils::send_wrap;

use super::api::fetch_user_rank_from_api;

#[component]
pub fn RankBadge() -> impl IntoView {
    let auth = auth_state();
    let user_rank = RwSignal::new(None::<u32>);
    let loading = RwSignal::new(false);

    // Get update trigger from context if available
    let update_trigger = use_context::<Trigger>().unwrap_or_else(|| Trigger::new());

    // Fetch rank when trigger fires or on mount
    Effect::new(move |_| {
        update_trigger.track();

        // Only fetch if user is logged in with OAuth
        if auth.is_logged_in_with_oauth().get() {
            spawn_local(send_wrap(async move {
                loading.set(true);

                // Get user principal
                if let Ok(principal) = auth.user_principal.await {
                    // Fetch rank from API
                    match fetch_user_rank_from_api(principal).await {
                        Ok(rank) => {
                            user_rank.set(rank);
                        }
                        Err(e) => {
                            leptos::logging::error!("Failed to fetch user rank: {}", e);
                        }
                    }
                }

                loading.set(false);
            }));
        }
    });

    view! {
        <Show when=move || user_rank().is_some() && !loading()>
            <div
                class="absolute top-4 right-4 z-20 cursor-pointer animate-fade-in"
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

                    // Rank Badge - positioned at bottom-right
                    <div class="absolute -bottom-1 -right-1 bg-[#f14331] text-white
                                rounded-lg px-1.5 py-0.5 text-xs font-bold min-w-[28px] 
                                text-center border-2 border-white shadow-md
                                group-hover:scale-110 transition-transform duration-200">
                        {move || format!("#{}", user_rank().unwrap_or(0))}
                    </div>

                    // Optional: Tooltip on hover
                    <div class="absolute bottom-full right-0 mb-2 opacity-0 group-hover:opacity-100
                                transition-opacity duration-200 pointer-events-none">
                        <div class="bg-black/80 text-white text-xs rounded px-2 py-1 whitespace-nowrap">
                            "Your Tournament Rank"
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
