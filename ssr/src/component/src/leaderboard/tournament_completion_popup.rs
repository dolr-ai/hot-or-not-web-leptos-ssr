use super::types::UserInfo;
use leptos::prelude::*;

#[derive(Clone, Debug)]
enum PopupVariant {
    Champion { reward: Option<u32> },
    Silver { reward: Option<u32> },
    Bronze { reward: Option<u32> },
    TopTen { rank: u32, reward: Option<u32> },
    BetterLuck,
}

#[component]
pub fn TournamentCompletionPopup(show: RwSignal<bool>, user_info: UserInfo) -> impl IntoView {
    let _navigate = leptos_router::hooks::use_navigate();

    // Determine popup variant based on rank and reward
    let _popup_variant = match user_info.rank {
        1 => PopupVariant::Champion {
            reward: user_info.reward,
        },
        2 => PopupVariant::Silver {
            reward: user_info.reward,
        },
        3 => PopupVariant::Bronze {
            reward: user_info.reward,
        },
        4..=10 if user_info.reward.is_some() && user_info.reward.unwrap() > 0 => {
            PopupVariant::TopTen {
                rank: user_info.rank,
                reward: user_info.reward,
            }
        }
        _ => PopupVariant::BetterLuck,
    };

    let popup_variant = PopupVariant::Champion {
        reward: user_info.reward,
    };

    // Get the appropriate content based on variant
    let (sunburst_svg, icon, title, reward_amount, title_color) = match &popup_variant {
        PopupVariant::Champion { reward } => (
            Some("/img/leaderboard/sunburst-gold.svg"),
            "/img/leaderboard/crown.svg",
            "You're the Champion!",
            reward.map(|r| r.to_string()),
            "text-[#FFEF00]",
        ),
        PopupVariant::Silver { reward } => (
            Some("/img/leaderboard/sunburst-silver.svg"),
            "/img/leaderboard/crown.svg",
            "Silver Star!",
            reward.map(|r| r.to_string()),
            "text-[#DCDCDC]",
        ),
        PopupVariant::Bronze { reward } => (
            Some("/img/leaderboard/sunburst-bronze.svg"),
            "/img/leaderboard/crown.svg",
            "Bronze Boss!",
            reward.map(|r| r.to_string()),
            "text-[#D99979]",
        ),
        PopupVariant::TopTen { reward, .. } => (
            None,
            "/img/leaderboard/trophy.svg",
            "Climbing the Ranks!",
            reward.map(|r| r.to_string()),
            "text-neutral-50",
        ),
        PopupVariant::BetterLuck => (
            None,
            "", // Will use emoji instead
            "Better luck next time!",
            None,
            "text-white",
        ),
    };

    view! {
        <Show when=move || show.get()>
            // Dark overlay
            <div class="fixed inset-0 bg-black/80 z-50 flex items-center justify-center p-4"
                on:click=move |_| show.set(false)
            >
                // Modal container
                <div class="relative bg-neutral-950 rounded-2xl max-w-sm w-full overflow-hidden"
                    on:click=move |e| e.stop_propagation()
                >
                    // Close button
                    <button
                        class="absolute top-4 right-4 z-20 text-white/60 hover:text-white transition-colors"
                        on:click=move |e| {
                            e.stop_propagation();
                            show.set(false);
                        }
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                        </svg>
                    </button>

                    // Sunburst background (only for top 3)
                    {sunburst_svg.map(|svg| view! {
                        <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                            <img src=svg alt="" class="w-[150%] h-auto opacity-30" />
                        </div>
                    })}

                    // Gradient background for ranks 4-10
                    {if matches!(&popup_variant, PopupVariant::TopTen { .. }) {
                        Some(view! {
                            <div
                                style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                                class="absolute z-[1] -left-1/2 bottom-1/3 size-[32rem] pointer-events-none"
                            />
                        })
                    } else {
                        None
                    }}

                    // Content
                    <div class="relative z-5 flex flex-col items-center justify-center px-6 py-10">
                        // Icon or emoji
                        {if matches!(&popup_variant, PopupVariant::BetterLuck) {
                            view! {
                                <div class="text-6xl mb-6">"😔"</div>
                            }.into_any()
                        } else {
                            let icon_size = if matches!(&popup_variant, PopupVariant::TopTen { .. }) {
                                "w-40 h-40"
                            } else {
                                "w-24 h-24"
                            };
                            view! {
                                <div class="relative mb-6">
                                    <img src=icon alt="" class=icon_size />
                                    {if matches!(&popup_variant, PopupVariant::Champion { .. } | PopupVariant::Silver { .. } | PopupVariant::Bronze { .. }) {
                                        Some(view! {
                                            <div class="absolute -top-4 -right-4 rotate-12">
                                                <img src="/img/leaderboard/crown.svg" alt="" class="w-8 h-8 transform rotate-12" />
                                            </div>
                                        })
                                    } else {
                                        None
                                    }}
                                </div>
                            }.into_any()
                        }}

                        // Reward amount badge (if applicable)
                        {reward_amount.clone().map(|amount| view! {
                            <div class="bg-[#1f1d17] border border-[rgba(255,244,86,0.43)] rounded-2xl px-4 py-2 mb-6 flex items-center gap-2">
                                <span class="text-[#ffc33a] text-3xl font-bold">{amount}</span>
                                <img src="/img/yral/yral-token.webp" alt="" class="w-8 h-8" />
                            </div>
                        })}

                        // Title
                        <h2 class=format!("text-2xl font-bold mb-4 {}", title_color)>
                            {title}
                        </h2>

                        // Description
                        <div class="text-neutral-300 text-center mb-6 leading-relaxed">
                            {match &popup_variant {
                                PopupVariant::Champion { reward } => view! {
                                    <span>
                                        "Congrats on coming 1st on the leaderboard and winning a "
                                        <span class="font-bold">{reward.unwrap_or(0)}" YRAL"</span>
                                        "! Next week's leaderboard drops soon."
                                    </span>
                                }.into_any(),
                                PopupVariant::Silver { reward } => view! {
                                    <span>
                                        "Amazing run! You've secured "
                                        <span class="font-bold">{reward.unwrap_or(0)}" YRAL"</span>
                                        " for finishing in 2nd place. You're just one step away — go for it next week!"
                                    </span>
                                }.into_any(),
                                PopupVariant::Bronze { reward } => view! {
                                    <span>
                                        "Great hustle! You've earned "
                                        <span class="font-bold">{reward.unwrap_or(0)}" YRAL"</span>
                                        " this week. Keep pushing—next week could be your golden moment!"
                                    </span>
                                }.into_any(),
                                PopupVariant::TopTen { rank, reward } => {
                                    let rank_str = match rank {
                                        4 => "4th".to_string(),
                                        5 => "5th".to_string(),
                                        6 => "6th".to_string(),
                                        7 => "7th".to_string(),
                                        8 => "8th".to_string(),
                                        9 => "9th".to_string(),
                                        10 => "10th".to_string(),
                                        _ => format!("{}th", rank)
                                    };
                                    view! {
                                        <span>
                                            "Great effort! You've finished "
                                            {rank_str}
                                            " and earned "
                                            <span class="font-bold">{reward.unwrap_or(0)}" YRAL"</span>
                                            " this week. Keep pushing—next week could be your golden moment!"
                                        </span>
                                    }.into_any()
                                },
                                PopupVariant::BetterLuck => view! {
                                    <span>
                                        "You didn't make it to the top this week. Keep earning YRAL to climb the ranks and claim your spot!"
                                    </span>
                                }.into_any()
                            }}
                        </div>

                        // Contest starts badge (placeholder for now)
                        <div class="bg-[#212121] rounded-full px-3 py-1 mb-8 flex items-center gap-2">
                            <span class="text-neutral-400 text-xs">Contest Starts on:</span>
                            <span class="text-neutral-50 text-xs font-medium">18th August, 10 AM IST</span>
                        </div>

                        // View Leaderboard button
                        <button
                            class="w-full bg-neutral-50 text-black font-bold py-3 px-6 rounded-lg hover:bg-white transition-colors"
                            on:click=move |_| {
                                show.set(false);
                                // Scroll to user's position if needed
                                // This could be enhanced to scroll to the user's rank
                            }
                        >
                            "View Leaderboard"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
