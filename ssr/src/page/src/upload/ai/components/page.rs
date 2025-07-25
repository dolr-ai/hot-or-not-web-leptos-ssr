use super::{PreUploadAiView, VideoGenerationLoadingScreen, VideoResultScreen};
use crate::upload::ai::helpers::create_video_request;
use crate::upload::ai::types::VideoGenerationParams;
use crate::upload::ai::server::upload_ai_video_from_url;
use crate::upload::ai::types::{UploadActionParams, AI_VIDEO_PARAMS_STORE};
use crate::upload::ai::videogen_client::{generate_video_with_signature, sign_videogen_request};
use crate::upload::PostUploadScreen;
use crate::upload::UploadParams;
use auth::delegate_short_lived_identity;
use candid::Principal;
use codee::string::JsonSerdeCodec;
use component::notification_nudge::NotificationNudge;
use component::spinner::SpinnerCircleStyled;
use leptos::prelude::*;
use leptos::reactive::send_wrapper_ext::SendOption;
use leptos_meta::Title;
use leptos_use::storage::use_local_storage;
use state::canisters::{auth_state, unauth_canisters};
use utils::event_streaming::events::{VideoUploadSuccessful, VideoUploadUnsuccessful};

#[component]
pub fn UploadAiPostPage() -> impl IntoView {
    let trigger_upload = RwSignal::new(SendOption::<UploadParams>::new_local(None));
    let uid = RwSignal::new(None);
    let upload_file_actual_progress = RwSignal::new(0.0f64);

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

    // Video generation action - this is the proper way to handle async operations
    let generate_action: Action<VideoGenerationParams, Result<String, String>> =
        Action::new_unsync({
            let show_form = show_form;
            let auth = auth.clone();
            move |params: &VideoGenerationParams| {
                let params = params.clone();
                let show_form = show_form;
                let auth = auth.clone();

                async move {
                    // Get auth canisters and identity for signing
                    let unauth_cans = unauth_canisters();
                    match auth.auth_cans(unauth_cans).await {
                        Ok(canisters) => {
                            let identity = canisters.identity();

                            // Create the video request
                            match create_video_request(
                                params.user_principal,
                                params.prompt,
                                params.model,
                                params.image_data,
                            ) {
                                Ok(request) => {
                                    // Sign the request on client side
                                    match sign_videogen_request(identity, request) {
                                        Ok(signed_request) => {
                                            // Generate video with signed request
                                            match generate_video_with_signature(signed_request)
                                                .await
                                            {
                                                Ok(videogen_resp) => {
                                                    // Set show_form to false to show result screen
                                                    show_form.set(false);
                                                    Ok(videogen_resp.video_url)
                                                }
                                                Err(err) => {
                                                    leptos::logging::error!(
                                                        "Video generation failed: {}",
                                                        err
                                                    );
                                                    Err(format!(
                                                        "Failed to generate video: {}",
                                                        err
                                                    ))
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            leptos::logging::error!(
                                                "Failed to sign request: {:?}",
                                                err
                                            );
                                            Err(format!("Failed to sign request: {:?}", err))
                                        }
                                    }
                                }
                                Err(err) => {
                                    leptos::logging::error!("Failed to create request: {}", err);
                                    Err(format!("Failed to create request: {}", err))
                                }
                            }
                        }
                        Err(err) => {
                            leptos::logging::error!("Failed to get auth canisters: {:?}", err);
                            Err(format!("Failed to get auth canisters: {:?}", err))
                        }
                    }
                }
            }
        });

    // Upload action - handles server-side video download and upload
    let upload_action: Action<UploadActionParams, Result<String, String>> = Action::new_unsync({
        let auth = auth.clone();
        let notification_nudge = notification_nudge.clone();
        let show_success_modal = show_success_modal.clone();
        move |params: &UploadActionParams| {
            let params = params.clone();
            let auth = auth.clone();
            let notification_nudge = notification_nudge.clone();
            let show_success_modal = show_success_modal.clone();
            async move {
                // Show notification nudge when starting upload
                notification_nudge.set(true);
                // Get unauth_canisters within the Action (like video_upload.rs)
                let unauth_cans = unauth_canisters();

                // Get delegated identity within the Action
                match auth.auth_cans(unauth_cans).await {
                    Ok(canisters) => {
                        let id = canisters.identity();
                        let delegated_identity = delegate_short_lived_identity(id);

                        // Call server function with delegated identity
                        match upload_ai_video_from_url(
                            params.video_url,
                            vec!["AI".to_string(), "Generated".to_string()],
                            "AI Generated Video".to_string(),
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

                                // Send success event
                                let ev_ctx = auth.event_ctx();
                                VideoUploadSuccessful.send_event(
                                    ev_ctx,
                                    video_uid.clone(),
                                    2,     // hashtags_len (AI tags)
                                    false, // is_nsfw
                                    false, // enable_hot_or_not
                                    0,     // post_id (using 0 for AI generated videos)
                                );

                                // Show success modal
                                show_success_modal.set(true);
                                Ok(video_uid)
                            }
                            Err(e) => {
                                leptos::logging::error!("Failed to upload video: {}", e);

                                // Send unsuccessful event
                                let ev_ctx = auth.event_ctx();
                                VideoUploadUnsuccessful.send_event(
                                    ev_ctx,
                                    e.to_string(),
                                    2,     // hashtags_len
                                    false, // enable_hot_or_not
                                    false, // is_nsfw
                                );

                                Err(format!("Upload failed: {}", e))
                            }
                        }
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to get auth canisters: {:?}", e);
                        Err(format!("Auth failed: {:?}", e))
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
                                        _trigger_upload=trigger_upload.write_only()
                                        _uid=uid
                                        _upload_file_actual_progress=upload_file_actual_progress.write_only()
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
                                    if let Some(result) = generate_action.value().get() {
                                        if let Ok(video_url) = result {
                                            leptos::logging::log!("Starting upload Action for video: {}", video_url);

                                            // Dispatch the upload action - it handles auth internally
                                            upload_action.dispatch(UploadActionParams {
                                                video_url,
                                            });
                                        }
                                    }
                                }
                                on_regenerate=move || {
                                    // Get the stored parameters and regenerate
                                    let params = stored_params.get_untracked();
                                    // Check if we have valid parameters (not default/empty)
                                    if !params.prompt.is_empty() && params.user_principal != Principal::anonymous() {
                                        leptos::logging::log!("Re-generating video with stored parameters");
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
