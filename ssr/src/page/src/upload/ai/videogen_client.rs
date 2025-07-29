use consts::OFF_CHAIN_AGENT_URL;
use gloo::timers::future::TimeoutFuture;
use videogen_common::{
    VideoGenClient, VideoGenError, VideoGenQueuedResponse, VideoGenRequest, VideoGenRequestStatus,
    VideoGenRequestWithSignature, VideoGenResponse,
};
use yral_canisters_client::rate_limits::RateLimits;

/// Create a message for videogen request signing
pub fn videogen_request_msg(request: VideoGenRequest) -> yral_identity::msg_builder::Message {
    yral_identity::msg_builder::Message::default()
        .method_name("videogen_generate".into())
        .args((request,))
        .expect("VideoGen request should serialize")
}

/// Sign a videogen request with the sender's identity
pub fn sign_videogen_request(
    sender: &impl ic_agent::Identity,
    request: VideoGenRequest,
) -> yral_identity::Result<VideoGenRequestWithSignature> {
    use yral_identity::ic_agent::sign_message;
    let msg = videogen_request_msg(request.clone());
    let signature = sign_message(sender, msg)?;

    Ok(VideoGenRequestWithSignature::new_with_signature(
        request, signature,
    ))
}

/// Generate video using the signature-based flow
/// The off-chain agent will handle signature verification and balance deduction
pub async fn generate_video_with_signature(
    signed_request: VideoGenRequestWithSignature,
    rate_limits: &RateLimits<'_>,
) -> Result<VideoGenResponse, VideoGenError> {
    // Create client and call the signed endpoint
    let client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.clone());

    // Get the queued response with request_key
    let queued_response: VideoGenQueuedResponse = client
        .generate_with_signature(signed_request)
        .await
        .map_err(|e| {
            leptos::logging::log!("Error generating video: {}", e);
            e
        })?;

    // Extract request_key from the response
    let request_key = queued_response.request_key;

    // Start polling for status
    poll_video_status(&client, &request_key, rate_limits).await
}

/// Poll the video generation status with exponential backoff
async fn poll_video_status(
    client: &VideoGenClient,
    request_key: &videogen_common::VideoGenRequestKey,
    rate_limits: &RateLimits<'_>,
) -> Result<VideoGenResponse, VideoGenError> {
    // Polling intervals: 2s, 4s, 8s, 16s, 30s, then every 30s
    let mut poll_interval_ms = 2000;
    const MAX_POLL_INTERVAL_MS: u32 = 30000;
    const MAX_ATTEMPTS: u32 = 20; // Maximum 10 minutes of polling

    for attempt in 0..MAX_ATTEMPTS {
        // Wait before polling (except on first attempt)
        if attempt > 0 {
            TimeoutFuture::new(poll_interval_ms).await;
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

        // Increase interval with exponential backoff
        poll_interval_ms = (poll_interval_ms * 2).min(MAX_POLL_INTERVAL_MS);
    }

    Err(VideoGenError::NetworkError(
        "Video generation timed out after 10 minutes".to_string(),
    ))
}
