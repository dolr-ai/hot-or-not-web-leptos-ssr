use component::infinite_scroller::InfiniteScroller;
use component::leaderboard::{
    api::fetch_leaderboard_page,
    provider::LeaderboardProvider,
    search_bar::SearchBar,
    tournament_header::TournamentHeader,
    types::{LeaderboardEntry, TournamentInfo, UserInfo},
};
use component::title::TitleText;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_use::use_debounce_fn;
use state::canisters::auth_state;

#[component]
pub fn Leaderboard() -> impl IntoView {
    let auth = auth_state();
    let navigate = use_navigate();

    // State management
    let (tournament_info, set_tournament_info) = signal(None::<TournamentInfo>);
    let (current_user_info, set_current_user_info) = signal(None::<UserInfo>);
    let (sort_order, set_sort_order) = signal("desc".to_string());
    let (search_query, set_search_query) = signal(String::new());
    let (provider_key, set_provider_key) = signal(0u32); // Key to force provider refresh

    // Fetch tournament info and user info once
    let tournament_resource = LocalResource::new(move || async move {
        let user_id = auth.user_principal.await.ok().map(|p| p.to_string());
        fetch_leaderboard_page(0, 1, user_id, Some("desc")).await
    });

    // Handle tournament info load
    Effect::new(move |_| {
        if let Some(Ok(response)) = tournament_resource.get() {
            set_tournament_info.set(Some(response.tournament_info));
            
            // Parse user info if present
            if let Some(user_json) = response.user_info {
                if let Ok(user_info) = serde_json::from_value::<UserInfo>(user_json) {
                    set_current_user_info.set(Some(user_info));
                }
            }
        }
    });

    // Get user ID once at initialization
    let (user_id, set_user_id) = signal(None::<String>);
    
    // Fetch user ID asynchronously
    leptos::task::spawn_local(async move {
        if let Ok(principal) = auth.user_principal.await {
            set_user_id.set(Some(principal.to_string()));
        }
    });
    
    // Create provider based on current state
    let provider = Memo::new(move |_| {
        provider_key.get(); // Subscribe to refresh key
        let uid = user_id.get();
        let order = sort_order.get();
        let query = search_query.get();
        
        if query.is_empty() {
            LeaderboardProvider::new(uid, order)
        } else {
            LeaderboardProvider::new(uid, order).with_search(query)
        }
    });

    // Debounced search function - executes 300ms after user stops typing
    let debounced_search = use_debounce_fn(
        move || {
            // Force provider refresh
            set_provider_key.update(|k| *k += 1);
        },
        300.0, // 300ms delay
    );

    // Search function that updates query and triggers debounced search
    let on_search = StoredValue::new(move |query: String| {
        set_search_query.set(query.clone());
        
        if query.is_empty() {
            // For empty query, reset immediately
            set_provider_key.update(|k| *k += 1);
        } else {
            // For non-empty query, trigger debounced search
            debounced_search();
        }
    });

    // Sort function - toggles between asc and desc
    let on_sort = move |field: String| {
        log::info!("Sorting by: {}", field);
        
        // Toggle sort order
        set_sort_order.update(|order| {
            *order = if order == "asc" {
                "desc".to_string()
            } else {
                "asc".to_string()
            };
        });
        
        // Force provider refresh
        set_provider_key.update(|k| *k += 1);
    };

    view! {
        <div class="min-h-screen bg-black text-white">
            // Header
            <TitleText>
                <div class="flex items-center justify-between w-full px-4">
                    <button
                        class="p-2"
                        on:click={let navigate = navigate.clone(); move |_| navigate("/", Default::default())}
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                        </svg>
                    </button>
                    <span class="text-xl font-bold">Leaderboard</span>
                    <button
                        class="text-pink-500 text-sm font-medium"
                        on:click={let navigate = navigate.clone(); move |_| navigate("/leaderboard/history", Default::default())}
                    >
                        "View History"
                    </button>
                </div>
            </TitleText>

            // Main content
            <div class="container mx-auto px-4 py-6 max-w-4xl">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center py-12">
                        <div class="animate-spin h-8 w-8 border-t-2 border-pink-500 rounded-full"></div>
                    </div>
                }>
                    {move || {
                        tournament_info.get().map(|tournament| {
                            view! {
                                <>
                                    // Tournament header
                                    <TournamentHeader tournament=tournament />

                                    // Search bar
                                    <SearchBar on_search=on_search.get_value() />

                                    // Infinite scrolling leaderboard
                                    <div class="w-full">
                                        // Table header
                                        <div class="flex items-center justify-between px-4 py-2 border-b border-white/10">
                                            <div class="flex items-center gap-1 w-[80px]">
                                                <span class="text-xs text-neutral-400 font-medium">Rank</span>
                                                <button 
                                                    class="text-neutral-400 hover:text-white transition-colors"
                                                    on:click={let on_sort = on_sort.clone(); move |_| on_sort("rank".to_string())}
                                                >
                                                    <span class="text-xs">{move || if sort_order.get() == "desc" { "↓" } else { "↑" }}</span>
                                                </button>
                                            </div>
                                            <div class="flex-1">
                                                <span class="text-xs text-neutral-400 font-medium">Username</span>
                                            </div>
                                            <div class="flex items-center gap-1 w-[100px] justify-end">
                                                <span class="text-xs text-neutral-400 font-medium">Games Played</span>
                                            </div>
                                            <div class="flex items-center gap-1 w-[100px] justify-end">
                                                <span class="text-xs text-neutral-400 font-medium">Rewards</span>
                                                <button 
                                                    class="text-neutral-400 hover:text-white transition-colors"
                                                    on:click={let on_sort = on_sort.clone(); move |_| on_sort("reward".to_string())}
                                                >
                                                    <span class="text-xs">{move || if sort_order.get() == "desc" { "↓" } else { "↑" }}</span>
                                                </button>
                                            </div>
                                        </div>
                                        
                                        <InfiniteScroller
                                            provider=provider.get()
                                            fetch_count=20
                                            children=move |entry: LeaderboardEntry, node_ref| {
                                                let is_current_user = current_user_info.get()
                                                    .map(|u| u.principal_id == entry.principal_id)
                                                    .unwrap_or(false);
                                                
                                                // Get rank styling based on position
                                                let rank_class = match entry.rank {
                                                    1 => "bg-gradient-to-r from-[#BF760B] via-[#FFE89F] to-[#C38F14] bg-clip-text text-transparent",
                                                    2 => "bg-gradient-to-r from-[#2F2F30] via-[#FFFFFF] to-[#4B4B4B] bg-clip-text text-transparent", 
                                                    3 => "bg-gradient-to-r from-[#6D4C35] via-[#DBA374] to-[#9F7753] bg-clip-text text-transparent",
                                                    _ => "text-white"
                                                };
                                                
                                                // Get username color based on rank
                                                let username_color = match entry.rank {
                                                    1 => "text-[#FDBF01]",
                                                    2 => "text-[#DCDCDC]",
                                                    3 => "text-[#D99979]",
                                                    _ => "text-white"
                                                };
                                                
                                                view! {
                                                    <div 
                                                        node_ref=node_ref.unwrap_or_default()
                                                        class=move || {
                                                            format!(
                                                                "flex items-center justify-between px-4 py-3 border-b border-[#212121] hover:bg-white/5 transition-colors {}",
                                                                if is_current_user { "bg-[rgba(226,1,123,0.2)]" } else { "" }
                                                            )
                                                        }
                                                    >
                                                        // Rank column
                                                        <div class="w-[80px]">
                                                            <span class=format!("text-lg font-bold {}", rank_class)>
                                                                "#"{entry.rank}
                                                            </span>
                                                        </div>
                                                        
                                                        // Username column
                                                        <div class="flex-1">
                                                            <span class=format!("text-sm font-medium {}", username_color)>
                                                                "@"{entry.username}
                                                            </span>
                                                        </div>
                                                        
                                                        // Games Played column
                                                        <div class="w-[100px] flex items-center justify-end gap-1">
                                                            <span class="text-sm font-semibold text-white">
                                                                {entry.score as u32}
                                                            </span>
                                                        </div>
                                                        
                                                        // Rewards column
                                                        <div class="w-[100px] flex items-center justify-end gap-1">
                                                            <span class="text-sm font-semibold text-white">
                                                                {entry.reward.unwrap_or(0)}
                                                            </span>
                                                            // Coin emoji as a simple icon
                                                            <span class="text-yellow-500">{"🪙"}</span>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                            empty_content=move || view! {
                                                <div class="text-center py-8 text-gray-400">
                                                    "No entries found"
                                                </div>
                                            }
                                        />
                                    </div>
                                </>
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}