use super::types::{TournamentInfo, UserInfo};
use crate::buttons::HighlightedButton;
use leptos::prelude::*;
use utils::timezone::format_tournament_date_with_fallback;

fn format_with_commas(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

#[derive(Clone, Debug)]
enum PopupVariant {
    Champion { reward: Option<u32> },
    Silver { reward: Option<u32> },
    Bronze { reward: Option<u32> },
    TopTen { rank: u32, reward: Option<u32> },
    BetterLuck,
}

#[component]
pub fn TournamentCompletionPopup(
    show: RwSignal<bool>,
    user_info: UserInfo,
    last_tournament_id: String,
    #[prop(optional)] upcoming_tournament: TournamentInfo,
) -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();

    // Determine popup variant based on rank and reward
    let popup_variant = match user_info.rank {
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

    // Get the appropriate content based on variant
    let (sunburst_svg, icon, title, reward_amount, title_color) = match &popup_variant {
        PopupVariant::Champion { reward } => (
            Some("/img/leaderboard/sunburst-gold.svg"),
            "/img/leaderboard/crown-gold.svg",
            "You're the Champion!",
            reward.map(|r| r.to_string()),
            "text-[#FFEF00]",
        ),
        PopupVariant::Silver { reward } => (
            Some("/img/leaderboard/sunburst-silver.svg"),
            "/img/leaderboard/crown-silver.svg",
            "Silver Star!",
            reward.map(|r| r.to_string()),
            "text-[#BFBFBF]",
        ),
        PopupVariant::Bronze { reward } => (
            Some("/img/leaderboard/sunburst-bronze.svg"),
            "/img/leaderboard/crown-bronze.svg",
            "Bronze Boss!",
            reward.map(|r| r.to_string()),
            "text-[#D99979]",
        ),
        PopupVariant::TopTen { reward, .. } => (
            None,
            "/img/leaderboard/trophy.svg",
            "Climbing the Ranks!",
            reward.map(|r| r.to_string()),
            "text-[#FAFAFA]",
        ),
        PopupVariant::BetterLuck => (
            None,
            "", // Will use emoji instead
            "Better luck next time!",
            None,
            "text-[#FAFAFA]",
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

                    // Sunburst background will be rendered inside the reward box container for top 3

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
                    <div class="relative z-10 flex flex-col items-center justify-center px-6 pt-16 pb-8">
                        // Main element - either reward badge for top 3, icon for 4-10, or emoji for 11+
                        {if matches!(&popup_variant, PopupVariant::Champion { .. } | PopupVariant::Silver { .. } | PopupVariant::Bronze { .. }) {
                            // For top 3: Always show reward badge with crown overlay
                            let (border_color, bg_color, text_color) = match &popup_variant {
                                PopupVariant::Champion { .. } => (
                                    "border-[rgba(255,244,86,0.43)]",
                                    "bg-[#1f1d17]",
                                    "text-[#FFC33A]",
                                ),
                                PopupVariant::Silver { .. } => (
                                    "border-[rgba(255,255,253,0.43)]",
                                    "bg-[#1a1a18]",
                                    "text-[#FFF9EB]",
                                ),
                                PopupVariant::Bronze { .. } => (
                                    "border-[rgba(217,153,121,0.43)]",
                                    "bg-[#1a1715]",
                                    "text-[#FFB380]",
                                ),
                                _ => (
                                    "border-[rgba(255,244,86,0.43)]",
                                    "bg-[#1f1d17]",
                                    "text-[#FFC33A]",
                                )
                            };
                            let reward_value = match &popup_variant {
                                PopupVariant::Champion { reward } => reward.unwrap_or(0),
                                PopupVariant::Silver { reward } => reward.unwrap_or(0),
                                PopupVariant::Bronze { reward } => reward.unwrap_or(0),
                                _ => 0
                            };
                            view! {
                                <div class="relative mb-10">
                                    // Sunburst background centered on the reward box
                                    {sunburst_svg.map(|svg| view! {
                                        <div class="absolute inset-0 flex items-center justify-center pointer-events-none overflow-hidden"
                                             style="top: -255px; left: -160px; width: 580px; height: 580px;">
                                            <img src=svg alt="" class="w-full h-full opacity-50" style="transform: rotate(180deg);" />
                                        </div>
                                    })}
                                    // Reward badge box with crown
                                    <div class="relative mt-12 w-[260px] h-[70px]">
                                        // Crown overlay positioned at top-right corner of box
                                        <div class="absolute z-20" style="top: -45px; right: -36px; transform: rotate(15.625deg);">
                                            <img src=icon alt="" class="w-[101px] h-[101px]" />
                                        </div>
                                        // Box content
                                        <div class=format!("relative z-10 flex flex-row items-center justify-center w-full h-full p-[10px] gap-2.5 {} border {} rounded-[20px]", bg_color, border_color)>
                                            <span class=format!("{} text-5xl font-bold tracking-[-1.44px]", text_color)>
                                                {format_with_commas(reward_value)}
                                            </span>
                                            <img src="/img/yral/yral-token.webp" alt="" class="w-12 h-[50px]" />
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        } else if matches!(&popup_variant, PopupVariant::TopTen { .. }) {
                            // For 4-10: Show trophy icon with reward badge below
                            view! {
                                <>
                                    <div class="relative mb-6">
                                        <img src=icon alt="" class="w-40 h-40" />
                                    </div>
                                    {reward_amount.clone().map(|amount| view! {
                                        <div class="bg-[#1f1d17] border border-[rgba(255,244,86,0.43)] rounded-2xl px-4 py-2 mb-6 flex items-center gap-2">
                                            <span class="text-[#ffc33a] text-3xl font-bold">{amount}</span>
                                            <img src="/img/yral/yral-token.webp" alt="" class="w-8 h-8" />
                                        </div>
                                    })}
                                </>
                            }.into_any()
                        } else {
                            // For 11+: Show emoji
                            view! {
                                <div class="text-6xl mb-6">"😔"</div>
                            }.into_any()
                        }}

                        // Title
                        <h2 class=format!("text-2xl font-semibold mb-6 {}", title_color)>
                            {title}
                        </h2>

                        // Description
                        <div class="text-[#A3A3A3] text-base text-center mb-6 leading-relaxed">
                            {match &popup_variant {
                                PopupVariant::Champion { reward } => view! {
                                    <span>
                                        "Congrats on coming 1st place on the leaderboard."
                                    </span>
                                }.into_any(),
                                PopupVariant::Silver { reward } => view! {
                                    <span>
                                        "Congrats on coming 2nd place on the leaderboard."
                                    </span>
                                }.into_any(),
                                PopupVariant::Bronze { reward } => view! {
                                    <span>
                                        "Congrats on coming 3rd place on the leaderboard."
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
                                        _ => format!("{rank}th")
                                    };
                                    view! {
                                        <span>
                                            "Great effort! You've finished "
                                            {rank_str}
                                            " and earned "
                                            <span class="font-semibold">
                                                {format_with_commas(reward.unwrap_or(0))}
                                                " YRAL"
                                            </span>
                                        </span>
                                    }.into_any()
                                },
                                PopupVariant::BetterLuck => view! {
                                    <span>
                                        "Keep playing more to climb the ranks and claim your spot!"
                                    </span>
                                }.into_any()
                            }}
                        </div>

                        // View Leaderboard button
                        <div class="w-full">
                            <HighlightedButton
                                on_click={
                                    let tournament_id = last_tournament_id.clone();
                                    let nav = navigate.clone();
                                    move || {
                                        show.set(false);
                                        nav(&format!("/leaderboard/tournament/{}", tournament_id), Default::default());
                                    }
                                }
                                classes="w-full".to_string()
                            >
                                "View Leaderboard"
                            </HighlightedButton>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
