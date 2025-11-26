use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a video pending approval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PendingVideo {
    pub video_uid: String,
    pub canister_id: String,
    pub post_id: String,
    pub publisher_user_id: String,
}

/// Response from fetching pending videos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingVideosResponse {
    pub videos: Vec<PendingVideo>,
    pub has_more: bool,
}

/// Fetch videos pending approval
/// TODO: Replace with actual API endpoint
#[server(endpoint = "fetch_pending_approval_videos")]
pub async fn fetch_pending_approval_videos(
    _offset: usize,
    _limit: usize,
) -> Result<PendingVideosResponse, ServerFnError> {
    // TODO: Implement actual API call
    // For now, return empty list as placeholder
    Ok(PendingVideosResponse {
        videos: vec![],
        has_more: false,
    })
}

/// Approve a video
/// TODO: Replace with actual API endpoint
#[server(endpoint = "approve_video")]
pub async fn approve_video(video_uid: String) -> Result<bool, ServerFnError> {
    // TODO: Implement actual API call
    leptos::logging::log!("Approving video: {}", video_uid);

    // Placeholder: always return success
    Ok(true)
}
