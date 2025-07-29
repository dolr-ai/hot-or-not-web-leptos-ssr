use super::{PreUploadAiView, VideoGenerationLoadingScreen, VideoResultScreen};
use crate::upload::ai::helpers::{
    create_and_sign_request, execute_video_generation, get_auth_canisters,
};
use crate::upload::ai::server::upload_ai_video_from_url;
use crate::upload::ai::types::VideoGenerationParams;
use crate::upload::ai::types::{UploadActionParams, AI_VIDEO_PARAMS_STORE};
use crate::upload::PostUploadScreen;
use auth::delegate_short_lived_identity;
use candid::Principal;
use codee::string::JsonSerdeCodec;
use component::notification_nudge::NotificationNudge;
use component::spinner::SpinnerCircleStyled;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_use::storage::use_local_storage;
use state::canisters::{auth_state, unauth_canisters};
use utils::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps, MixpanelPostGameType};

#[component]
pub fn UploadAiPostPage() -> impl IntoView {
    // Signal to control returning to form for re-generation
    let show_form = RwSignal::new(true);

    // Notification and modal signals
    let notification_nudge = RwSignal::new(false);
    let show_success_modal = RwSignal::new(false);

    // Local storage for video generation parameters (for regeneration)
    let (stored_params, set_stored_params, _) =
        use_local_storage::<VideoGenerationParams, JsonSerdeCodec>(AI_VIDEO_PARAMS_STORE);

    // Get auth state outside actions for reuse
    let auth = auth_state();

    // Get unauth_canisters at component level to preserve reactive context
    let unauth_cans = unauth_canisters();
    let unauth_cans_for_upload = unauth_cans.clone();

    // Video generation action - cleaned up with helper functions
    let generate_action: Action<VideoGenerationParams, Result<String, String>> =
        Action::new_unsync({
            move |params: &VideoGenerationParams| {
                let params = params.clone();
                let show_form = show_form;
                let unauth_cans = unauth_cans.clone();
                let ev_ctx = auth.event_ctx();

                async move {
                    // Store model name for tracking
                    let model_name = params.model.name.clone();
                    
                    // Get auth canisters
                    let canisters = get_auth_canisters(&auth, unauth_cans).await?;
                    
                    // Get identity from canisters
                    let identity = canisters.identity();

                    // Create and sign the request
                    let signed_request = create_and_sign_request(identity, &params)?;

                    // Execute video generation
                    let result = execute_video_generation(signed_request, &canisters).await;
                    
                    // Track video generation result
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        match &result {
                            Ok(_) => {
                                MixPanelEvent::track_ai_video_generated(
                                    global,
                                    true,
                                    None,
                                    model_name
                                );
                            }
                            Err(error) => {
                                MixPanelEvent::track_ai_video_generated(
                                    global,
                                    false,
                                    Some(error.clone()),
                                    model_name
                                );
                            }
                        }
                    }
                    
                    // Update UI state on success
                    if result.is_ok() {
                        show_form.set(false);
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
            let unauth_cans = unauth_cans_for_upload.clone();
            let ev_ctx = auth.event_ctx();
            async move {
                // Show notification nudge when starting upload
                notification_nudge.set(true);
                
                // Track video upload initiated for AI video
                if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                    MixPanelEvent::track_video_upload_initiated(
                        global,
                        false, // caption_added - we're not using captions for AI videos
                        false, // hashtags_added - we're not using hashtags for AI videos
                        Some("ai_video".to_string())
                    );
                }

                // Get delegated identity within the Action
                match auth.auth_cans(unauth_cans).await {
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
                                        crate::consts::CREATOR_COMMISION_PERCENT,
                                        false, // is_game_enabled - AI videos don't have game enabled
                                        MixpanelPostGameType::HotOrNot,
                                        Some("ai_video".to_string())
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

    view! {
        <Title text="YRAL AI - Upload" />
        <NotificationNudge pop_up=notification_nudge />
        <div class="w-full h-full">
            <Show
                when=move || generate_action.pending().get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || {
                                // Show result screen if video generation was successful AND we're not in form mode
                                if let Some(result) = generate_action.value().get() {
                                    result.is_ok() && !show_form.get()
                                } else {
                                    false
                                }
                            }
                            fallback=move || {
                                view! {
                                    <PreUploadAiView
                                        generate_action=generate_action
                                        set_stored_params=set_stored_params
                                    />
                                }
                            }
                        >
                            <VideoResultScreen
                                video_url={
                                    generate_action.value().get()
                                        .and_then(|result| result.ok())
                                        .unwrap_or_default()
                                }
                                _generate_action=generate_action
                                on_upload=move || {
                                    // Get the generated video URL and upload using Action
                                    if let Some(Ok(video_url)) = generate_action.value().get() {
                                        leptos::logging::log!("Starting upload Action for video: {}", video_url);

                                        // Dispatch the upload action - it handles auth internally
                                        upload_action.dispatch(UploadActionParams {
                                            video_url,
                                        });
                                    }
                                }
                                on_regenerate=move || {
                                    // Get the stored parameters and regenerate
                                    let params = stored_params.get_untracked();
                                    // Check if we have valid parameters (not default/empty)
                                    if !params.prompt.is_empty() && params.user_principal != Principal::anonymous() {
                                        leptos::logging::log!("Re-generating video with stored parameters");
                                        
                                        // Track regenerate clicked
                                        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx()) {
                                            MixPanelEvent::track_regenerate_video_clicked(
                                                global,
                                                params.model.name.clone()
                                            );
                                        }
                                        
                                        // Dispatch the action again with the stored parameters
                                        generate_action.dispatch(params);
                                    } else {
                                        leptos::logging::warn!("No valid stored parameters found for regeneration");
                                        // Fallback to showing form
                                        show_form.set(true);
                                    }
                                }
                            />
                        </Show>
                    }
                }
            >
                <VideoGenerationLoadingScreen
                    prompt=stored_params.get().prompt.clone()
                    model=stored_params.get().model.clone()
                />
            </Show>
        </div>

        // Loading overlay during upload
        <Show when=move || upload_action.pending().get()>
            <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-75">
                <div class="flex flex-col items-center gap-4">
                    <div class="w-16 h-16">
                        <SpinnerCircleStyled />
                    </div>
                    <p class="text-white text-lg">Uploading video...</p>
                </div>
            </div>
        </Show>

        // Success screen
        <Show when=move || show_success_modal.get()>
            <PostUploadScreen />
        </Show>
    }
}
