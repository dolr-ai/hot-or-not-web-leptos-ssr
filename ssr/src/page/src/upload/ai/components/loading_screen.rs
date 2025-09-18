use component::back_btn::BackButton;
use leptos::prelude::*;
use leptos_icons::*;
use videogen_common::ProviderInfo;

#[component]
pub fn VideoGenerationLoadingScreen(
    prompt: String,
    provider: ProviderInfo,
    loading_state: String,
) -> impl IntoView {
    let loading_state_1 = loading_state.clone();
    let loading_state_2 = loading_state;
    view! {
        <div class="flex flex-col bg-black min-w-dvw min-h-dvh">
            // Header with back button and title
            <div class="flex items-center justify-between p-4 pt-12">
                <div class="text-white">
                    <BackButton fallback="/upload-options".to_string() />
                </div>
                <h1 class="text-lg font-bold text-white">"Generate Video"</h1>
                <div class="w-6"></div> // Spacer for centering
            </div>

            // Prompt and model info section
            <div class="px-4 mt-4">
                <div class="max-w-md mx-auto w-full space-y-3">
                    // Prompt display
                    <div class="bg-neutral-900 rounded-lg p-4">
                        <p class="text-neutral-300 text-sm leading-relaxed">{prompt}</p>
                    </div>

                    // Provider info
                    <div class="flex items-center gap-2 px-4">
                        {
                            match provider.model_icon.clone() {
                                Some(icon_path) => view! {
                                    <img
                                        src=icon_path
                                        alt="Model icon"
                                        class="w-5 h-5"
                                    />
                                }.into_any(),
                                None => view! {
                                    <div class="w-5 h-5 bg-pink-500 rounded flex items-center justify-center">
                                        <span class="text-white font-bold text-xs">"AI"</span>
                                    </div>
                                }.into_any()
                            }
                        }
                        <span class="text-neutral-400 text-sm">{provider.name}</span>
                        <span class="text-neutral-500 text-sm">|</span>
                        <span class="text-neutral-500 text-xs">{provider.description}</span>
                    </div>
                </div>
            </div>

            // Main loading content
            <div class="flex-1 flex flex-col items-center justify-center px-4">
                <div class="flex flex-col items-center gap-8 max-w-md w-full">

                    // Progress animation circle
                    <div class="relative w-32 h-32">
                        // Outer circle (background)
                        <div class="absolute inset-0 rounded-full border-4 border-neutral-800"></div>

                        // Progress circle with gradient
                        <svg class="absolute inset-0 w-full h-full -rotate-90 animate-spin" viewBox="0 0 128 128">
                            <circle
                                cx="64"
                                cy="64"
                                r="60"
                                fill="none"
                                stroke="url(#gradient)"
                                stroke-width="4"
                                stroke-linecap="round"
                                stroke-dasharray="377"
                                stroke-dashoffset="94.25"
                                class="animate-pulse"
                            />
                            <defs>
                                <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                                    <stop offset="0%" stop-color="#FF6DC4" />
                                    <stop offset="50%" stop-color="#F7007C" />
                                    <stop offset="100%" stop-color="#690039" />
                                </linearGradient>
                            </defs>
                        </svg>

                        // Center icon
                        <div class="absolute inset-0 flex items-center justify-center">
                            <Icon
                                icon=icondata::AiPlayCircleOutlined
                                attr:class="text-white text-4xl"
                            />
                        </div>
                    </div>

                    // Status text
                    <div class="text-center">
                        <h2 class="text-xl font-bold text-white mb-2">
                            {move || {
                                match loading_state_1.as_str() {
                                    "uploading" => "Uploading video",
                                    _ => "Generating video"
                                }
                            }}
                        </h2>
                        <p class="text-sm text-neutral-400">
                            {move || {
                                match loading_state_2.as_str() {
                                    "uploading" => "Almost there...",
                                    _ => "This may take a few minutes..."
                                }
                            }}
                        </p>
                    </div>

                    // Progress dots animation
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 bg-pink-500 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
                        <div class="w-2 h-2 bg-pink-500 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
                        <div class="w-2 h-2 bg-pink-500 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
                    </div>
                </div>
            </div>
        </div>
    }
}
