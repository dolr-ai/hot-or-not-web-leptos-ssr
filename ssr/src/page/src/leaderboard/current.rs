use codee::string::JsonSerdeCodec;
use component::infinite_scroller::InfiniteScroller;
use component::leaderboard::{
    api::fetch_leaderboard_page,
    podium::TournamentPodium,
    provider::LeaderboardProvider,
    search_bar::SearchBar,
    tournament_completion_popup::TournamentCompletionPopup,
    tournament_header::TournamentHeader,
    types::{LeaderboardEntry, TournamentInfo, UserInfo},
};
use component::title::TitleText;
use leptos::{html, prelude::*};
use leptos_router::hooks::use_navigate;
use leptos_use::storage::use_local_storage;
#[cfg(feature = "hydrate")]
use leptos_use::use_debounce_fn;
#[cfg(feature = "hydrate")]
use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};
use state::canisters::{auth_state, unauth_canisters};

#[component]
pub fn Leaderboard() -> impl IntoView {
    let auth = auth_state();
    let canisters = unauth_canisters();
    let navigate = use_navigate();

    // State management
    let (tournament_info, set_tournament_info) = signal(None::<TournamentInfo>);
    let (upcoming_tournament_info, set_upcoming_tournament_info) = signal(None::<TournamentInfo>);
    let (current_user_info, set_current_user_info) = signal(None::<UserInfo>);
    let (sort_order, set_sort_order) = signal("desc".to_string());
    let (search_input, set_search_input) = signal(String::new()); // Immediate input value
    let (search_query, set_search_query) = signal(String::new()); // Debounced search value
    let (provider_key, set_provider_key) = signal(0u32); // Key to force provider refresh
    let show_completion_popup = RwSignal::new(false);
    let (user_row_visible, set_user_row_visible) = signal(false); // Track if user's actual row is visible

    // Fetch tournament info and user info once
    let tournament_resource = LocalResource::new(move || async move {
        let user_id = auth.user_principal.await.ok().map(|p| p.to_string());
        fetch_leaderboard_page(0, 1, user_id, Some("desc"), None).await
    });

    // Fetch top 3 winners and their profiles when tournament is completed
    let top_winners_resource = LocalResource::new(move || {
        let status = tournament_info.get().map(|t| t.status.clone());
        let canisters = canisters.clone();
        async move {
            if status == Some("completed".to_string()) {
                // Fetch top 3 from leaderboard
                let response = fetch_leaderboard_page(0, 3, None, Some("desc"), None).await?;
                let top_3 = response.data;

                // Fetch profile details for each winner
                let mut profiles = Vec::new();
                for entry in &top_3 {
                    let profile = canisters
                        .get_profile_details(entry.principal_id.clone())
                        .await
                        .ok()
                        .flatten();
                    profiles.push(profile);
                }

                Ok((top_3, profiles))
            } else {
                Err("Not completed".to_string())
            }
        }
    });

    // LocalStorage for tracking seen popups
    let (seen_tournaments, set_seen_tournaments, _) =
        use_local_storage::<Vec<String>, JsonSerdeCodec>("seen_tournament_completions");

    // Handle tournament info load
    Effect::new(move |_| {
        if let Some(Ok(response)) = tournament_resource.get() {
            let tournament = response.tournament_info.clone();
            set_tournament_info.set(Some(tournament.clone()));
            
            // Set upcoming tournament info if present
            if let Some(upcoming) = response.upcoming_tournament_info {
                set_upcoming_tournament_info.set(Some(upcoming));
            }

            // Parse user info if present
            if let Some(user_json) = response.user_info {
                if let Ok(user_info) = serde_json::from_value::<UserInfo>(user_json) {
                    set_current_user_info.set(Some(user_info.clone()));

                    // Check if should show completion popup
                    if tournament.status == "completed" {
                        let tournament_id = tournament.id.to_string();
                        let seen_list = seen_tournaments.get_untracked();

                        // Only show if not seen before and user is logged in
                        if !seen_list.contains(&tournament_id) {
                            // TODO: for testing, uncomment later by human
                            show_completion_popup.set(true);

                            // Mark as seen
                            let mut updated_seen = seen_list;
                            updated_seen.push(tournament_id);
                            set_seen_tournaments.set(updated_seen);
                        }
                    }
                }
            }
        }
    });

    // Debounced search function - executes 300ms after user stops typing
    // Only use debounce on client side to avoid SendWrapper thread issues in SSR
    #[cfg(feature = "hydrate")]
    let debounced_search = use_debounce_fn(
        move || {
            // Copy input value to search query after debounce
            let input = search_input.get();
            set_search_query.set(input);
            // Force provider refresh
            set_provider_key.update(|k| *k += 1);
        },
        500.0,
    );

    // For SSR, execute immediately without debounce
    #[cfg(not(feature = "hydrate"))]
    let debounced_search = move || {
        let input = search_input.get();
        set_search_query.set(input);
        set_provider_key.update(|k| *k += 1);
    };

    // Search function that updates input and triggers debounced search
    let on_search = StoredValue::new(move |input: String| {
        set_search_input.set(input.clone());

        if input.is_empty() {
            // For empty input, clear search immediately
            set_search_query.set(String::new());
            set_provider_key.update(|k| *k += 1);
        } else {
            // For non-empty input, trigger debounced search
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
                            let is_completed = tournament.status == "completed";

                            // Create provider inside Suspense to avoid hydration warnings
                            provider_key.get(); // Subscribe to refresh key
                            let uid = auth
                                .user_principal
                                .get()
                                .and_then(|res| res.ok())
                                .map(|p| p.to_string());
                            let order = sort_order.get();
                            let query = search_query.get();

                            let mut provider = if query.is_empty() {
                                LeaderboardProvider::new(uid, order)
                            } else {
                                LeaderboardProvider::new(uid, order).with_search(query.clone())
                            };

                            // Skip top 3 entries if tournament is completed (they're shown in podium)
                            if is_completed && query.is_empty() {
                                provider = provider.with_start_offset(3);
                            }

                            view! {
                                <>
                                    // Tournament header - only show if not completed
                                    <Show when=move || !is_completed>
                                        <TournamentHeader tournament=tournament.clone() />
                                    </Show>

                                    // Show podium if tournament is completed
                                    <Show when=move || is_completed>
                                        <Suspense fallback=move || view! {
                                            <div class="flex justify-center py-8">
                                                <div class="animate-spin h-8 w-8 border-t-2 border-pink-500 rounded-full"></div>
                                            </div>
                                        }>
                                            {move || {
                                                top_winners_resource.get().and_then(|result| {
                                                    result.ok().map(|(winners, profiles)| {
                                                        view! { <TournamentPodium winners winner_profiles=profiles /> }
                                                    })
                                                })
                                            }}
                                        </Suspense>
                                    </Show>

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

                                        // Sticky current user row (only shown when actual row is not visible and no search is active)
                                        <Show when=move || {
                                            !user_row_visible.get() && current_user_info.get().is_some() && search_query.get().is_empty()
                                        }>
                                            {move || {
                                                current_user_info.get().map(|user_info| {
                                                    // Get rank styling based on position
                                                    let rank_class = match user_info.rank {
                                                        1 => "bg-gradient-to-r from-[#BF760B] via-[#FFE89F] to-[#C38F14] bg-clip-text text-transparent",
                                                        2 => "bg-gradient-to-r from-[#2F2F30] via-[#FFFFFF] to-[#4B4B4B] bg-clip-text text-transparent",
                                                        3 => "bg-gradient-to-r from-[#6D4C35] via-[#DBA374] to-[#9F7753] bg-clip-text text-transparent",
                                                        _ => "text-white"
                                                    };

                                                    // Get username color based on rank
                                                    let username_color = match user_info.rank {
                                                        1 => "text-[#FDBF01]",
                                                        2 => "text-[#DCDCDC]",
                                                        3 => "text-[#D99979]",
                                                        _ => "text-white"
                                                    };

                                                    view! {
                                                        <div class="flex items-center justify-between px-4 py-3 border-b border-[#212121] bg-[rgba(226,1,123,0.2)]">
                                                            // Rank column
                                                            <div class="w-[80px]">
                                                                <span class=format!("text-lg font-bold {}", rank_class)>
                                                                    "#"{user_info.rank}
                                                                </span>
                                                            </div>

                                                            // Username column
                                                            <div class="flex-1">
                                                                <span class=format!("text-sm font-medium {}", username_color)>
                                                                    "@"{user_info.username}
                                                                </span>
                                                            </div>

                                                            // Games Played column
                                                            <div class="w-[100px] flex items-center justify-end gap-1">
                                                                <span class="text-sm font-semibold text-white">
                                                                    {user_info.score as u32}
                                                                </span>
                                                            </div>

                                                            // Rewards column
                                                            <div class="w-[100px] flex items-center justify-end gap-1">
                                                                <span class="text-sm font-semibold text-white">
                                                                    {user_info.reward.unwrap_or(0)}
                                                                </span>
                                                                // YRAL token icon
                                                                <img src="/img/yral/yral-token.webp" alt="" class="w-[17px] h-[18px]" />
                                                            </div>
                                                        </div>
                                                    }
                                                })
                                            }}
                                        </Show>

                                        <InfiniteScroller
                                            provider
                                            fetch_count=20
                                            children=move |entry: LeaderboardEntry, node_ref| {
                                                let is_current_user = current_user_info.get()
                                                    .map(|u| u.principal_id == entry.principal_id)
                                                    .unwrap_or(false);
                                                
                                                // Create a node ref for the current user's row
                                                let user_row_ref = NodeRef::<html::Div>::new();

                                                // Set up intersection observer for current user's row (only on client side)
                                                #[cfg(feature = "hydrate")]
                                                if is_current_user {
                                                    let _ = use_intersection_observer_with_options(
                                                        user_row_ref,
                                                        move |entries, _| {
                                                            if let Some(entry) = entries.get(0) {
                                                                set_user_row_visible.set(entry.is_intersecting());
                                                            }
                                                        },
                                                        UseIntersectionObserverOptions::default()
                                                            .root_margin("0px".to_string())
                                                            .thresholds(vec![0.1]),
                                                    );
                                                }

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
                                                        node_ref=if is_current_user { user_row_ref } else { node_ref.unwrap_or_default() }
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
                                                            // YRAL token icon
                                                            <img src="/img/yral/yral-token.webp" alt="" class="w-[17px] h-[18px]" />
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

            // Tournament completion popup
            <Show when=move || current_user_info.get().is_some() && show_completion_popup.get()>
                {
                    let popup_view = if let Some(upcoming) = upcoming_tournament_info.get() {
                        view! {
                            <TournamentCompletionPopup
                                show=show_completion_popup
                                user_info=current_user_info.get().unwrap()
                                upcoming_tournament=upcoming
                            />
                        }
                    } else {
                        view! {
                            <TournamentCompletionPopup
                                show=show_completion_popup
                                user_info=current_user_info.get().unwrap()
                            />
                        }
                    };
                    popup_view
                }
            </Show>
        </div>
    }
}
