use super::types::{LeaderboardEntry, UserInfo};
use leptos::prelude::*;

#[component]
pub fn LeaderboardTable(
    entries: Vec<LeaderboardEntry>,
    current_user: Option<UserInfo>,
    on_sort: impl Fn(String) + Clone + 'static,
    sort_order: String,
) -> impl IntoView {
    let current_user_id = current_user.as_ref().map(|u| u.principal_id.clone());

    view! {
        <div class="w-full overflow-x-auto">
            <table class="w-full">
                <thead>
                    <tr class="border-b border-neutral-800">
                        <th class="text-left py-3 px-4">
                            <button
                                class="flex items-center gap-1 text-neutral-400 hover:text-white transition-colors"
                                on:click={let on_sort = on_sort.clone(); move |_| on_sort("rank".to_string())}
                            >
                                "Rank"
                                <span class="text-xs">
                                    {if sort_order == "asc" { "↑" } else { "↓" }}
                                </span>
                            </button>
                        </th>
                        <th class="text-left py-3 px-4">
                            <span class="text-neutral-400">Username</span>
                        </th>
                        <th class="text-right py-3 px-4">
                            <button
                                class="flex items-center gap-1 text-neutral-400 hover:text-white transition-colors ml-auto"
                                on:click={let on_sort = on_sort.clone(); move |_| on_sort("earned".to_string())}
                            >
                                <img src="/img/yral/yral-token.webp" alt="" class="w-4 h-4 inline-block" />
                                "Earned"
                                <span class="text-xs">
                                    {if sort_order == "asc" { "↑" } else { "↓" }}
                                </span>
                            </button>
                        </th>
                        <th class="text-right py-3 px-4">
                            <span class="text-neutral-400">Rewards</span>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {entries
                        .into_iter()
                        .map(|entry| {
                            let is_current_user = current_user_id.as_ref()
                                .map(|id| id == &entry.principal_id)
                                .unwrap_or(false);

                            view! {
                                <LeaderboardRow
                                    entry=entry
                                    is_current_user=is_current_user
                                />
                            }
                        })
                        .collect_view()
                    }
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn LeaderboardRow(entry: LeaderboardEntry, is_current_user: bool) -> impl IntoView {
    // Determine rank styling
    let rank_class = match entry.rank {
        1 => {
            "bg-gradient-to-r from-yellow-400 to-yellow-600 bg-clip-text text-transparent font-bold"
        }
        2 => "bg-gradient-to-r from-gray-300 to-gray-500 bg-clip-text text-transparent font-bold",
        3 => {
            "bg-gradient-to-r from-orange-400 to-orange-600 bg-clip-text text-transparent font-bold"
        }
        _ => "text-white",
    };

    // Row background for current user
    let row_class = if is_current_user {
        "bg-pink-500/20 border-l-4 border-pink-500"
    } else {
        "hover:bg-neutral-900/50 transition-colors"
    };

    view! {
        <tr class=format!("border-b border-neutral-800 {}", row_class)>
            <td class="py-4 px-4">
                <span class=format!("text-lg {}", rank_class)>
                    {format!("#{}", entry.rank)}
                </span>
            </td>
            <td class="py-4 px-4">
                <span class="text-white font-medium">
                    {format!("@{}", entry.username)}
                </span>
            </td>
            <td class="py-4 px-4 text-right">
                <span class="text-white font-semibold">
                    {format!("{:.0}", entry.score)}
                </span>
            </td>
            <td class="py-4 px-4 text-right">
                <div class="flex items-center justify-end gap-1">
                    <span class="text-white font-semibold">
                        {entry.reward.map(|r| r.to_string()).unwrap_or_else(|| "0".to_string())}
                    </span>
                    <img src="/img/yral/yral-token.webp" alt="" class="w-[17px] h-[18px] inline-block" />
                </div>
            </td>
        </tr>
    }
}
