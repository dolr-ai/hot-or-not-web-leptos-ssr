use super::models::VideoModel;
use candid::Principal;
use videogen_common::{ImageInput, Veo3AspectRatio, VideoGenInput, VideoGenRequest};

// Helper function to create video request
pub fn create_video_request(
    user_principal: Principal,
    prompt: String,
    model: VideoModel,
    image_data: Option<String>,
) -> Result<VideoGenRequest, Box<dyn std::error::Error>> {
    leptos::logging::log!("Starting video generation with prompt: {}", prompt);
    leptos::logging::log!("Image data received: {:?}", image_data.as_ref().map(|d| &d[..50.min(d.len())]));

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
                data: data,
                mime_type: "image/png".to_string(),
            })
        }
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
                    "pollo_1_6" => 8,
                    "cling_2_1" => 8,
                    "cling_2_1_master" => 8, // Max duration for u8
                    _ => 8,
                },
                generate_audio: true,
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
