use super::{PreUploadAiView, VideoGenerationLoadingScreen};
use crate::upload::ai::components::PostUploadScreenAi;
use crate::upload::ai::helpers::{create_video_request_v2, get_auth_canisters};
use crate::upload::ai::server::generate_and_upload_video;
use crate::upload::ai::types::VideoGenerationParams;
use crate::upload::ai::types::AI_VIDEO_PARAMS_STORE;
use auth::delegate_short_lived_identity;
use codee::string::JsonSerdeCodec;
use component::notification_nudge::NotificationNudge;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_use::storage::use_local_storage;
use state::canisters::auth_state;
use utils::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps};

#[component]
pub fn UploadAiPostPage() -> impl IntoView {
    // Signal to control returning to form for re-generation
    let show_form = RwSignal::new(true);

    // Notification and modal signals
    let notification_nudge = RwSignal::new(false);
    let show_success_modal = RwSignal::new(false);

    // Store generated video UID after complete flow
    let video_uid = RwSignal::new(None::<String>);

    // Local storage for video generation parameters (for regeneration)
    let (stored_params, set_stored_params, _) =
        use_local_storage::<VideoGenerationParams, JsonSerdeCodec>(AI_VIDEO_PARAMS_STORE);

    // Get auth state outside actions for reuse
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    // Video generation action - now calls server function that handles entire flow atomically
    let generate_action: Action<VideoGenerationParams, Result<String, String>> =
        Action::new_unsync({
            move |params: &VideoGenerationParams| {
                let params = params.clone();
                let show_form = show_form;
                let notification_nudge = notification_nudge;
                let show_success_modal = show_success_modal;

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

                    // Track video upload initiated
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        MixPanelEvent::track_video_upload_initiated(
                            global,
                            false, // caption_added
                            false, // hashtags_added
                            Some("ai_video".to_string()),
                            format!("{:?}", params.token_type).to_lowercase(),
                        );
                    }

                    // Call server function that handles generation + polling + upload atomically
                    let result = generate_and_upload_video(
                        request,
                        delegated_identity,
                        vec![],         // hashtags
                        "".to_string(), // description
                        false,          // is_nsfw
                        false,          // enable_hot_or_not
                    )
                    .await
                    .map_err(|e| e.to_string());

                    // Track video generation result
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        match &result {
                            Ok(_video_uid) => {
                                MixPanelEvent::track_ai_video_generated(
                                    global,
                                    true,
                                    None,
                                    provider_name.clone(),
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

                    // Update UI state on success
                    if let Ok(uid) = &result {
                        show_form.set(false);
                        notification_nudge.set(false);
                        show_success_modal.set(true);
                        video_uid.set(Some(uid.clone()));
                    }

                    result
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
                    loading_state="processing".to_string()
                />
            </Show>
        </div>

        // Success screen
        <Show when=move || show_success_modal.get()>
            <PostUploadScreenAi
                video_url=video_uid.get()
                    .map(|uid| utils::stream_url(&uid))
                    .unwrap_or_default()
            />
        </Show>
    }
}
