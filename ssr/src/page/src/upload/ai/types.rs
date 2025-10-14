use candid::Principal;
use serde::{Deserialize, Serialize};
use videogen_common::{ProviderInfo, TokenType};

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
#[derive(Clone, Serialize, Deserialize)]
pub struct VideoGenerationParams {
    pub user_principal: Principal,
    pub prompt: String,
    pub provider: ProviderInfo,
    pub image_data: Option<String>,
    pub audio_data: Option<String>, // Base64 encoded audio data
    pub token_type: TokenType,
}

// We need a Default implementation, but since ProviderInfo doesn't have a default,
// we'll create a dummy one. The actual provider will be set when providers are loaded.
impl Default for VideoGenerationParams {
    fn default() -> Self {
        // Create a minimal dummy ProviderInfo - this will be replaced when actual providers load
        let dummy_provider = serde_json::from_str::<ProviderInfo>(
            r#"
            {
                "id": "placeholder",
                "name": "Loading...",
                "description": "Please wait while providers load",
                "cost": {"usd_cents": 0, "dolr": 0, "sats": 0},
                "supports_image": false,
                "supports_negative_prompt": false,
                "supports_audio": false,
                "supports_audio_input": false,
                "supports_seed": false,
                "allowed_aspect_ratios": [],
                "allowed_resolutions": [],
                "allowed_durations": [5],
                "default_aspect_ratio": null,
                "default_resolution": null,
                "default_duration": 5,
                "is_available": false,
                "is_internal": false,
                "model_icon": null,
                "extra_info": {}
            }
        "#,
        )
        .unwrap();

        Self {
            user_principal: Principal::anonymous(),
            prompt: String::new(),
            provider: dummy_provider,
            image_data: None,
            audio_data: None,
            token_type: TokenType::Sats,
        }
    }
}

// Implement PartialEq manually to work around ProviderInfo not implementing it
impl PartialEq for VideoGenerationParams {
    fn eq(&self, other: &Self) -> bool {
        self.user_principal == other.user_principal
            && self.prompt == other.prompt
            && self.provider.id == other.provider.id  // Compare by ID only
            && self.image_data == other.image_data
            && self.audio_data == other.audio_data
            && self.token_type == other.token_type
    }
}
