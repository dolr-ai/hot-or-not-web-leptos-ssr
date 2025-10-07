use super::{PreUploadAiView, VideoGenerationLoadingScreen};
use crate::upload::ai::components::PostUploadScreenAi;
use crate::upload::ai::server::{generate_and_upload_ai_video, ProviderInfoSerde};
use crate::upload::ai::types::VideoGenerationParams;
use crate::upload::ai::types::AI_VIDEO_PARAMS_STORE;
use codee::string::JsonSerdeCodec;
use component::notification_nudge::NotificationNudge;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_use::storage::use_local_storage;
use state::canisters::auth_state;
use utils::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps, MixpanelPostGameType};
use yral_canisters_common::CanistersAuthWire;

#[component]
pub fn UploadAiPostPage() -> impl IntoView {
    // Signal to control returning to form for re-generation
    let show_form = RwSignal::new(true);

    // Notification and modal signals
    let notification_nudge = RwSignal::new(false);
    let show_success_modal = RwSignal::new(false);

    // Loading state for different phases
    let loading_state = RwSignal::new("generating".to_string());

    // Store generated video URL for upload
    let generated_video_url = RwSignal::new(None::<String>);

    // Local storage for video generation parameters (for regeneration)
    let (stored_params, set_stored_params, _) =
        use_local_storage::<VideoGenerationParams, JsonSerdeCodec>(AI_VIDEO_PARAMS_STORE);

    // Get auth state outside actions for reuse
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    // Combined action for generation and upload - atomic server-side operation
    let generate_action: Action<VideoGenerationParams, Result<String, String>> =
        Action::new_unsync({
            move |params: &VideoGenerationParams| {
                let params = params.clone();
                let show_form = show_form;
                let notification_nudge = notification_nudge;
                let show_success_modal = show_success_modal;
                let loading_state = loading_state;
                let generated_video_url = generated_video_url;

                async move {
                    // Store provider name for tracking
                    let provider_name = params.provider.name.clone();
                    let token_type_str = format!("{:?}", params.token_type).to_lowercase();

                    // Get auth canisters and create delegated identity
                    let canisters = auth
                        .auth_cans()
                        .await
                        .map_err(|e| format!("Failed to get auth canisters: {e:?}"))?;

                    // Convert provider to serializable form
                    let provider_serde = ProviderInfoSerde {
                        id: params.provider.id.clone(),
                        name: params.provider.name.clone(),
                        is_available: params.provider.is_available,
                        supports_image: params.provider.supports_image,
                        supports_audio_input: params.provider.supports_audio_input,
                        default_duration: params.provider.default_duration,
                        default_aspect_ratio: params.provider.default_aspect_ratio.clone(),
                        default_resolution: params.provider.default_resolution.clone(),
                    };

                    // Update UI state before starting
                    show_form.try_set(false);
                    loading_state.try_set("generating".to_string());

                    // Track video generation initiated
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        MixPanelEvent::track_video_upload_initiated(
                            global,
                            false,
                            false,
                            Some("ai_video".to_string()),
                            token_type_str.clone(),
                        );
                    }

                    // Call the server function that handles everything atomically
                    let result = generate_and_upload_ai_video(
                        params.user_principal,
                        params.prompt.clone(),
                        provider_serde,
                        params.image_data.clone(),
                        params.audio_data.clone(),
                        params.token_type,
                        CanistersAuthWire::from(canisters),
                    )
                    .await;

                    // Track result
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        match &result {
                            Ok(video_url) => {
                                // Track generation success
                                MixPanelEvent::track_ai_video_generated(
                                    global.clone(),
                                    true,
                                    None,
                                    provider_name.clone(),
                                    token_type_str.clone(),
                                );

                                // Track upload success (we don't have video_uid, use video_url)
                                MixPanelEvent::track_video_upload_success(
                                    global,
                                    video_url.clone(),
                                    global_constants::CREATOR_COMMISSION_PERCENT,
                                    false,
                                    MixpanelPostGameType::HotOrNot,
                                    Some("ai_video".to_string()),
                                    token_type_str,
                                );
                            }
                            Err(error) => {
                                MixPanelEvent::track_ai_video_generated(
                                    global,
                                    false,
                                    Some(error.to_string()),
                                    provider_name,
                                    token_type_str,
                                );
                            }
                        }
                    }

                    // Handle success
                    match result {
                        Ok(video_url) => {
                            // Store the video URL for preview
                            generated_video_url.try_set(Some(video_url.clone()));
                            show_success_modal.try_set(true);
                            notification_nudge.try_set(true);
                            Ok(video_url)
                        }
                        Err(e) => {
                            leptos::logging::error!("Failed to generate and upload video: {}", e);
                            Err(e.to_string())
                        }
                    }
                }
            }
        });

    view! {
        <Title text="YRAL AI - Upload" />
        <NotificationNudge pop_up=notification_nudge />
        <div class="w-full h-full">
            <Show
                when=move || generate_action.pending().get()
                fallback=move || {
                    view! {
                        <PreUploadAiView
                            generate_action=generate_action
                            set_stored_params=set_stored_params
                        />
                    }
                }
            >
                <VideoGenerationLoadingScreen
                    prompt=stored_params.get().prompt.clone()
                    provider=stored_params.get().provider.clone()
                    loading_state=loading_state.get()
                />
            </Show>
        </div>

        // Success screen
        <Show when=move || show_success_modal.get()>
            <PostUploadScreenAi
                video_url=generated_video_url.get().unwrap_or_default()
            />
        </Show>
    }
}
