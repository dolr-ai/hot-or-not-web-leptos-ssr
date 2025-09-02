use component::leaderboard::{
    api::{fetch_leaderboard_page, search_users},
    pagination::PaginationControls,
    search_bar::SearchBar,
    table::LeaderboardTable,
    tournament_header::TournamentHeader,
    types::{LeaderboardEntry, UserInfo},
};
use component::title::TitleText;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use leptos_use::use_debounce_fn;
use state::canisters::auth_state;

#[component]
pub fn Leaderboard() -> impl IntoView {
    let auth = auth_state();
    let navigate = use_navigate();

    // State management
    let (entries, set_entries) = signal(Vec::<LeaderboardEntry>::new());
    let (tournament_info, set_tournament_info) = signal(None);
    let (current_user_info, set_current_user_info) = signal(None::<UserInfo>);
    let (cursor_start, set_cursor_start) = signal(0u32);
    let (search_cursor, set_search_cursor) = signal(0u32);
    let (has_more, set_has_more) = signal(true);
    let (is_loading, set_is_loading) = signal(false);
    let (load_trigger, set_load_trigger) = signal(());
    let (error_message, set_error_message) = signal(None::<String>);
    let (sort_order, set_sort_order) = signal("desc".to_string());
    let (search_query, set_search_query) = signal(String::new());

    // Fetch initial data
    let initial_resource = LocalResource::new(move || async move {
        let user_id = auth.user_principal.await.ok().map(|p| p.to_string());
        let order = sort_order.get_untracked();

        fetch_leaderboard_page(0, 20, user_id, Some(&order)).await
    });

    // Handle initial data load
    Effect::new(move |_| {
        if let Some(Ok(response)) = initial_resource.get() {
            set_entries.set(response.data);
            set_tournament_info.set(Some(response.tournament_info));
            set_has_more.set(response.cursor_info.has_more);
            set_cursor_start.set(response.cursor_info.next_cursor.unwrap_or(0));

            // Parse user info if present
            if let Some(user_json) = response.user_info {
                if let Ok(user_info) = serde_json::from_value::<UserInfo>(user_json) {
                    set_current_user_info.set(Some(user_info));
                }
            }
        }
    });

    // Load more effect - triggers when load_trigger changes
    Effect::new(move |_| {
        load_trigger.get(); // Subscribe to trigger
        let query = search_query.get_untracked();
        
        // Check if we're in search mode or normal mode
        if query.is_empty() {
            // Normal mode - load from leaderboard
            if cursor_start.get() > 0 {
                // Skip initial load
                spawn_local(async move {
                    set_is_loading.set(true);
                    let user_id = auth.user_principal.await.ok().map(|p| p.to_string());
                    let order = sort_order.get_untracked();

                    match fetch_leaderboard_page(cursor_start.get(), 20, user_id, Some(&order)).await {
                        Ok(response) => {
                            let mut current = entries.get();
                            current.extend(response.data);
                            set_entries.set(current);
                            set_has_more.set(response.cursor_info.has_more);
                            set_cursor_start.set(response.cursor_info.next_cursor.unwrap_or(0));
                        }
                        Err(e) => {
                            set_error_message.set(Some(format!("Failed to load more: {}", e)));
                        }
                    }
                    set_is_loading.set(false);
                });
            }
        } else {
            // Search mode - load more search results
            if search_cursor.get() > 0 {
                spawn_local(async move {
                    set_is_loading.set(true);
                    let order = sort_order.get_untracked();
                    
                    match search_users(query, search_cursor.get(), 20, Some(&order)).await {
                        Ok(response) => {
                            let mut current = entries.get();
                            current.extend(response.data);
                            set_entries.set(current);
                            set_has_more.set(response.cursor_info.has_more);
                            set_search_cursor.set(response.cursor_info.next_cursor.unwrap_or(0));
                        }
                        Err(e) => {
                            set_error_message.set(Some(format!("Failed to load more search results: {}", e)));
                        }
                    }
                    set_is_loading.set(false);
                });
            }
        }
    });

    // Debounced search function - executes 300ms after user stops typing
    let debounced_search = use_debounce_fn(
        {
            let set_entries = set_entries.clone();
            let set_has_more = set_has_more.clone();
            let set_is_loading = set_is_loading.clone();
            let set_error_message = set_error_message.clone();
            let search_query = search_query.clone();
            let sort_order = sort_order.clone();
            let set_search_cursor = set_search_cursor.clone();
            let set_cursor_start = set_cursor_start.clone();

            move || {
                let query = search_query.get_untracked();
                
                if query.is_empty() {
                    // Reset to initial state
                    set_search_cursor.set(0);
                    initial_resource.refetch();
                    return;
                }

                let set_entries = set_entries.clone();
                let set_has_more = set_has_more.clone();
                let set_is_loading = set_is_loading.clone();
                let set_error_message = set_error_message.clone();
                let sort_order = sort_order.clone();
                let set_search_cursor = set_search_cursor.clone();
                let set_cursor_start = set_cursor_start.clone();

                spawn_local(async move {
                    set_is_loading.set(true);
                    let order = sort_order.get_untracked();
                    match search_users(query, 0, 20, Some(&order)).await {
                        Ok(response) => {
                            set_entries.set(response.data);
                            set_has_more.set(response.cursor_info.has_more);
                            set_search_cursor.set(response.cursor_info.next_cursor.unwrap_or(0));
                            // Reset normal cursor when entering search mode
                            set_cursor_start.set(0);
                        }
                        Err(e) => {
                            set_error_message.set(Some(format!("Search failed: {}", e)));
                        }
                    }
                    set_is_loading.set(false);
                });
            }
        },
        300.0  // 300ms delay
    );

    // Search function that updates query and triggers debounced search
    let on_search = {
        let debounced_search = debounced_search.clone();
        StoredValue::new(move |query: String| {
            set_search_query.set(query.clone());
            
            if query.is_empty() {
                // For empty query, reset immediately without debouncing
                set_search_cursor.set(0);
                initial_resource.refetch();
            } else {
                // For non-empty query, reset search cursor and trigger debounced search
                set_search_cursor.set(0);
                debounced_search();
            }
        })
    };

    // Sort function - toggles between asc and desc
    let on_sort = {
        let set_sort_order = set_sort_order.clone();
        let set_cursor_start = set_cursor_start.clone();
        let set_search_cursor = set_search_cursor.clone();
        let initial_resource = initial_resource.clone();
        let search_query = search_query.clone();
        let debounced_search = debounced_search.clone();
        
        move |field: String| {
            log::info!("Sorting by: {}", field);
            
            // Toggle sort order
            set_sort_order.update(|order| {
                *order = if order == "asc" { "desc".to_string() } else { "asc".to_string() };
            });
            
            // Reset pagination
            set_cursor_start.set(0);
            set_search_cursor.set(0);
            
            // Check if we're in search mode or normal mode
            if search_query.get_untracked().is_empty() {
                // Normal mode - refetch from leaderboard
                initial_resource.refetch();
            } else {
                // Search mode - re-run search with new sort order
                debounced_search();
            }
        }
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

                                    // Error message
                                    {move || error_message.get().map(|msg| {
                                        view! {
                                            <div class="bg-red-500/20 border border-red-500 rounded-lg p-4 mb-4">
                                                <span class="text-red-400">{msg}</span>
                                            </div>
                                        }
                                    })}

                                    // Leaderboard table
                                    <LeaderboardTable
                                        entries=entries.get()
                                        current_user=current_user_info.get()
                                        on_sort=on_sort.clone()
                                        sort_order=sort_order.get()
                                    />

                                    // Pagination
                                    <PaginationControls
                                        has_more=has_more.into()
                                        is_loading=is_loading.into()
                                        trigger_load=set_load_trigger
                                    />
                                </>
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
