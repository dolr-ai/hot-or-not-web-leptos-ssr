use super::types::{SerializablePostDetailsFromFrontend, UploadUrlResponse, VideoMetadata};
use consts::{OFF_CHAIN_AGENT_URL, UPLOAD_URL};
use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde_json::json;
use utils::host::show_preview_component;
use videogen_common::{
    ProviderInfo, VideoGenClient, VideoGenQueuedResponseV2, VideoGenRequestStatus,
    VideoGenRequestV2, VideoGenRequestWithIdentityV2,
};
use yral_canisters_common::Canisters;
use yral_types::delegated_identity::DelegatedIdentityWire;

// Internal function to download AI video and upload using existing worker flow
// TODO: shift to direct URL upload to Cloudflare Stream
async fn upload_ai_video_from_url_impl(
    video_url: String,
    hashtags: Vec<String>,
    description: String,
    delegated_identity_wire: DelegatedIdentityWire,
    is_nsfw: bool,
    enable_hot_or_not: bool,
) -> Result<String, ServerFnError> {
    leptos::logging::log!("Starting AI video upload from URL: {}", video_url);

    // Step 1: Download video using reqwest
    let client = reqwest::Client::new();
    let response = client
        .get(&video_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to download video: {e}")))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to download video: HTTP {}",
            response.status()
        )));
    }

    let video_bytes = response
        .bytes()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read video bytes: {e}")))?;

    leptos::logging::log!("Downloaded video, size: {} bytes", video_bytes.len());

    // Step 2: Get upload URL from worker
    let upload_response = client
        .get(format!("{UPLOAD_URL}/get_upload_url_v2"))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get upload URL: {e}")))?;

    if !upload_response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to get upload URL: HTTP {}",
            upload_response.status()
        )));
    }

    let upload_response_text = upload_response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read upload URL response: {e}")))?;

    let upload_message: UploadUrlResponse = serde_json::from_str(&upload_response_text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse upload URL response: {e}")))?;

    if !upload_message.success {
        return Err(ServerFnError::new(format!(
            "Upload URL request failed: {}",
            upload_message.message.unwrap_or_default()
        )));
    }

    let upload_data = upload_message
        .data
        .ok_or_else(|| ServerFnError::new("Upload URL data not found in response".to_string()))?;

    let upload_url = upload_data
        .upload_url
        .ok_or_else(|| ServerFnError::new("Upload URL not found in response".to_string()))?;

    let video_uid = upload_data
        .uid
        .ok_or_else(|| ServerFnError::new("Video UID not found in response".to_string()))?;

    leptos::logging::log!("Got upload URL and video UID: {}", video_uid);

    // Step 3: Upload to Cloudflare Stream
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(video_bytes.to_vec())
            .file_name("ai_generated_video.mp4")
            .mime_str("video/mp4")
            .map_err(|e| ServerFnError::new(format!("Failed to set MIME type: {e}")))?,
    );

    let upload_result = client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to upload to Cloudflare: {e}")))?;

    if !upload_result.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Cloudflare upload failed: HTTP {}",
            upload_result.status()
        )));
    }

    leptos::logging::log!("Successfully uploaded to Cloudflare Stream");

    // Step 4: Update metadata using types from video_upload.rs
    let metadata_request = json!({
        "video_uid": video_uid,
        "delegated_identity_wire": delegated_identity_wire,
        "meta": VideoMetadata{
            title: description.clone(),
            description: description.clone(),
            tags: hashtags.join(",")
        },
        "post_details": SerializablePostDetailsFromFrontend{
            is_nsfw,
            hashtags,
            description,
            video_uid: video_uid.clone(),
            creator_consent_for_inclusion_in_hot_or_not: enable_hot_or_not,
        }
    });

    let metadata_result = client
        .post(format!("{UPLOAD_URL}/update_metadata"))
        .json(&metadata_request)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update metadata: {e}")))?;

    if !metadata_result.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Metadata update failed: HTTP {}",
            metadata_result.status()
        )));
    }

    leptos::logging::log!("Successfully updated metadata for video: {}", video_uid);

    Ok(video_uid)
}

// Server function wrapper for upload_ai_video_from_url_impl
#[server(endpoint = "upload_ai_video_from_url", input = Json, output = Json)]
pub async fn upload_ai_video_from_url(
    video_url: String,
    hashtags: Vec<String>,
    description: String,
    delegated_identity_wire: DelegatedIdentityWire,
    is_nsfw: bool,
    enable_hot_or_not: bool,
) -> Result<String, ServerFnError> {
    upload_ai_video_from_url_impl(
        video_url,
        hashtags,
        description,
        delegated_identity_wire,
        is_nsfw,
        enable_hot_or_not,
    )
    .await
}

// Server function that handles the complete video generation flow:
// 1. Initiate video generation with identity
// 2. Poll for completion (up to 5 minutes)
// 3. Upload the completed video
// This runs entirely on the server, so it continues even if user navigates away
#[server(endpoint = "generate_and_upload_video", input = Json, output = Json)]
pub async fn generate_and_upload_video(
    request: VideoGenRequestV2,
    delegated_identity: DelegatedIdentityWire,
    hashtags: Vec<String>,
    description: String,
    is_nsfw: bool,
    enable_hot_or_not: bool,
) -> Result<String, ServerFnError> {
    leptos::logging::log!("Starting video generation and upload flow");

    // Create client
    let client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.clone());

    // Create request with identity for V2 API
    let identity_request = VideoGenRequestWithIdentityV2 {
        request,
        delegated_identity: delegated_identity.clone(),
    };

    // Step 1: Initiate video generation
    let queued_response: VideoGenQueuedResponseV2 = client
        .generate_with_identity_v2(identity_request)
        .await
        .map_err(|e| {
            leptos::logging::error!("Error generating video with identity: {}", e);
            ServerFnError::new(format!("Failed to initiate video generation: {e}"))
        })?;

    let request_key = queued_response.request_key;
    leptos::logging::log!(
        "Video generation initiated with request_key: {:?}",
        request_key
    );

    // Step 2: Poll for completion (15 second intervals for 5 minutes)
    const POLL_INTERVAL_SECS: u64 = 15;
    const MAX_ATTEMPTS: u32 = 20; // 20 * 15 = 300 seconds = 5 minutes

    // Create canisters from delegated identity wire
    let canisters = Canisters::authenticate_with_network(delegated_identity.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create canisters: {e}")))?;
    let rate_limits_client = canisters.rate_limits().await;

    let mut final_url: Option<String> = None;

    for attempt in 0..MAX_ATTEMPTS {
        // Wait before polling (except on first attempt)
        if attempt > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }

        leptos::logging::log!(
            "Polling video status, attempt {}/{}",
            attempt + 1,
            MAX_ATTEMPTS
        );

        // Poll the status
        match client
            .poll_video_status_with_client(&request_key, &rate_limits_client)
            .await
        {
            Ok(status) => match status {
                VideoGenRequestStatus::Complete(video_url) => {
                    leptos::logging::log!("Video generation completed: {}", video_url);
                    final_url = Some(video_url);
                    break;
                }
                VideoGenRequestStatus::Failed(error) => {
                    leptos::logging::error!("Video generation failed: {}", error);
                    return Err(ServerFnError::new(format!(
                        "Video generation failed: {error}"
                    )));
                }
                VideoGenRequestStatus::Pending | VideoGenRequestStatus::Processing => {
                    leptos::logging::log!("Video generation in progress: {:?}", status);
                    // Continue polling
                }
            },
            Err(e) => {
                leptos::logging::warn!("Error polling status (will retry): {}", e);
                // Continue polling on transient errors
            }
        }
    }

    let video_url = final_url.ok_or_else(|| {
        ServerFnError::new("Video generation timed out after 5 minutes".to_string())
    })?;

    leptos::logging::log!("Video ready at URL: {}", video_url);

    // Step 3: Upload the completed video
    upload_ai_video_from_url_impl(
        video_url,
        hashtags,
        description,
        delegated_identity,
        is_nsfw,
        enable_hot_or_not,
    )
    .await
}

// Server function to fetch available video generation providers from the API
#[server(endpoint = "fetch_video_providers", input = Json, output = Json)]
pub async fn fetch_video_providers() -> Result<Vec<ProviderInfo>, ServerFnError> {
    let client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.clone());
    let is_preview = show_preview_component();

    // Use get_providers_all for preview mode to include test models
    let providers_result = if is_preview {
        client.get_providers_all().await
    } else {
        client.get_providers().await
    };

    match providers_result {
        Ok(providers_response) => {
            // Filter out internal/test providers in non-preview mode
            let providers = if !is_preview {
                providers_response
                    .providers
                    .into_iter()
                    .filter(|p| !p.is_internal)
                    .collect()
            } else {
                providers_response.providers
            };
            Ok(providers)
        }
        Err(e) => {
            leptos::logging::error!("Failed to fetch providers from API: {}", e);
            // Return empty vector as fallback
            Ok(Vec::new())
        }
    }
}
