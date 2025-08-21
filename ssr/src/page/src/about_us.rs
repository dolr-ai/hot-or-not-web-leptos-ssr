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
        <div class="flex flex-col items-center pt-4 pb-12 w-screen min-h-screen text-white bg-black">
            <div class="sticky top-0 z-10 w-full bg-black">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between">
                        <BackButton fallback="/menu".to_string() />
                        <div>
                            <span class="text-xl font-bold">About Us</span>
                        </div>
                        <div></div>
                    </div>
                </TitleText>
            </div>

            <div class="flex overflow-hidden overflow-y-auto flex-col px-8 mx-auto mt-2 w-full max-w-5xl h-full md:px-16">

                <div class="mb-6 text-sm text-left whitespace-pre-line md:text-lg md:text-center">
                    {"Yral is a short video-sharing platform built on the Internet Computer Protocol (ICP) blockchain, powered by Rust. The platform merges social media entertainment with user monetization, letting users earn YRAL tokens by interacting with content. We aim to create a social platform where users receive financial rewards for their engagement. Through various skill-based games, users can earn rewards while engaging with creators' content."}
                </div>

                <div class="mb-6 text-sm text-left whitespace-pre-line md:text-lg md:text-center">
                    {"Most Yral data is stored on the blockchain, except for videos and profile pictures which are hosted on Cloudflare. As technology advances, we plan to move all storage onto the blockchain. Yral tackles the common problems of monetization and centralization found in traditional social media by creating a fair and transparent system."}
                </div>

                <div class="mb-6 text-sm text-left whitespace-pre-line md:text-lg md:text-center">
                    {"Users can upload 60-second videos, interact with content, personalize their profiles, grow their communities, and enjoy customized content feeds. Using blockchain technology, Yral ensures users maintain control over their data, supporting Web3 principles of privacy and data ownership."}
                </div>

                <div class="mb-8 text-sm text-left whitespace-pre-line md:text-lg md:text-center">
                    {"Yral is operated by HotorNot (HON) GmbH."}
                </div>

                <div class="flex flex-col mb-12 space-y-4">
                    <div class="mb-6 text-lg font-semibold md:text-xl md:text-center">
                        Our Leadership
                    </div>

                    <div class="flex flex-col gap-4 md:flex-row">
                        <div class="flex-1 p-4 rounded-lg bg-neutral-900">
                            <div class="text-base font-semibold md:text-lg">Rishi Chadha</div>
                            <div class="text-gray-400">CEO & Co-Founder</div>
                            <div class="mt-2 text-sm md:text-base">
                                A serial entrepreneur with global experience across 35+ countries, leading our vision for decentralized social media.
                            </div>
                        </div>

                        <div class="flex-1 p-4 rounded-lg bg-neutral-900">
                            <div class="text-base font-semibold md:text-lg">Saikat Das</div>
                            <div class="text-gray-400">CTO & Co-Founder</div>
                            <div class="mt-2 text-sm md:text-base">
                                Tech innovator specializing in Rust programming, blockchain, and AI, driving our technological advancement.
                            </div>
                        </div>

                        <div class="flex-1 p-4 rounded-lg bg-neutral-900">
                            <div class="text-base font-semibold md:text-lg">Utkarsh Goyal</div>
                            <div class="text-gray-400">CFO & Co-Founder</div>
                            <div class="mt-2 text-sm md:text-base">
                                Financial strategist with an MBA, overseeing operations and ensuring sustainable growth.
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
