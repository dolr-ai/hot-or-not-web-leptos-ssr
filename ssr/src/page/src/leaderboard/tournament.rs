use chrono::{DateTime, Utc};
use component::infinite_scroller::InfiniteScroller;
use component::leaderboard::{
    api::fetch_leaderboard_page,
    tournament_provider::TournamentLeaderboardProvider,
    types::{LeaderboardEntry, TournamentInfo, UserInfo},
};
use component::title::TitleText;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
use state::canisters::auth_state;

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Params)]
pub struct TournamentParams {
    pub id: String,
}

fn format_date(timestamp: i64) -> String {
    let datetime =
        DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now().into());
    datetime.format("%b %d").to_string()
}

#[component]
pub fn TournamentResults() -> impl IntoView {
    let auth = auth_state();
    let navigate = use_navigate();
    let params = use_params::<TournamentParams>();

    // Get tournament ID from params
    let tournament_id = move || params.get().map(|p| p.id).unwrap_or_else(|_| String::new());

    // State management
    let (tournament_info, set_tournament_info) = signal(None::<TournamentInfo>);
    let (current_user_info, set_current_user_info) = signal(None::<UserInfo>);
    let (sort_order, set_sort_order) = signal("desc".to_string());
    let (provider_key, set_provider_key) = signal(0u32); // Key to force provider refresh

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

    // Provider will be created inside Suspense to avoid hydration warnings

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
                        on:click={let navigate = navigate.clone(); move |_| navigate("/leaderboard/history", Default::default())}
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                        </svg>
                    </button>
                    <span class="text-xl font-bold">Tournament Results</span>
                    <div class="w-10"></div> // Spacer for centering
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
                            // Create provider inside Suspense to avoid hydration warnings
                            provider_key.get(); // Subscribe to refresh key
                            let tid = tournament_id();
                            let uid = auth
                                .user_principal
                                .get()
                                .and_then(|res| res.ok())
                                .map(|p| p.to_string());
                            let order = sort_order.get();

                            if tid.is_empty() {
                                view! {
                                    <div class="text-center py-8">
                                        <p class="text-gray-400">"Tournament not found"</p>
                                    </div>
                                }.into_any()
                            } else {
                                let prov = TournamentLeaderboardProvider::new(tid, uid, order);
                                view! {
                                    <>
                                        // Tournament info card
                                        <div class="bg-black border border-[#212121] rounded-lg p-4 mb-6">
                                            <div class="flex items-center justify-between mb-2">
                                                <h2 class="text-lg font-bold">Tournament Details</h2>
                                                <span class=move || {
                                                    match tournament.status.as_str() {
                                                        "active" => "text-green-500",
                                                        "completed" => "text-gray-500",
                                                        _ => "text-yellow-500"
                                                    }
                                                }>
                                                    {tournament.status.clone()}
                                                </span>
                                            </div>
                                            <div class="text-sm text-gray-400 space-y-1">
                                                <div>
                                                    "Prize Pool: "
                                                    <span class="text-[#FFEF00] font-bold">
                                                        {tournament.prize_pool} " " {tournament.prize_token}
                                                    </span>
                                                </div>
                                                <div>"Date: " {format_date(tournament.start_time)}</div>
                                            </div>
                                        </div>

                                        // Leaderboard
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
                                                provider=prov
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
