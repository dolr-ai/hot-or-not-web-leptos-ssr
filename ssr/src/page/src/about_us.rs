use leptos::prelude::*;
use leptos_meta::*;

use component::{back_btn::BackButton, title::TitleText};
use state::app_state::AppState;

#[component]
pub fn AboutUs() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - About Us";
    view! {
        <Title text=page_title />
        <div class="w-screen min-h-screen bg-black pt-4 pb-12 text-white flex flex-col items-center">
            <div class="sticky top-0 z-10 bg-black w-full">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between">
                        <BackButton fallback="/menu".to_string() />
                        <div>
                            <span class="font-bold text-xl">About Us</span>
                        </div>
                        <div></div>
                    </div>
                </TitleText>
            </div>

            <div class="px-8 md:px-16 flex h-full w-full flex-col overflow-hidden overflow-y-auto max-w-5xl mx-auto mt-2">

                <div class="text-sm md:text-lg whitespace-pre-line text-left md:text-center mb-6">
                    {"Yral is a short video-sharing platform built on the Internet Computer Protocol (ICP) blockchain, powered by Rust. The platform merges social media entertainment with user monetization, letting users earn COYN tokens by interacting with content. We aim to create a social platform where users receive financial rewards for their engagement. Through various skill-based games, users can earn rewards while engaging with creators' content."}
                </div>

                <div class="text-sm md:text-lg whitespace-pre-line text-left md:text-center mb-6">
                    {"Most Yral data is stored on the blockchain, except for videos and profile pictures which are hosted on Cloudflare. As technology advances, we plan to move all storage onto the blockchain. Yral tackles the common problems of monetization and centralization found in traditional social media by creating a fair and transparent system."}
                </div>

                <div class="text-sm md:text-lg whitespace-pre-line text-left md:text-center mb-6">
                    {"Users can upload 60-second videos, interact with content, personalize their profiles, grow their communities, and enjoy customized content feeds. Using blockchain technology, Yral ensures users maintain control over their data, supporting Web3 principles of privacy and data ownership."}
                </div>

                <div class="text-sm md:text-lg whitespace-pre-line text-left md:text-center mb-8">
                    {"Yral is operated by HotorNot (HON) GmbH."}
                </div>

                <div class="flex flex-col space-y-4 mb-12">
                    <div class="text-lg md:text-xl font-semibold md:text-center mb-6">Our Leadership</div>

                    <div class="flex flex-col md:flex-row gap-4">
                        <div class="bg-neutral-900 rounded-lg p-4 flex-1">
                            <div class="font-semibold text-base md:text-lg">Rishi Chadha</div>
                            <div class="text-gray-400">CEO & Co-Founder</div>
                            <div class="mt-2 text-sm md:text-base">A serial entrepreneur with global experience across 35+ countries, leading our vision for decentralized social media.</div>
                        </div>

                        <div class="bg-neutral-900 rounded-lg p-4 flex-1">
                            <div class="font-semibold text-base md:text-lg">Saikat Das</div>
                            <div class="text-gray-400">CTO & Co-Founder</div>
                            <div class="mt-2 text-sm md:text-base">Tech innovator specializing in Rust programming, blockchain, and AI, driving our technological advancement.</div>
                        </div>

                        <div class="bg-neutral-900 rounded-lg p-4 flex-1">
                            <div class="font-semibold text-base md:text-lg">Utkarsh Goyal</div>
                            <div class="text-gray-400">CFO & Co-Founder</div>
                            <div class="mt-2 text-sm md:text-base">Financial strategist with an MBA, overseeing operations and ensuring sustainable growth.</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
