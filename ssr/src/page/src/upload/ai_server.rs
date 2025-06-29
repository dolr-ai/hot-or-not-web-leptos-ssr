use leptos::prelude::*;
use serde::{Deserialize, Serialize};

const API_KEY: &str = "d364f1fe-209b-43e5-9dff-386c39b67682";
const API_BASE_URL: &str = "https://veo3-video-generator-874803795683.us-central1.run.app";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenerateVideoRequest {
    pub prompt: String,
    pub sample_count: u32,
    pub generate_audio: bool,
    pub aspect_ratio: String,
    pub negative_prompt: String,
    pub duration_seconds: u32,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct GenerateVideoResponse {
    operation_name: String,
    message: String,
}

#[derive(Serialize)]
struct CheckStatusRequest {
    operation_name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CheckStatusResponse {
    completed: bool,
    gcs_uri: Option<String>,
    operation_name: String,
    message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoGenerationResult {
    pub video_url: String,
    pub message: String,
}

#[server(endpoint = "generate_video_from_prompt", input = server_fn::codec::Json)]
pub async fn generate_video_from_prompt(
    req: GenerateVideoRequest,
) -> Result<VideoGenerationResult, ServerFnError> {
    let client = reqwest::Client::new();

    // Step 1: Initiate video generation
    let url = format!("{}/generate_video_from_prompt", API_BASE_URL);
    let response = client
        .post(&url)
        .header("accept", "application/json")
        .header("X-API-Key", API_KEY)
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "API error: {}",
            response.status()
        )));
    }

    let gen_response = response
        .json::<GenerateVideoResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {}", e)))?;

    // Step 2: Poll for completion
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 60; // Poll for up to 5 minutes

    // gloo::timers::future::sleep(std::time::Duration::from_secs(2)).await; // Initial wait before polling

    loop {
        if attempts >= MAX_ATTEMPTS {
            return Err(ServerFnError::new("Video generation timed out".to_string()));
        }

        attempts += 1;

        // Wait 5 seconds before polling
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Check status
        let status_url = format!("{}/check_generation_status", API_BASE_URL);
        let status_req = CheckStatusRequest {
            operation_name: gen_response.operation_name.clone(),
        };

        let status_response = client
            .post(&status_url)
            .header("accept", "application/json")
            .header("X-API-Key", API_KEY)
            .header("Content-Type", "application/json")
            .json(&status_req)
            .send()
            .await
            .map_err(|e| ServerFnError::new(format!("Status check failed: {}", e)))?;

        if !status_response.status().is_success() {
            return Err(ServerFnError::new(format!(
                "Status check API error: {}",
                status_response.status()
            )));
        }

        let status = status_response
            .json::<CheckStatusResponse>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to parse status response: {}", e)))?;

        if status.completed {
            if let Some(_gcs_uri) = status.gcs_uri {
                // Convert GCS URI to public URL or use test URL
                // let video_url = if gcs_uri.starts_with("gs://") {
                //     gcs_uri.replace("gs://", "https://storage.googleapis.com/")
                // } else {
                //     // For testing
                //     "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/2472e3f1cbb742038f0e86a27c8ac98a/downloads/default.mp4".to_string()
                // };
                // For testing purposes, we will use a hardcoded URL
                let video_url = "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/2472e3f1cbb742038f0e86a27c8ac98a/downloads/default.mp4".to_string();

                return Ok(VideoGenerationResult {
                    video_url,
                    message: "Video generated successfully".to_string(),
                });
            } else {
                return Err(ServerFnError::new(
                    "Video completed but no URL provided".to_string(),
                ));
            }
        }
    }
}

#[server(endpoint = "fetch_video_bytes", input = server_fn::codec::Json)]
pub async fn fetch_video_bytes(
    video_url: String,
) -> Result<Vec<u8>, ServerFnError> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(&video_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch video: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to download video: {}",
            response.status()
        )));
    }
    
    // Check content length to avoid downloading huge files
    if let Some(content_length) = response.content_length() {
        if content_length > 100_000_000 { // 100MB limit
            return Err(ServerFnError::new("Video file too large (>100MB)".to_string()));
        }
    }
    
    let bytes = response
        .bytes()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read video bytes: {}", e)))?;
    
    leptos::logging::log!("Downloaded video bytes: {} bytes", bytes.len());
    
    Ok(bytes.to_vec())
}

#[server(endpoint = "get_video_proxy_url", input = server_fn::codec::Json)]
pub async fn get_video_proxy_url(
    video_url: String,
) -> Result<String, ServerFnError> {
    // For now, just return the URL as-is
    // In production, you might want to:
    // 1. Validate the URL
    // 2. Generate a signed URL
    // 3. Set up a proper video streaming proxy
    Ok(video_url)
}
