use super::{PreUploadAiView, VideoGenerationLoadingScreen};
use crate::upload::ai::components::PostUploadScreenAi;
use crate::upload::ai::helpers::{
    create_video_request_v2, execute_video_generation_with_identity_v2, get_auth_canisters,
};
use crate::upload::ai::server::upload_ai_video_from_url;
use crate::upload::ai::types::VideoGenerationParams;
use crate::upload::ai::types::{UploadActionParams, AI_VIDEO_PARAMS_STORE};
use auth::delegate_short_lived_identity;
use codee::string::JsonSerdeCodec;
use component::notification_nudge::NotificationNudge;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_use::storage::use_local_storage;
use state::canisters::auth_state;
use utils::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps, MixpanelPostGameType};

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

    // Video generation action - cleaned up with helper functions
    let generate_action: Action<VideoGenerationParams, Result<String, String>> =
        Action::new_unsync({
            move |params: &VideoGenerationParams| {
                let params = params.clone();
                let show_form = show_form;

                async move {
                    // Store provider name for tracking
                    let provider_name = params.provider.name.clone();

                    // Get auth canisters
                    let canisters = get_auth_canisters(&auth).await?;

                    // Get identity from canisters
                    let identity = canisters.identity();

                    // Always use delegated identity flow for all token types
                    let request = create_video_request_v2(
                        params.user_principal,
                        params.prompt.clone(),
                        &params.provider,
                        params.image_data.clone(),
                        params.audio_data.clone(),
                        params.token_type,
                    )
                    .map_err(|e| e.to_string())?;

                    let delegated_identity = delegate_short_lived_identity(identity);
                    let result = execute_video_generation_with_identity_v2(
                        request,
                        delegated_identity,
                        &canisters,
                    )
                    .await;

                    // Track video generation result
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        match &result {
                            Ok(_) => {
                                MixPanelEvent::track_ai_video_generated(
                                    global,
                                    true,
                                    None,
                                    provider_name,
                                    format!("{:?}", params.token_type).to_lowercase(),
                                );
                            }
                            Err(error) => {
                                MixPanelEvent::track_ai_video_generated(
                                    global,
                                    false,
                                    Some(error.clone()),
                                    provider_name,
                                    format!("{:?}", params.token_type).to_lowercase(),
                                );
                            }
                        }
                    }

                    // Update UI state on success and trigger upload
                    if let Ok(video_url) = &result {
                        show_form.set(false);
                        generated_video_url.set(Some(video_url.clone()));
                    }

                    result
                }
            }
        });

    // Upload action - handles server-side video download and upload
    let upload_action: Action<UploadActionParams, Result<String, String>> = Action::new_unsync({
        move |params: &UploadActionParams| {
            let params = params.clone();
            let notification_nudge = notification_nudge;
            let show_success_modal = show_success_modal;
            let loading_state = loading_state;
            async move {
                // Update loading state to uploading
                loading_state.set("uploading".to_string());
                // Show notification nudge when starting upload
                notification_nudge.set(true);

                // Track video upload initiated for AI video
                if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                    MixPanelEvent::track_video_upload_initiated(
                        global,
                        false, // caption_added - we're not using captions for AI videos
                        false, // hashtags_added - we're not using hashtags for AI videos
                        Some("ai_video".to_string()),
                        format!("{:?}", params.token_type).to_lowercase(),
                    );
                }

                // Get delegated identity within the Action
                match auth.auth_cans().await {
                    Ok(canisters) => {
                        let id = canisters.identity();
                        let delegated_identity = delegate_short_lived_identity(id);

                        // Call server function with delegated identity
                        match upload_ai_video_from_url(
                            params.video_url,
                            vec![],
                            "".to_string(),
                            delegated_identity,
                            false, // is_nsfw
                            false, // enable_hot_or_not
                        )
                        .await
                        {
                            Ok(video_uid) => {
                                leptos::logging::log!(
                                    "Video uploaded successfully with UID: {}",
                                    video_uid
                                );

                                // Track video upload success for AI video
                                if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                                    MixPanelEvent::track_video_upload_success(
                                        global,
                                        video_uid.clone(),
                                        global_constants::CREATOR_COMMISSION_PERCENT,
                                        false, // is_game_enabled - AI videos don't have game enabled
                                        MixpanelPostGameType::HotOrNot,
                                        Some("ai_video".to_string()),
                                        format!("{:?}", params.token_type).to_lowercase(),
                                    );
                                }

                                // Show success modal
                                show_success_modal.set(true);
                                Ok(video_uid)
                            }
                            Err(e) => {
                                leptos::logging::error!("Failed to upload video: {}", e);

                                Err(format!("Upload failed: {e}"))
                            }
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to get auth canisters: {:?}", e);
                        Err(format!("Auth failed: {e:?}"))
                    }
                }
            }
        }
    });

    // Effect to trigger upload after generation
    Effect::new(move |_| {
        if let Some(video_url) = generated_video_url.get() {
            if !upload_action.pending().get() && !show_success_modal.get() {
                leptos::logging::log!("Auto-uploading generated video: {}", video_url);
                let token_type = stored_params.get().token_type;
                upload_action.dispatch(UploadActionParams {
                    video_url,
                    token_type,
                });
            }
        }
    });

    view! {
        <Title text="YRAL AI - Upload" />
        <NotificationNudge pop_up=notification_nudge />
        <div class="w-full h-full">
            <Show
                when=move || generate_action.pending().get() || upload_action.pending().get()
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
