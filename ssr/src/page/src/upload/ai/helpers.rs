use crate::upload::ai::videogen_client::generate_video_with_identity;
use candid::Principal;
use state::canisters::AuthState;
use videogen_common::{ImageData, ImageInput, TokenType, VideoGenRequest, VideoModel};
use yral_canisters_common::Canisters;
use yral_types::delegated_identity::DelegatedIdentityWire;

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
    let image_data = if let Some(data) = image_data {
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

    // Create video generation input based on model
    let input = model.to_video_gen_input(prompt, image_data)?;

    // Create the request
    let request = VideoGenRequest {
        principal: user_principal,
        input,
        token_type,
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

/// Execute video generation with delegated identity (for DOLR)
pub async fn execute_video_generation_with_identity(
    request: VideoGenRequest,
    delegated_identity: DelegatedIdentityWire,
    canisters: &Canisters<true>,
) -> Result<String, String> {
    // Get rate limits client from canisters
    let rate_limits = canisters.rate_limits().await;

    // Generate video with delegated identity
    generate_video_with_identity(request, delegated_identity, &rate_limits)
        .await
        .map(|response| response.video_url)
        .map_err(|err| {
            leptos::logging::error!("Video generation with identity failed: {}", err);
            format!("Failed to generate video: {err}")
        })
}
