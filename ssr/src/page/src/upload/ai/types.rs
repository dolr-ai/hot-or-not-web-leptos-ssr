use candid::Principal;
use serde::{Deserialize, Serialize};
use utils::host::show_preview_component;
use videogen_common::{TokenType, VideoGenProvider, VideoModel};

// Local storage key for video generation parameters
pub const AI_VIDEO_PARAMS_STORE: &str = "ai_video_generation_params";

// Internal types for server function
#[derive(Deserialize)]
pub struct UploadUrlResponse {
    pub data: Option<UploadUrlData>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct UploadUrlData {
    pub uid: Option<String>,
    #[serde(rename = "uploadURL")]
    pub upload_url: Option<String>,
}

#[derive(Serialize)]
pub struct VideoMetadata {
    pub title: String,
    pub description: String,
    pub tags: String,
}

#[derive(Serialize)]
pub struct SerializablePostDetailsFromFrontend {
    pub is_nsfw: bool,
    pub hashtags: Vec<String>,
    pub description: String,
    pub video_uid: String,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
}

// Upload action parameters struct
#[derive(Clone)]
pub struct UploadActionParams {
    pub video_url: String,
    pub token_type: TokenType,
}

// Video generation parameters
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoGenerationParams {
    pub user_principal: Principal,
    pub prompt: String,
    pub model: VideoModel,
    pub image_data: Option<String>,
    pub token_type: TokenType,
}

impl Default for VideoGenerationParams {
    fn default() -> Self {
        let is_preview = show_preview_component();
        let all_models = VideoModel::get_models();
        let filtered_models: Vec<VideoModel> = if is_preview {
            all_models
        } else {
            all_models
                .into_iter()
                .filter(|model| model.provider != VideoGenProvider::IntTest)
                .collect()
        };

        Self {
            user_principal: Principal::anonymous(),
            prompt: String::new(),
            model: filtered_models.into_iter().next().unwrap_or_default(),
            image_data: None,
            token_type: TokenType::Sats,
        }
    }
}
