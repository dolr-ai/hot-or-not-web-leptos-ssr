use super::types::{SerializablePostDetailsFromFrontend, UploadUrlResponse, VideoMetadata};
use candid::Principal;
use consts::{OFF_CHAIN_AGENT_URL, UPLOAD_URL};
use leptos::prelude::*;
use leptos::server_fn::codec::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use utils::host::show_preview_component;
use videogen_common::{
    types_v2::{AspectRatioV2, ResolutionV2},
    AudioData, AudioInput, ImageData, ImageInput, ProviderInfo, TokenType, VideoGenClient,
    VideoGenQueuedResponseV2, VideoGenRequestStatus, VideoGenRequestV2,
    VideoGenRequestWithIdentityV2,
};
use yral_types::delegated_identity::DelegatedIdentityWire;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderInfoSerde {
    pub id: String,
    pub name: String,
    pub is_available: bool,
    pub supports_image: bool,
    pub supports_audio_input: bool,
    pub default_duration: Option<u8>,
    pub default_aspect_ratio: Option<AspectRatioV2>,
    pub default_resolution: Option<ResolutionV2>,
}

// Combined server function for video generation and upload - atomic operation
#[server(endpoint = "generate_and_upload_ai_video", input = Json, output = Json)]
pub async fn generate_and_upload_ai_video(
    user_principal: Principal,
    prompt: String,
    provider: ProviderInfoSerde,
    image_data: Option<String>,
    audio_data: Option<String>,
    token_type: TokenType,
    delegated_identity_wire: DelegatedIdentityWire,
) -> Result<String, ServerFnError> {
    leptos::logging::log!("Starting combined video generation and upload");

    // Check if provider is available
    if !provider.is_available {
        return Err(ServerFnError::new(format!(
            "Provider {} is coming soon",
            provider.name
        )));
    }

    // Convert image data if provided
    let image_data_converted = if let Some(data) = image_data {
        if !provider.supports_image {
            return Err(ServerFnError::new(format!(
                "Provider {} does not support image input",
                provider.name
            )));
        }

        if data.starts_with("data:") {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() == 2 {
                let mime_part = parts[0];
                let mime_type = if let Some(start) = mime_part.find(':') {
                    if let Some(end) = mime_part.find(';') {
                        mime_part[start + 1..end].to_string()
                    } else {
                        mime_part[start + 1..].to_string()
                    }
                } else {
                    "image/png".to_string()
                };

                Some(ImageData::Base64(ImageInput {
                    data: parts[1].to_string(),
                    mime_type,
                }))
            } else {
                None
            }
        } else {
            Some(ImageData::Base64(ImageInput {
                data,
                mime_type: "image/png".to_string(),
            }))
        }
    } else {
        None
    };

    // Convert audio data if provided
    let audio_data_converted = if let Some(data) = audio_data {
        if !provider.supports_audio_input {
            return Err(ServerFnError::new(format!(
                "Provider {} does not support audio input",
                provider.name
            )));
        }

        if data.starts_with("data:") {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() == 2 {
                let mime_part = parts[0];
                let mime_type = if let Some(start) = mime_part.find(':') {
                    if let Some(end) = mime_part.find(';') {
                        mime_part[start + 1..end].to_string()
                    } else {
                        mime_part[start + 1..].to_string()
                    }
                } else {
                    "audio/mp3".to_string()
                };

                Some(AudioData::Base64(AudioInput {
                    data: parts[1].to_string(),
                    mime_type,
                }))
            } else {
                None
            }
        } else {
            Some(AudioData::Base64(AudioInput {
                data,
                mime_type: "audio/mp3".to_string(),
            }))
        }
    } else {
        None
    };

    // For TalkingHead, use a placeholder prompt
    let final_prompt = if provider.id == "talkinghead" {
        "[TalkingHead: Audio-based generation]".to_string()
    } else {
        prompt
    };

    // Create the V2 request
    let request = VideoGenRequestV2 {
        principal: user_principal,
        prompt: final_prompt,
        model_id: provider.id.clone(),
        token_type,
        negative_prompt: None,
        image: image_data_converted,
        audio: audio_data_converted,
        aspect_ratio: provider.default_aspect_ratio,
        duration_seconds: provider.default_duration,
        resolution: provider.default_resolution,
        generate_audio: None,
        seed: None,
        extra_params: HashMap::new(),
    };

    // Generate video
    leptos::logging::log!("Calling video generation API");
    let video_url = generate_video_server_side(request, delegated_identity_wire.clone()).await?;
    leptos::logging::log!("Video generated successfully: {}", video_url);

    // Upload video
    leptos::logging::log!("Uploading video to platform");
    let video_uid = upload_ai_video_from_url(
        video_url.clone(),
        vec![],
        "".to_string(),
        delegated_identity_wire,
        false, // is_nsfw
        false, // enable_hot_or_not
    )
    .await?;

    leptos::logging::log!("Video uploaded successfully with UID: {}", video_uid);

    // Return the original video URL for preview (not the video_uid)
    Ok(video_url)
}

// Helper function for server-side video generation with polling
async fn generate_video_server_side(
    request: VideoGenRequestV2,
    delegated_identity: DelegatedIdentityWire,
) -> Result<String, ServerFnError> {
    let videogen_client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.clone());

    let identity_request = VideoGenRequestWithIdentityV2 {
        request,
        delegated_identity,
    };

    // Get the queued response with request_key
    let queued_response: VideoGenQueuedResponseV2 = videogen_client
        .generate_with_identity_v2(identity_request)
        .await
        .map_err(|e| ServerFnError::new(format!("Video generation failed: {e}")))?;

    let request_key = queued_response.request_key;

    // Poll for status using direct HTTP requests (server-side polling with tokio::time::sleep)
    const POLL_INTERVAL_SECS: u64 = 15;
    const MAX_ATTEMPTS: u32 = 20; // 5 minutes total

    let http_client = reqwest::Client::new();
    let poll_url = format!(
        "{}/api/v2/video/poll/{}/{}",
        OFF_CHAIN_AGENT_URL.as_str(),
        request_key.principal,
        request_key.counter
    );

    for attempt in 0..MAX_ATTEMPTS {
        if attempt > 0 {
            #[cfg(feature = "ssr")]
            tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;

            #[cfg(not(feature = "ssr"))]
            gloo::timers::future::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }

        match http_client.get(&poll_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<VideoGenRequestStatus>().await {
                        Ok(status) => match status {
                            VideoGenRequestStatus::Complete(video_url) => {
                                leptos::logging::log!("Video generation completed: {}", video_url);
                                return Ok(video_url);
                            }
                            VideoGenRequestStatus::Failed(error) => {
                                return Err(ServerFnError::new(format!(
                                    "Video generation failed: {error}"
                                )));
                            }
                            VideoGenRequestStatus::Pending | VideoGenRequestStatus::Processing => {
                                leptos::logging::log!(
                                    "Video generation in progress (attempt {}/{})",
                                    attempt + 1,
                                    MAX_ATTEMPTS
                                );
                            }
                        },
                        Err(e) => {
                            leptos::logging::log!("Error parsing poll response: {}", e);
                            // Continue polling on parse errors
                        }
                    }
                } else {
                    leptos::logging::log!("Poll request failed with status: {}", response.status());
                    // Continue polling on HTTP errors
                }
            }
            Err(e) => {
                leptos::logging::log!("Error polling status: {}", e);
                // Continue polling on transient errors
            }
        }
    }

    Err(ServerFnError::new(
        "Video generation timed out after 5 minutes".to_string(),
    ))
}

// Server function to download AI video and upload using existing worker flow
// TODO: shift to direct URL upload to Cloudflare Stream
pub async fn upload_ai_video_from_url(
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
