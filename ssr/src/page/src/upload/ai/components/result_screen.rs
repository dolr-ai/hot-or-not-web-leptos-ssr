use crate::upload::ai::types::VideoGenerationParams;
use component::{back_btn::BackButton, buttons::GradientButton};
use leptos::prelude::*;
use leptos_icons::*;

#[component]
pub fn VideoResultScreen(
    video_url: String,
    on_upload: impl Fn() + 'static,
    on_regenerate: impl Fn() + 'static,
) -> impl IntoView {
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

            // Main video content
            <div class="flex-1 flex flex-col px-4 py-6 max-w-md mx-auto w-full">
                <div class="flex flex-col gap-6">

                    // Video preview section
                    <div class="w-full">
                        <video
                            class="w-full rounded-lg bg-neutral-900 aspect-video"
                            controls=true
                            autoplay=true
                            preload="metadata"
                            src=video_url.clone()
                        >
                            <p class="text-white p-4">"Your browser doesn't support video playback."</p>
                        </video>
                    </div>

                    // Status text
                    <div class="text-center">
                        <h2 class="text-xl font-bold text-white mb-2">"Video generated successfully!"</h2>
                        <p class="text-sm text-neutral-400">"Your AI video is ready. You can re-generate or upload it."</p>
                    </div>

                    // Action buttons
                    <div class="flex flex-col gap-3 mt-4">

                        // Re-generate button
                        <button
                            class="w-full h-12 px-5 py-3 rounded-lg border-2 border-neutral-600 bg-transparent text-white font-bold hover:border-neutral-500 transition-colors flex items-center justify-center gap-2"
                            on:click=move |_| {
                                on_regenerate();
                            }
                        >
                            <Icon icon=icondata::AiReloadOutlined attr:class="text-lg" />
                            "Re-generate"
                        </button>

                        // Upload button (primary action)
                        <GradientButton
                            on_click=move || {
                                on_upload();
                            }
                            classes="w-full h-12 rounded-lg font-bold".to_string()
                            disabled=Signal::derive(|| false)
                        >
                            <div class="flex items-center justify-center gap-2">
                                <Icon icon=icondata::AiUploadOutlined attr:class="text-lg" />
                                "Upload"
                            </div>
                        </GradientButton>
                    </div>

                    // Video info (optional)
                    <div class="mt-6 p-4 bg-neutral-900 rounded-lg">
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-neutral-400">"Duration:"</span>
                            <span class="text-white">"Auto-detected"</span>
                        </div>
                        <div class="flex items-center justify-between text-sm mt-2">
                            <span class="text-neutral-400">"Format:"</span>
                            <span class="text-white">"MP4"</span>
                        </div>
                        <div class="flex items-center justify-between text-sm mt-2">
                            <span class="text-neutral-400">"Quality:"</span>
                            <span class="text-white">"HD"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
