use leptos::prelude::*;
use super::types::LeaderboardEntry;
use yral_canisters_common::utils::profile::ProfileDetails;

#[component]
pub fn TournamentPodium(
    winners: Vec<LeaderboardEntry>,
    winner_profiles: Vec<Option<ProfileDetails>>,
) -> impl IntoView {
    // Ensure we have exactly 3 winners
    if winners.len() < 3 {
        return view! { <div></div> }.into_any();
    }
    
    // Extract winners in order: 1st, 2nd, 3rd
    let first = winners.get(0).cloned();
    let second = winners.get(1).cloned();
    let third = winners.get(2).cloned();
    
    let first_profile = winner_profiles.get(0).and_then(|p| p.clone());
    let second_profile = winner_profiles.get(1).and_then(|p| p.clone());
    let third_profile = winner_profiles.get(2).and_then(|p| p.clone());
    
    view! {
        <div class="relative w-full pt-16 pb-8 mb-8">
            // Sunburst background effect (exact Figma design)
            <div class="absolute inset-0 overflow-hidden pointer-events-none">
                <div class="absolute top-[-63px] left-1/2 transform -translate-x-1/2 w-[471px] h-[450px]">
                    <img 
                        src="/img/leaderboard/sunburst.svg"
                        alt=""
                        class="w-full h-full object-contain"
                        style="transform: rotate(180deg)"
                    />
                </div>
            </div>
            
            // Podium container
            <div class="relative flex items-end justify-center gap-[42px] px-4 max-w-md mx-auto mt-8">
                // 2nd Place (Left)
                {second.map(|winner| {
                    let profile_pic = second_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));
                    
                    view! {
                        <div class="flex flex-col items-center">
                            // Silver trophy
                            <div class="relative w-[46px] h-[135px] mb-[-50px] z-10">
                                <img 
                                    src="/img/leaderboard/trophy-silver.svg"
                                    alt="Silver trophy"
                                    class="w-full h-full object-contain"
                                />
                            </div>
                            
                            // Profile picture
                            <div class="relative mb-2 z-20">
                                <img 
                                    src=profile_pic
                                    alt=format!("{}'s profile", winner.username)
                                    class="w-[50px] h-[50px] rounded-full object-cover border-[3px] border-[#2F2F30]"
                                />
                            </div>
                            
                            // Username and reward
                            <div class="text-center mt-2">
                                <div class="text-sm font-medium text-neutral-400 mb-1">
                                    "@"{winner.username}
                                </div>
                                <div class="flex items-center justify-center gap-1">
                                    <span class="text-sm font-bold text-white">
                                        {winner.reward.unwrap_or(0)}
                                    </span>
                                    <img src="/img/yral/yral-token.webp" alt="" class="w-[17px] h-[18px]" />
                                </div>
                            </div>
                        </div>
                    }
                })}
                
                // 1st Place (Center, Elevated)
                {first.map(|winner| {
                    let profile_pic = first_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));
                    
                    view! {
                        <div class="flex flex-col items-center -mt-8">
                            // Gold trophy
                            <div class="relative w-[67px] h-[146px] mb-[-60px] z-10">
                                <img 
                                    src="/img/leaderboard/trophy-gold.svg"
                                    alt="Gold trophy"
                                    class="w-full h-full object-contain"
                                />
                            </div>
                            
                            // Profile picture
                            <div class="relative mb-2 z-20">
                                <img 
                                    src=profile_pic
                                    alt=format!("{}'s profile", winner.username)
                                    class="w-[63px] h-[63px] rounded-full object-cover border-4 border-[#BF760B]"
                                />
                            </div>
                            
                            // Username and reward
                            <div class="text-center mt-2">
                                <div class="text-sm font-medium text-neutral-400 mb-1">
                                    "@"{winner.username}
                                </div>
                                <div class="flex items-center justify-center gap-1">
                                    <span class="text-sm font-bold text-white">
                                        {winner.reward.unwrap_or(0)}
                                    </span>
                                    <img src="/img/yral/yral-token.webp" alt="" class="w-[17px] h-[18px]" />
                                </div>
                            </div>
                        </div>
                    }
                })}
                
                // 3rd Place (Right)
                {third.map(|winner| {
                    let profile_pic = third_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));
                    
                    view! {
                        <div class="flex flex-col items-center">
                            // Bronze trophy
                            <div class="relative w-[45px] h-[91px] mb-[-40px] z-10">
                                <img 
                                    src="/img/leaderboard/trophy-bronze.svg"
                                    alt="Bronze trophy"
                                    class="w-full h-full object-contain"
                                />
                            </div>
                            
                            // Profile picture
                            <div class="relative mb-2 z-20">
                                <img 
                                    src=profile_pic
                                    alt=format!("{}'s profile", winner.username)
                                    class="w-12 h-12 rounded-full object-cover border-[2.5px] border-[#6D4C35]"
                                />
                            </div>
                            
                            // Username and reward
                            <div class="text-center mt-2">
                                <div class="text-sm font-medium text-neutral-400 mb-1">
                                    "@"{winner.username}
                                </div>
                                <div class="flex items-center justify-center gap-1">
                                    <span class="text-sm font-bold text-white">
                                        {winner.reward.unwrap_or(0)}
                                    </span>
                                    <img src="/img/yral/yral-token.webp" alt="" class="w-[17px] h-[18px]" />
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
    format!("https://api.dicebear.com/7.x/identicon/svg?seed={}", username)
}