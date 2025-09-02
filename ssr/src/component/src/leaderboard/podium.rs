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
        <div class="relative w-full py-8 mb-8">
            // Sunburst background effect (behind winner)
            <div class="absolute inset-0 overflow-hidden">
                <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-96 h-96">
                    <div class="absolute inset-0 bg-gradient-to-r from-yellow-400/10 via-yellow-500/5 to-transparent animate-pulse"
                        style="background: radial-gradient(circle, rgba(253, 191, 1, 0.15) 0%, transparent 70%)"
                    />
                </div>
            </div>
            
            // Podium container
            <div class="relative flex items-end justify-center gap-4 px-4 max-w-md mx-auto">
                // 2nd Place (Left)
                {second.map(|winner| {
                    let profile_pic = second_profile
                        .map(|p| p.profile_pic_or_random())
                        .unwrap_or_else(|| generate_default_avatar(&winner.username));
                    
                    view! {
                        <div class="flex flex-col items-center">
                            // Profile picture
                            <div class="relative mb-2">
                                <img 
                                    src=profile_pic
                                    alt=format!("{}'s profile", winner.username)
                                    class="w-16 h-16 rounded-full object-cover border-4 border-[#DCDCDC]"
                                />
                                // Silver medal emoji
                                <div class="absolute -bottom-1 -right-1 text-2xl">
                                    {"🥈"}
                                </div>
                            </div>
                            
                            // Podium base
                            <div class="bg-gradient-to-b from-[#2F2F30] via-[#FFFFFF] to-[#4B4B4B] rounded-t-lg px-4 py-3 min-w-[100px]">
                                <div class="text-center">
                                    <div class="text-3xl font-bold text-black mb-1">2</div>
                                    <div class="text-xs font-medium text-black truncate max-w-[80px]">
                                        "@"{winner.username}
                                    </div>
                                    <div class="text-xs font-bold text-black mt-1">
                                        {winner.reward.unwrap_or(0)} <span class="text-yellow-500">{"🪙"}</span>
                                    </div>
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
                            // Profile picture
                            <div class="relative mb-2">
                                <img 
                                    src=profile_pic
                                    alt=format!("{}'s profile", winner.username)
                                    class="w-20 h-20 rounded-full object-cover border-4 border-[#FDBF01] shadow-lg"
                                />
                                // Gold trophy emoji
                                <div class="absolute -bottom-1 -right-1 text-3xl">
                                    {"🏆"}
                                </div>
                            </div>
                            
                            // Podium base (taller)
                            <div class="bg-gradient-to-b from-[#BF760B] via-[#FFE89F] to-[#C38F14] rounded-t-lg px-4 py-4 min-w-[120px]">
                                <div class="text-center">
                                    <div class="text-4xl font-bold text-black mb-1">1</div>
                                    <div class="text-sm font-medium text-black truncate max-w-[100px]">
                                        "@"{winner.username}
                                    </div>
                                    <div class="text-sm font-bold text-black mt-1">
                                        {winner.reward.unwrap_or(0)} <span class="text-yellow-500">{"🪙"}</span>
                                    </div>
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
                            // Profile picture
                            <div class="relative mb-2">
                                <img 
                                    src=profile_pic
                                    alt=format!("{}'s profile", winner.username)
                                    class="w-16 h-16 rounded-full object-cover border-4 border-[#D99979]"
                                />
                                // Bronze medal emoji
                                <div class="absolute -bottom-1 -right-1 text-2xl">
                                    {"🥉"}
                                </div>
                            </div>
                            
                            // Podium base
                            <div class="bg-gradient-to-b from-[#6D4C35] via-[#DBA374] to-[#9F7753] rounded-t-lg px-4 py-2 min-w-[100px]">
                                <div class="text-center">
                                    <div class="text-3xl font-bold text-black mb-1">3</div>
                                    <div class="text-xs font-medium text-black truncate max-w-[80px]">
                                        "@"{winner.username}
                                    </div>
                                    <div class="text-xs font-bold text-black mt-1">
                                        {winner.reward.unwrap_or(0)} <span class="text-yellow-500">{"🪙"}</span>
                                    </div>
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