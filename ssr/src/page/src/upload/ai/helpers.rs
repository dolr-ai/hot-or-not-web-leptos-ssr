use super::models::VideoModel;
use super::videogen_client::generate_video_with_signature;
use base64::{engine::general_purpose, Engine as _};
use candid::Principal;
use videogen_common::{
    ImageInput, Veo3AspectRatio, VideoGenInput, VideoGenRequest, VideoGenRequestWithSignature,
};

// Helper function to create video request
pub fn create_video_request(
    user_principal: Principal,
    prompt: String,
    model: VideoModel,
    image_data: Option<String>,
) -> Result<VideoGenRequest, Box<dyn std::error::Error>> {
    leptos::logging::log!("Starting video generation with prompt: {}", prompt);

    // Convert image data if provided
    let image_input = if let Some(data) = image_data {
        // Assuming the image_data is a base64 encoded string or data URL
        let image_bytes = if data.starts_with("data:") {
            // Parse data URL
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() == 2 {
                general_purpose::STANDARD.decode(parts[1]).ok()
            } else {
                None
            }
        } else {
            // Direct base64
            general_purpose::STANDARD.decode(&data).ok()
        };

        image_bytes.map(|bytes| ImageInput {
            data: bytes,
            mime_type: "image/png".to_string(), // Default to PNG, could be extracted from data URL
        })
    } else {
        None
    };

    // Create video generation input based on model
    let input = match model.id.as_str() {
        "pollo_1_6" | "cling_2_1" | "cling_2_1_master" => {
            VideoGenInput::Veo3 {
                prompt,
                negative_prompt: None,
                image: image_input,
                aspect_ratio: Veo3AspectRatio::Ratio16x9, // Default to 16:9
                duration_seconds: match model.id.as_str() {
                    "pollo_1_6" => 60,
                    "cling_2_1" => 120,
                    "cling_2_1_master" => 255, // Max duration for u8
                    _ => 60,
                },
                generate_audio: false,
            }
        }
        _ => {
            return Err("Unsupported model".into());
        }
    };

    // Create the request
    let request = VideoGenRequest {
        principal: user_principal,
        input,
    };

    Ok(request)
}
