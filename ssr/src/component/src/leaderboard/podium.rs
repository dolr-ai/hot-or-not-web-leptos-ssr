use super::types::LeaderboardEntry;
use leptos::prelude::*;
use yral_canisters_common::utils::profile::ProfileDetails;

#[component]
pub fn TournamentPodium(
    winners: Vec<LeaderboardEntry>,
    winner_profiles: Vec<Option<ProfileDetails>>,
    #[prop(optional)] prize_token: String,
) -> impl IntoView {
    // Ensure we have exactly 3 winners
    if winners.len() < 3 {
        return view! { <div></div> }.into_any();
    }

    // Extract winners in order: 1st, 2nd, 3rd
    let first = winners.first().cloned();
    let second = winners.get(1).cloned();
    let third = winners.get(2).cloned();

    let first_profile = winner_profiles.first().and_then(|p| p.clone());
    let second_profile = winner_profiles.get(1).and_then(|p| p.clone());
    let third_profile = winner_profiles.get(2).and_then(|p| p.clone());

    view! {
        <div class="relative w-full h-[380px] mb-8">
            // Sunburst background effect centered on 1st place avatar
            <div class="absolute inset-0 overflow-hidden pointer-events-none">
                // Position sunburst so its center aligns with 1st place avatar at top: 125px + 31.5px (half of 63px avatar)
                <div class="absolute left-1/2 transform -translate-x-1/2" style="top: -18px; width: 390px; height: 387px;">
                    <img
                        src="/img/leaderboard/sunburst.svg"
                        alt=""
                        class="w-full h-full object-contain"
                    />
                </div>
            </div>

            // Podium container with absolute positioning
            <div class="relative w-full h-full">
                // Trophy bases layer (positioned at top: 145px)
                <div class="absolute left-1/2 transform -translate-x-1/2 top-[145px] flex items-end justify-center gap-[42px]">
                    // Silver trophy base
                    <div class="w-[45.75px] h-[135.317px]">
                        <img
                            src="/img/leaderboard/trophy-silver.svg"
                            alt="Silver trophy"
                            class="w-full h-full object-contain"
                        />
                    </div>

                    // Gold trophy base
                    <div class="w-[66.75px] h-[146.266px]">
                        <img
                            src="/img/leaderboard/trophy-gold.svg"
                            alt="Gold trophy"
                            class="w-full h-full object-contain"
                        />
                    </div>

                    // Bronze trophy base
                    <div class="w-[45px] h-[91px]">
                        <img
                            src="/img/leaderboard/trophy-bronze.svg"
                            alt="Bronze trophy"
                            class="w-full h-full object-contain"
                        />
                    </div>
                </div>

                // 2nd Place Avatar (Left) - positioned on top of silver trophy
                {second.clone().map(|winner| {
                    let profile_pic = second_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));

                    view! {
                        <div class="absolute left-1/2 transform -translate-x-1/2" style="left: calc(50% - 97px); top: 166px;">
                            // Profile picture overlaying trophy head
                            <img
                                src=profile_pic
                                alt=format!("{}'s profile", winner.username)
                                class="w-[50px] h-[50px] rounded-full object-cover border-[3px] border-[#2F2F30]"
                            />
                        </div>
                    }
                })}

                // 1st Place Avatar (Center) - positioned on top of gold trophy
                {first.clone().map(|winner| {
                    let profile_pic = first_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));

                    view! {
                        <div class="absolute left-1/2 transform -translate-x-1/2 top-[125px]">
                            // Profile picture overlaying trophy head
                            <img
                                src=profile_pic
                                alt=format!("{}'s profile", winner.username)
                                class="w-[63px] h-[63px] rounded-full object-cover border-4 border-[#BF760B]"
                            />
                        </div>
                    }
                })}

                // 3rd Place Avatar (Right) - positioned on top of bronze trophy
                {third.clone().map(|winner| {
                    let profile_pic = third_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));

                    view! {
                        <div class="absolute left-1/2 transform -translate-x-1/2" style="left: calc(50% + 99px); top: 182px;">
                            // Profile picture overlaying trophy head
                            <img
                                src=profile_pic
                                alt=format!("{}'s profile", winner.username)
                                class="w-12 h-12 rounded-full object-cover border-[2.5px] border-[#6D4C35]"
                            />
                        </div>
                    }
                })}

                // 1st Place username and reward (positioned higher)
                {first.map(|winner| {
                    view! {
                        <div class="absolute left-1/2 transform -translate-x-1/2 top-[295px]">
                            <div class="flex flex-col gap-2 items-center w-[93px]">
                                <div class="text-sm font-medium text-neutral-400 truncate w-full text-center">
                                    "@"{winner.username}
                                </div>
                                <div class="flex items-center justify-center gap-1">
                                    <span class="text-sm font-bold text-white">
                                        {if prize_token == "CKBTC" {
                                            format!("${}", winner.reward.unwrap_or(0))
                                        } else {
                                            winner.reward.unwrap_or(0).to_string()
                                        }}
                                    </span>
                                    <img src={if prize_token == "CKBTC" {
                                        "/img/hotornot/bitcoin.svg"
                                    } else {
                                        "/img/yral/yral-token.webp"
                                    }} alt="" class="w-[17px] h-[18px]" />
                                </div>
                            </div>
                        </div>
                    }
                })}

                // 2nd Place username and reward (positioned under silver trophy)
                {second.map(|winner| {
                    view! {
                        <div class="absolute left-1/2 transform -translate-x-1/2 top-[308px]" style="left: calc(50% - 97px);">
                            <div class="flex flex-col gap-2 items-center w-[93px]">
                                <div class="text-sm font-medium text-neutral-400 truncate w-full text-center">
                                    "@"{winner.username}
                                </div>
                                <div class="flex items-center justify-center gap-1">
                                    <span class="text-sm font-bold text-white">
                                        {if prize_token == "CKBTC" {
                                            format!("${}", winner.reward.unwrap_or(0))
                                        } else {
                                            winner.reward.unwrap_or(0).to_string()
                                        }}
                                    </span>
                                    <img src={if prize_token == "CKBTC" {
                                        "/img/hotornot/bitcoin.svg"
                                    } else {
                                        "/img/yral/yral-token.webp"
                                    }} alt="" class="w-[17px] h-[18px]" />
                                </div>
                            </div>
                        </div>
                    }
                })}

                // 3rd Place username and reward (positioned under bronze trophy)
                {third.map(|winner| {
                    view! {
                        <div class="absolute left-1/2 transform -translate-x-1/2 top-[308px]" style="left: calc(50% + 99px);">
                            <div class="flex flex-col gap-2 items-center w-[93px]">
                                <div class="text-sm font-medium text-neutral-400 truncate w-full text-center">
                                    "@"{winner.username}
                                </div>
                                <div class="flex items-center justify-center gap-1">
                                    <span class="text-sm font-bold text-white">
                                        {if prize_token == "CKBTC" {
                                            format!("${}", winner.reward.unwrap_or(0))
                                        } else {
                                            winner.reward.unwrap_or(0).to_string()
                                        }}
                                    </span>
                                    <img src={if prize_token == "CKBTC" {
                                        "/img/hotornot/bitcoin.svg"
                                    } else {
                                        "/img/yral/yral-token.webp"
                                    }} alt="" class="w-[17px] h-[18px]" />
                                </div>
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }.into_any()
}

// Helper function to generate a default avatar URL based on username
fn generate_default_avatar(username: &str) -> String {
    // Use a service like UI Avatars or DiceBear for generating avatars
    format!("https://api.dicebear.com/7.x/identicon/svg?seed={username}")
}
