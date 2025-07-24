use serde::{Deserialize, Serialize};

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
}