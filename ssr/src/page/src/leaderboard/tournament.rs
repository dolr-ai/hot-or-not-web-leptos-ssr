use chrono::{DateTime, Utc};
use component::infinite_scroller::InfiniteScroller;
use component::leaderboard::{
    api::fetch_leaderboard_page,
    podium::TournamentPodium,
    tournament_provider::TournamentLeaderboardProvider,
    types::{LeaderboardEntry, TournamentInfo, UserInfo},
};
use leptos::{html, prelude::*};
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
#[cfg(feature = "hydrate")]
use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};
use state::canisters::{auth_state, unauth_canisters};

// Sticky header component for leaderboard with solid background
#[component]
fn StickyLeaderboardHeader(children: Children) -> impl IntoView {
    view! {
        <div class="sticky top-0 z-50 bg-black border-b border-white/10">
            <div class="flex items-center justify-between w-full px-4 py-4">
                {children()}
            </div>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Params)]
pub struct TournamentParams {
    pub id: String,
}

fn _format_date(timestamp: i64) -> String {
    let datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
    datetime.format("%b %d").to_string()
}

#[component]
pub fn TournamentResults() -> impl IntoView {
    let auth = auth_state();
    let canisters = unauth_canisters();
    let navigate = use_navigate();
    let params = use_params::<TournamentParams>();

    // Get tournament ID from params
    let tournament_id = move || params.get().map(|p| p.id).unwrap_or_else(|_| String::new());

    // State management
    let (tournament_info, set_tournament_info) = signal(None::<TournamentInfo>);
    let (current_user_info, set_current_user_info) = signal(None::<UserInfo>);
    let (provider_key, _set_provider_key) = signal(0u32); // Key to force provider refresh
    let (user_row_visible, set_user_row_visible) = signal(false);

    // Fetch tournament info and user info
    let tournament_resource = LocalResource::new(move || async move {
        let tid = tournament_id();
        let uid = auth.user_principal.await.ok().map(|p| p.to_string());
        if tid.is_empty() {
            Err("No tournament ID provided".to_string())
        } else {
            fetch_leaderboard_page(0, 1, uid, Some("desc"), Some(tid)).await
        }
    });

    // Fetch top 3 winners and their profiles when tournament is completed
    let top_winners_resource = LocalResource::new(move || {
        let status = tournament_info.get().map(|t| t.status.clone());
        let tid = tournament_id();
        let canisters = canisters.clone();
        async move {
            if status == Some("completed".to_string()) && !tid.is_empty() {
                // Fetch top 3 from leaderboard
                let response = fetch_leaderboard_page(0, 3, None, Some("desc"), Some(tid)).await?;
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

    view! {
        <div class="min-h-screen bg-black text-white">
            // Sticky Header with solid background
            <StickyLeaderboardHeader>
                <button
                    on:click={let navigate = navigate.clone(); move |_| navigate("/leaderboard/history", Default::default())}
                >
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                    </svg>
                </button>
                <span class="p-2 text-xl font-bold">Tournament Results</span>
                <div class="w-10"></div> // Spacer for centering
            </StickyLeaderboardHeader>

            // Main content
            <div class="container mx-auto px-4 py-6 max-w-4xl">
                <Suspense fallback=move || view! {
                    <div class="flex justify-center py-12">
                        <div class="animate-spin h-8 w-8 border-t-2 border-pink-500 rounded-full"></div>
                    </div>
                }>
                    {move || {
                        tournament_info.get().map(|tournament| {
                            // Create provider inside Suspense to avoid hydration warnings
                            provider_key.get(); // Subscribe to refresh key
                            let tid = tournament_id();
                            let uid = auth
                                .user_principal
                                .get()
                                .and_then(|res| res.ok())
                                .map(|p| p.to_string());

                            if tid.is_empty() {
                                view! {
                                    <div class="text-center py-8">
                                        <p class="text-gray-400">"Tournament not found"</p>
                                    </div>
                                }.into_any()
                            } else {
                                let is_completed = tournament.status == "completed";
                                let mut prov = TournamentLeaderboardProvider::new(tid.clone(), uid, "desc".to_string());

                                // Skip top 3 entries if tournament is completed (they're shown in podium)
                                if is_completed {
                                    prov = prov.with_start_offset(3);
                                }

                                view! {
                                    <>

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

                                        // Table header - sticky below the main header
                                        <div class="sticky top-[72px] z-30 flex items-center justify-between px-4 py-2 border-b border-white/10 bg-black">
                                                <div class="flex items-center gap-1 w-[60px]">
                                                    <span class="text-xs text-neutral-400 font-medium">Rank</span>
                                                </div>
                                                <div class="flex-1 text-left">
                                                    <span class="text-xs text-neutral-400 font-medium">Username</span>
                                                </div>
                                                <div class="flex items-center gap-1 w-[81px] justify-end">
                                                    <span class="text-xs text-neutral-400 font-medium">Games Played</span>
                                                </div>
                                                <div class="flex items-center gap-1 w-[80px] justify-end">
                                                    <span class="text-xs text-neutral-400 font-medium">Prize</span>
                                                </div>
                                        </div>

                                        // Sticky current user row (only shown when actual row is not visible and user is not in top 3)
                                        <Show when=move || {
                                            !user_row_visible.get() &&
                                            current_user_info.get().is_some() &&
                                            current_user_info.get().map(|u| u.rank > 3).unwrap_or(false)
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
                                                        <div class="top-[120px] z-25 flex items-center justify-between px-4 py-3 border-b border-[#212121]"
                                                            style="background: linear-gradient(90deg, rgba(226, 1, 123, 0.3), rgba(226, 1, 123, 0.1));">
                                                            // Rank column
                                                            <div class="w-[60px]">
                                                                <span class=format!("text-lg font-bold {}", rank_class)>
                                                                    "#"{user_info.rank}
                                                                </span>
                                                            </div>

                                                            // Username column
                                                            <div class="flex-1 text-left min-w-0">
                                                                <span class=format!("text-sm font-medium truncate block {}", username_color)>
                                                                    "@"{user_info.username}
                                                                </span>
                                                            </div>

                                                            // Games Played column
                                                            <div class="w-[80px] text-right">
                                                                <span class="text-sm font-semibold text-white">
                                                                    {user_info.score as u32}
                                                                </span>
                                                            </div>

                                                            // Rewards column
                                                            <div class="w-[80px] flex items-center justify-end gap-1">
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

                                        <div class="w-full">
                                            <InfiniteScroller
                                                provider=prov
                                                fetch_count=20
                                                children=move |entry: LeaderboardEntry, node_ref| {
                                                    let is_current_user = current_user_info.get()
                                                        .map(|u| u.principal_id == entry.principal_id)
                                                        .unwrap_or(false);

                                                    // Create a dedicated node ref for the current user's row
                                                    let user_row_ref = NodeRef::<html::Div>::new();

                                                    // Set up intersection observer for current user's row (only on client side)
                                                    #[cfg(feature = "hydrate")]
                                                    if is_current_user {
                                                        let _ = use_intersection_observer_with_options(
                                                            user_row_ref,
                                                            move |entries, _| {
                                                                if let Some(entry) = entries.first() {
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
                                                            <div class="w-[60px]">
                                                                <span class=format!("text-lg font-bold {}", rank_class)>
                                                                    "#"{entry.rank}
                                                                </span>
                                                            </div>

                                                            // Username column
                                                            <div class="flex-1 text-left min-w-0">
                                                                <span class=format!("text-sm font-medium truncate block {}", username_color)>
                                                                    "@"{entry.username}
                                                                </span>
                                                            </div>

                                                            // Games Played column
                                                            <div class="w-[80px] text-right">
                                                                <span class="text-sm font-semibold text-white">
                                                                    {entry.score as u32}
                                                                </span>
                                                            </div>

                                                            // Rewards column
                                                            <div class="w-[80px] flex items-center justify-end gap-1">
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
                                }.into_any()
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
