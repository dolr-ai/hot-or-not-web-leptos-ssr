use consts::OFF_CHAIN_AGENT_URL;
use videogen_common::{
    VideoGenClient, VideoGenError, VideoGenRequest, VideoGenRequestWithSignature, VideoGenResponse,
};

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
) -> Result<VideoGenResponse, VideoGenError> {
    // TODO: Remove this dummy implementation later when feature is stable
    // Sleep for 2 seconds to simulate processing
    // gloo::timers::future::TimeoutFuture::new(2_000).await;

    // // // Return dummy response
    // let dummy_response = VideoGenResponse {
    //     operation_id: format!("dummy-op-{}", uuid::Uuid::new_v4()),
    //     video_url: "https://storage.googleapis.com/yral_ai_generated_videos/veo-output/5790230970440583959/sample_0.mp4".to_string(),
    //     provider: "dummy".to_string(),
    // };

    // Ok(dummy_response)

    // Create client and call the signed endpoint
    let client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.clone());

    let video_response = client
        .generate_with_signature(signed_request)
        .await
        .map_err(|e| {
            leptos::logging::log!("Error generating video: {}", e);
            e
        })?;

    Ok(video_response)
}
