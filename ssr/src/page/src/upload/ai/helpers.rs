use crate::upload::ai::videogen_client::generate_video_with_identity_v2;
use candid::Principal;
use state::canisters::AuthState;
use std::collections::HashMap;
use videogen_common::{
    AudioData, AudioInput, ImageData, ImageInput, ProviderInfo, TokenType, VideoGenRequestV2,
};
use yral_canisters_common::Canisters;
use yral_types::delegated_identity::DelegatedIdentityWire;

// Helper function to create video request using V2 API
pub fn create_video_request_v2(
    user_principal: Principal,
    prompt: String,
    provider: &ProviderInfo,
    image_data: Option<String>,
    audio_data: Option<String>,
    token_type: TokenType,
) -> Result<VideoGenRequestV2, Box<dyn std::error::Error>> {
    leptos::logging::log!("Starting video generation with prompt: {}", prompt);

    // Check if provider is available
    if !provider.is_available {
        return Err(format!("Provider {} is coming soon", provider.name).into());
    }

    // Convert image data if provided
    let image_data = if let Some(data) = image_data {
        // Check if provider supports image input
        if !provider.supports_image {
            return Err(format!("Provider {} does not support image input", provider.name).into());
        }

        // We expect data URLs from the file upload: data:image/png;base64,iVBORw0KGgo...
        if data.starts_with("data:") {
            // Parse data URL to extract mime type and base64 data
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() == 2 {
                // Extract mime type from the first part
                let mime_part = parts[0];
                let mime_type = if let Some(start) = mime_part.find(':') {
                    if let Some(end) = mime_part.find(';') {
                        mime_part[start + 1..end].to_string()
                    } else {
                        // No semicolon found, take everything after colon
                        mime_part[start + 1..].to_string()
                    }
                } else {
                    "image/png".to_string() // Default fallback
                };

                leptos::logging::log!("Extracted mime type: {}", mime_type);

                Some(ImageData::Base64(ImageInput {
                    data: parts[1].to_string(),
                    mime_type,
                }))
            } else {
                leptos::logging::warn!("Invalid data URL format");
                None
            }
        } else {
            // If not a data URL, assume it's raw base64 with unknown type
            leptos::logging::warn!("Image data is not a data URL, defaulting to image/png");
            Some(ImageData::Base64(ImageInput {
                data,
                mime_type: "image/png".to_string(),
            }))
        }
    } else {
        None
    };

    // Convert audio data if provided (for TalkingHead)
    let audio_data = if let Some(data) = audio_data {
        // Check if provider supports audio input
        if !provider.supports_audio_input {
            return Err(format!("Provider {} does not support audio input", provider.name).into());
        }

        // We expect data URLs from the file upload: data:audio/mp3;base64,SUQzBAAAAAA...
        if data.starts_with("data:") {
            // Parse data URL to extract mime type and base64 data
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() == 2 {
                // Extract mime type from the first part
                let mime_part = parts[0];
                let mime_type = if let Some(start) = mime_part.find(':') {
                    if let Some(end) = mime_part.find(';') {
                        mime_part[start + 1..end].to_string()
                    } else {
                        // No semicolon found, take everything after colon
                        mime_part[start + 1..].to_string()
                    }
                } else {
                    "audio/mp3".to_string() // Default fallback
                };

                leptos::logging::log!("Extracted audio mime type: {}", mime_type);

                Some(AudioData::Base64(AudioInput {
                    data: parts[1].to_string(),
                    mime_type,
                }))
            } else {
                leptos::logging::warn!("Invalid audio data URL format");
                None
            }
        } else {
            // If not a data URL, assume it's raw base64 with unknown type
            leptos::logging::warn!("Audio data is not a data URL, defaulting to audio/mp3");
            Some(AudioData::Base64(AudioInput {
                data,
                mime_type: "audio/mp3".to_string(),
            }))
        }
    } else {
        None
    };

    // For TalkingHead, use a placeholder prompt since backend validation requires non-empty prompt
    let final_prompt = if provider.id == "talkinghead" {
        "[TalkingHead: Audio-based generation]".to_string() // Placeholder prompt for validation
    } else {
        prompt
    };

    // Use the provider's default duration (which is None for TalkingHead)
    let duration_seconds = provider.default_duration;

    // Create the V2 request
    let request = VideoGenRequestV2 {
        principal: user_principal,
        prompt: final_prompt,
        model_id: provider.id.clone(),
        token_type,
        negative_prompt: None, // Can be added later if needed
        image: image_data,
        audio: audio_data,
        aspect_ratio: provider.default_aspect_ratio.clone(),
        duration_seconds,
        resolution: provider.default_resolution.clone(),
        generate_audio: None,
        seed: None,
        extra_params: HashMap::new(),
    };

    Ok(request)
}

/// Get authenticated canisters
pub async fn get_auth_canisters(auth: &AuthState) -> Result<Canisters<true>, String> {
    auth.auth_cans().await.map_err(|err| {
        leptos::logging::error!("Failed to get auth canisters: {:?}", err);
        format!("Failed to get auth canisters: {err:?}")
    })
}

/// Execute video generation with delegated identity using V2 API
pub async fn execute_video_generation_with_identity_v2(
    request: VideoGenRequestV2,
    delegated_identity: DelegatedIdentityWire,
    canisters: &Canisters<true>,
) -> Result<String, String> {
    // Get rate limits client from canisters
    let rate_limits = canisters.rate_limits().await;

    // Generate video with delegated identity using V2 API
    generate_video_with_identity_v2(request, delegated_identity, &rate_limits)
        .await
        .map(|response| response.video_url)
        .map_err(|err| {
            leptos::logging::error!("Video generation with identity failed: {}", err);
            format!("Failed to generate video: {err}")
        })
}
