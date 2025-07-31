use crate::upload::ai::types::VideoGenerationParams;
use crate::upload::ai::videogen_client::{generate_video_with_signature, sign_videogen_request};
use candid::Principal;
use state::canisters::AuthState;
use videogen_common::{ImageInput, TokenType, VideoGenRequest, VideoGenRequestWithSignature, VideoModel};
use yral_canisters_common::Canisters;

// Helper function to create video request
pub fn create_video_request(
    user_principal: Principal,
    prompt: String,
    model: VideoModel,
    image_data: Option<String>,
    token_type: TokenType,
) -> Result<VideoGenRequest, Box<dyn std::error::Error>> {
    leptos::logging::log!("Starting video generation with prompt: {}", prompt);

    // Convert image data if provided
    let image_input = if let Some(data) = image_data {
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

                Some(ImageInput {
                    data: parts[1].to_string(),
                    mime_type,
                })
            } else {
                leptos::logging::warn!("Invalid data URL format");
                None
            }
        } else {
            // If not a data URL, assume it's raw base64 with unknown type
            leptos::logging::warn!("Image data is not a data URL, defaulting to image/png");
            Some(ImageInput {
                data,
                mime_type: "image/png".to_string(),
            })
        }
    } else {
        None
    };

    // Create video generation input based on model
    let input = model.to_video_gen_input(prompt, image_input)?;

    // Create the request
    let request = VideoGenRequest {
        principal: user_principal,
        input,
        token_type,
    };

    Ok(request)
}

/// Get authenticated canisters
pub async fn get_auth_canisters(
    auth: &AuthState,
    unauth_cans: Canisters<false>,
) -> Result<Canisters<true>, String> {
    auth.auth_cans(unauth_cans).await.map_err(|err| {
        leptos::logging::error!("Failed to get auth canisters: {:?}", err);
        format!("Failed to get auth canisters: {err:?}")
    })
}

/// Create and sign a video generation request
pub fn create_and_sign_request(
    identity: &impl ic_agent::Identity,
    params: &VideoGenerationParams,
) -> Result<VideoGenRequestWithSignature, String> {
    // Create the video request
    let request = create_video_request(
        params.user_principal,
        params.prompt.clone(),
        params.model.clone(),
        params.image_data.clone(),
        params.token_type,
    )
    .map_err(|err| {
        leptos::logging::error!("Failed to create request: {}", err);
        format!("Failed to create request: {err}")
    })?;

    // Sign the request
    sign_videogen_request(identity, request).map_err(|err| {
        leptos::logging::error!("Failed to sign request: {:?}", err);
        format!("Failed to sign request: {err:?}")
    })
}

/// Execute video generation with signed request
pub async fn execute_video_generation(
    signed_request: VideoGenRequestWithSignature,
    canisters: &Canisters<true>,
) -> Result<String, String> {
    // Get rate limits client from canisters
    let rate_limits = canisters.rate_limits().await;

    // Generate video with signed request
    generate_video_with_signature(signed_request, &rate_limits)
        .await
        .map(|response| response.video_url)
        .map_err(|err| {
            leptos::logging::error!("Video generation failed: {}", err);
            format!("Failed to generate video: {err}")
        })
}
