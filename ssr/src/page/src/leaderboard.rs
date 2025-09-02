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
                                        <div class="mb-4 flex justify-end">
                                            <button
                                                class="text-sm text-gray-400 hover:text-white transition-colors"
                                                on:click={let on_sort = on_sort.clone(); move |_| on_sort("score".to_string())}
                                            >
                                                {move || format!("Sort: {}", if sort_order.get() == "desc" { "↓ Highest" } else { "↑ Lowest" })}
                                            </button>
                                        </div>
                                        
                                        <InfiniteScroller
                                            provider=provider.get()
                                            fetch_count=20
                                            children=move |entry: LeaderboardEntry, node_ref| {
                                                let is_current_user = current_user_info.get()
                                                    .map(|u| u.principal_id == entry.principal_id)
                                                    .unwrap_or(false);
                                                
                                                view! {
                                                    <div 
                                                        node_ref=node_ref.unwrap_or_default()
                                                        class=move || {
                                                            format!(
                                                                "flex items-center justify-between p-4 border-b border-white/10 hover:bg-white/5 transition-colors {}",
                                                                if is_current_user { "bg-pink-500/10" } else { "" }
                                                            )
                                                        }
                                                    >
                                                        <div class="flex items-center gap-4">
                                                            <span class="text-2xl font-bold text-white/60 w-12 text-center">
                                                                {entry.rank}
                                                            </span>
                                                            <div class="flex flex-col">
                                                                <span class="text-white font-medium">{entry.username}</span>
                                                                <span class="text-white/40 text-xs">{entry.principal_id.chars().take(10).collect::<String>()}"..."</span>
                                                            </div>
                                                        </div>
                                                        <div class="flex items-center gap-4">
                                                            <span class="text-white font-bold">{entry.score}</span>
                                                            {entry.reward.map(|reward| {
                                                                view! {
                                                                    <span class="text-green-400 text-sm">
                                                                        "+" {reward} " YRAL"
                                                                    </span>
                                                                }
                                                            })}
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