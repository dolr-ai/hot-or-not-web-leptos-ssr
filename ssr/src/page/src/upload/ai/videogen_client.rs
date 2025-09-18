use consts::OFF_CHAIN_AGENT_URL;
use gloo::timers::future::TimeoutFuture;
use videogen_common::{
    VideoGenClient, VideoGenError, VideoGenQueuedResponseV2, VideoGenRequestStatus,
    VideoGenRequestV2, VideoGenRequestWithIdentityV2, VideoGenResponse,
};
use yral_canisters_client::rate_limits::RateLimits;
use yral_types::delegated_identity::DelegatedIdentityWire;

/// Generate video using the delegated identity flow with V2 API
/// The off-chain agent will use the user's identity to make direct transfers
pub async fn generate_video_with_identity_v2(
    request: VideoGenRequestV2,
    delegated_identity: DelegatedIdentityWire,
    rate_limits: &RateLimits<'_>,
) -> Result<VideoGenResponse, VideoGenError> {
    // Create client
    let client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.clone());

    // Create request with identity for V2 API
    let identity_request = VideoGenRequestWithIdentityV2 {
        request,
        delegated_identity,
    };

    // Get the queued response with request_key
    let queued_response: VideoGenQueuedResponseV2 = client
        .generate_with_identity_v2(identity_request)
        .await
        .map_err(|e| {
            leptos::logging::log!("Error generating video with identity: {}", e);
            e
        })?;

    // Extract request_key from the response
    let request_key = queued_response.request_key;

    // Start polling for status
    poll_video_status(&client, &request_key, rate_limits).await
}

/// Poll the video generation status every 15 seconds for 5 minutes
async fn poll_video_status(
    client: &VideoGenClient,
    request_key: &videogen_common::VideoGenRequestKey,
    rate_limits: &RateLimits<'_>,
) -> Result<VideoGenResponse, VideoGenError> {
    // Poll every 15 seconds for 5 minutes
    const POLL_INTERVAL_MS: u32 = 15000; // 15 seconds
    const MAX_ATTEMPTS: u32 = 20; // 20 attempts * 15 seconds = 5 minutes

    for attempt in 0..MAX_ATTEMPTS {
        // Wait before polling (except on first attempt)
        if attempt > 0 {
            TimeoutFuture::new(POLL_INTERVAL_MS).await;
        }

        // Poll the status
        match client
            .poll_video_status_with_client(request_key, rate_limits)
            .await
        {
            Ok(status) => match status {
                VideoGenRequestStatus::Complete(video_url) => {
                    leptos::logging::log!("Video generation completed: {}", video_url);
                    return Ok(VideoGenResponse {
                        operation_id: format!("{}_{}", request_key.principal, request_key.counter),
                        video_url,
                        provider: "unknown".to_string(), // We don't have provider info from status
                    });
                }
                VideoGenRequestStatus::Failed(error) => {
                    leptos::logging::log!("Video generation failed: {}", error);
                    return Err(VideoGenError::ProviderError(error));
                }
                VideoGenRequestStatus::Pending | VideoGenRequestStatus::Processing => {
                    leptos::logging::log!("Video generation status: {:?}", status);
                    // Continue polling
                }
            },
            Err(e) => {
                leptos::logging::log!("Error polling status: {}", e);
                // Continue polling on transient errors
            }
        }
    }

    Err(VideoGenError::NetworkError(
        "Video generation timed out after 5 minutes".to_string(),
    ))
}
