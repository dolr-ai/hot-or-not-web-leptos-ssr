// use consts::OFF_CHAIN_AGENT_URL; // Uncomment when using real implementation
use videogen_common::{
    // VideoGenClient, // Uncomment when using real implementation
    VideoGenRequest,
    VideoGenRequestWithSignature,
    VideoGenResponse,
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
    _signed_request: VideoGenRequestWithSignature,
) -> Result<VideoGenResponse, Box<dyn std::error::Error>> {
    // TODO: Remove this dummy implementation when ready
    // Sleep for 2 seconds to simulate processing
    gloo::timers::future::TimeoutFuture::new(2_000).await;

    // Return dummy response
    let dummy_response = VideoGenResponse {
        operation_id: format!("dummy-op-{}", uuid::Uuid::new_v4()),
        video_url: "https://storage.googleapis.com/yral_ai_generated_videos/veo-output/5790230970440583959/sample_0.mp4".to_string(),
        provider: "dummy".to_string(),
    };

    Ok(dummy_response)

    // Original implementation (commented out for now):
    /*
    // Create client and call the signed endpoint
    let client = VideoGenClient::new(OFF_CHAIN_AGENT_URL.as_str().to_string());

    let video_response = client.generate_with_signature(signed_request).await?;

    Ok(video_response)
    */
}

// Dummy testing code (commented out)
/*
#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use videogen_common::{VideoGenInput, Veo3AspectRatio};

    #[tokio::test]
    async fn test_video_generation() {
        // Example test for video generation
        let request = VideoGenRequest {
            principal: Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap(),
            input: VideoGenInput::Veo3 {
                prompt: "A beautiful sunset over mountains".to_string(),
                negative_prompt: None,
                image: None,
                aspect_ratio: Veo3AspectRatio::Ratio16x9,
                duration_seconds: 5,
                generate_audio: false,
            },
        };

        // Mock identity for testing
        // let identity = test_identity();
        // let signed_request = sign_videogen_request(&identity, request).unwrap();
        // let response = generate_video_with_signature(signed_request).await.unwrap();
        // println!("Video generated: {}", response.video_url);
    }
}
*/
