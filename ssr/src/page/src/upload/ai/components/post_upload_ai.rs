use component::buttons::HighlightedLinkButton;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn PostUploadScreenAi(video_url: String) -> impl IntoView {
    view! {
        <div
            style="background: radial-gradient(circle, rgba(0,0,0,0) 0%, rgba(0,0,0,0) 75%, rgba(50,0,28,0.5) 100%);"
            class="flex fixed top-0 right-0 bottom-0 left-0 z-50 justify-center items-center w-screen h-screen"
        >
            // Background image with fade effect
            <img
                alt="bg"
                src="/img/airdrop/bg.webp"
                class="object-cover absolute inset-0 w-full h-full z-25 fade-in"
            />

            // Content container
            <div class="relative z-50 flex flex-col items-center w-full max-w-[390px] h-full">
                // Close button
                <div class="absolute top-[48px] right-[20px] cursor-pointer z-50">
                    <a
                        href = "/".to_string()
                        class="flex items-center justify-center w-8 h-8 text-white hover:opacity-80 transition-opacity"
                    >
                        <Icon
                            icon=icondata::AiCloseOutlined
                            attr:class="w-full h-full"
                        />
                    </a>
                </div>

                // Main content area - centered
                <div class="flex flex-col items-center justify-center flex-1 w-full px-4">
                    // Video preview
                    <div class="w-full max-w-[358px] h-[250px] mb-6">
                        <div class="relative w-full h-full bg-neutral-950 rounded-lg border border-neutral-800 overflow-hidden shadow-2xl">
                            <video
                                class="w-full h-full object-cover"
                                controls=true
                                autoplay=true
                                loop=true
                                muted=true
                                preload="metadata"
                                src=video_url.clone()
                            >
                                <p class="text-white p-4">"Your browser doesn't support video playback."</p>
                            </video>
                        </div>
                    </div>

                    // Text content and button
                    <div class="w-full max-w-[321px] flex flex-col items-center gap-[30px]">
                    <div class="flex flex-col items-center gap-2.5 text-center">
                        <h1 class="font-semibold text-[20px] text-neutral-50 font-['Kumbh_Sans']">
                            "AI Video generated Successfully!"
                        </h1>
                        <p class="text-[16px] text-neutral-400 font-['Kumbh_Sans'] font-normal w-[303px] leading-[1.4]">
                            "Your video is being processed and will appear in \"Your Videos\" under your profile shortly. Happy scrolling!"
                        </p>
                    </div>

                        // Done button
                        <HighlightedLinkButton
                            alt_style=false
                            disabled=false
                            classes="w-full h-[45px] px-5 py-3".to_string()
                            href="/".to_string()
                        >
                            "Done"
                        </HighlightedLinkButton>
                    </div>
                </div>
            </div>
        </div>
    }
}
