use candid::Principal;
use serde::{Deserialize, Serialize};
use utils::host::show_preview_component;
use videogen_common::{
    types_v2::{AspectRatioV2, ProviderInfo, ResolutionV2, VideoGenRequestV2},
    TokenType, VideoGenProvider, VideoModel,
};

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

// V2 Video generation parameters using provider info
#[derive(Clone, Serialize, Deserialize)]
pub struct VideoGenerationParamsV2 {
    pub user_principal: Principal,
    pub prompt: String,
    pub model_id: String,
    pub provider_info: Option<ProviderInfo>, // Cached provider info
    pub image_data: Option<String>,
    pub token_type: TokenType,
    pub aspect_ratio: Option<AspectRatioV2>,
    pub resolution: Option<ResolutionV2>,
    pub duration_seconds: Option<u8>,
}

impl VideoGenerationParamsV2 {
    /// Convert to VideoGenRequestV2 for API call
    pub fn to_request(&self) -> VideoGenRequestV2 {
        VideoGenRequestV2 {
            principal: self.user_principal,
            prompt: self.prompt.clone(),
            model_id: self.model_id.clone(),
            token_type: self.token_type.clone(),
            negative_prompt: None,
            image: self.image_data.as_ref().map(|data| {
                videogen_common::ImageData::Base64(videogen_common::ImageInput {
                    data: data.clone(),
                    mime_type: "image/png".to_string(), // Default to PNG
                })
            }),
            aspect_ratio: self.aspect_ratio.clone(),
            duration_seconds: self.duration_seconds,
            resolution: self.resolution.clone(),
            generate_audio: None,
            seed: None,
            extra_params: Default::default(),
        }
    }
}
