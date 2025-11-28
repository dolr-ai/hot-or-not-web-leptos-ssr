use consts::OFF_CHAIN_AGENT_URL;
use leptos::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use yral_types::delegated_identity::DelegatedIdentityWire;

/// Represents a video pending approval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PendingVideo {
    pub video_id: String,
    pub post_id: String,
    pub canister_id: String,
    pub user_id: String,
    pub created_at: String,
}

/// Response from fetching pending videos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingVideosResponse {
    pub videos: Vec<PendingVideo>,
    pub total_count: usize,
}

/// Response from approve/disapprove actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationActionResponse {
    pub success: bool,
    pub message: String,
}

/// Fetch videos pending approval
#[server(endpoint = "fetch_pending_approval_videos")]
pub async fn fetch_pending_approval_videos(
    delegated_identity_wire: DelegatedIdentityWire,
    offset: u32,
    limit: u32,
) -> Result<PendingVideosResponse, ServerFnError> {
    let client = Client::new();
    let body = json!({
        "delegated_identity_wire": delegated_identity_wire,
        "limit": limit,
        "offset": offset
    });

    let url = OFF_CHAIN_AGENT_URL
        .join("api/v1/moderation/pending")
        .map_err(|e| ServerFnError::new(format!("Invalid URL: {e}")))?;

    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch pending videos: {e}")))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ServerFnError::new(
            "Unauthorized: Invalid delegated identity",
        ));
    }
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(ServerFnError::new(
            "Forbidden: You are not a whitelisted moderator",
        ));
    }
    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to fetch pending videos: HTTP {}",
            response.status()
        )));
    }

    let result: PendingVideosResponse = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {e}")))?;

    Ok(result)
}

/// Approve a video
#[server(endpoint = "approve_video")]
pub async fn approve_video(
    delegated_identity_wire: DelegatedIdentityWire,
    video_id: String,
) -> Result<ModerationActionResponse, ServerFnError> {
    let client = Client::new();
    let body = json!({
        "delegated_identity_wire": delegated_identity_wire
    });

    let url = OFF_CHAIN_AGENT_URL
        .join(&format!("api/v1/moderation/approve/{video_id}"))
        .map_err(|e| ServerFnError::new(format!("Invalid URL: {e}")))?;

    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to approve video: {e}")))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ServerFnError::new(
            "Unauthorized: Invalid delegated identity",
        ));
    }
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(ServerFnError::new(
            "Forbidden: You are not a whitelisted moderator",
        ));
    }
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ModerationActionResponse {
            success: false,
            message: format!("Video {video_id} not found"),
        });
    }
    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to approve video: HTTP {}",
            response.status()
        )));
    }

    let result: ModerationActionResponse = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {e}")))?;

    Ok(result)
}

/// Disapprove a video (delete from approval queue)
#[server(endpoint = "disapprove_video")]
pub async fn disapprove_video(
    delegated_identity_wire: DelegatedIdentityWire,
    video_id: String,
) -> Result<ModerationActionResponse, ServerFnError> {
    let client = Client::new();
    let body = json!({
        "delegated_identity_wire": delegated_identity_wire
    });

    let url = OFF_CHAIN_AGENT_URL
        .join(&format!("api/v1/moderation/disapprove/{video_id}"))
        .map_err(|e| ServerFnError::new(format!("Invalid URL: {e}")))?;

    let response = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to disapprove video: {e}")))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ServerFnError::new(
            "Unauthorized: Invalid delegated identity",
        ));
    }
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(ServerFnError::new(
            "Forbidden: You are not a whitelisted moderator",
        ));
    }
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ModerationActionResponse {
            success: false,
            message: format!("Video {video_id} not found"),
        });
    }
    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to disapprove video: HTTP {}",
            response.status()
        )));
    }

    let result: ModerationActionResponse = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {e}")))?;

    Ok(result)
}
