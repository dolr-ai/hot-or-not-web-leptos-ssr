use candid::Principal;
use leptos::prelude::*;
use num_bigint::{BigInt, BigUint, Sign};
use serde::Serialize;
use videogen_common::{
    videogen_request_msg, VideoGenClient, VideoGenRequestWithSignature,
    VideoGenResponse,
};

use consts::OFF_CHAIN_AGENT_URL;

const VIDEOGEN_COST_SATS: u64 = 1000; // Cost for video generation in sats

/// Load current balance for user from worker
async fn load_sats_balance(user_principal: Principal) -> Result<BigUint, ServerFnError> {
    let worker_url = "https://yral-hot-or-not.go-bazzinga.workers.dev";
    let req_url = format!("{}/balance/{}", worker_url, user_principal);
    
    let client = reqwest::Client::new();
    let res = client
        .get(&req_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch balance: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Balance fetch failed with status: {}",
            res.status()
        )));
    }

    let balance_str = res
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read balance response: {}", e)))?;

    // Parse the balance from string to BigUint
    balance_str
        .trim()
        .parse::<BigUint>()
        .map_err(|e| ServerFnError::new(format!("Failed to parse balance: {}", e)))
}

/// Deduct balance for video generation
async fn deduct_videogen_balance(
    user_principal: Principal,
    cost_sats: u64,
) -> Result<BigUint, ServerFnError> {
    // Load current balance
    let balance = load_sats_balance(user_principal).await?;
    
    // Check if user has sufficient balance
    let cost_biguint = BigUint::from(cost_sats);
    if balance < cost_biguint {
        return Err(ServerFnError::new("Insufficient balance for video generation"));
    }

    // Create balance update request with negative delta
    let delta = BigInt::from_biguint(Sign::Minus, cost_biguint);
    
    #[derive(Serialize)]
    struct SatsBalanceUpdateRequestV2 {
        previous_balance: BigUint,
        delta: BigInt,
        is_airdropped: bool,
    }
    
    let worker_req = SatsBalanceUpdateRequestV2 {
        previous_balance: balance.clone(),
        delta,
        is_airdropped: false,
    };

    // Send balance update to worker
    let worker_url = "https://yral-hot-or-not.go-bazzinga.workers.dev";
    let req_url = format!("{}/v2/update_balance/{}", worker_url, user_principal);
    let client = reqwest::Client::new();
    let res = client
        .post(&req_url)
        .json(&worker_req)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update balance: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Balance update failed with status: {}",
            res.status()
        )));
    }

    // Return the original balance for potential rollback
    Ok(balance)
}

/// Rollback balance deduction by adding the cost back
async fn rollback_videogen_balance(
    user_principal: Principal,
    original_balance: BigUint,
    cost_sats: u64,
) -> Result<(), ServerFnError> {
    // Create balance update request with positive delta to add the cost back
    let cost_biguint = BigUint::from(cost_sats);
    let delta = BigInt::from_biguint(Sign::Plus, cost_biguint.clone());
    
    #[derive(Serialize)]
    struct SatsBalanceUpdateRequestV2 {
        previous_balance: BigUint,
        delta: BigInt,
        is_airdropped: bool,
    }
    
    // Calculate what the balance should be after deduction for the previous_balance field
    let deducted_balance = &original_balance - &cost_biguint;
    
    let worker_req = SatsBalanceUpdateRequestV2 {
        previous_balance: deducted_balance,
        delta,
        is_airdropped: false,
    };

    // Send balance update to worker
    let worker_url = "https://yral-hot-or-not.go-bazzinga.workers.dev";
    let req_url = format!("{}/v2/update_balance/{}", worker_url, user_principal);
    let client = reqwest::Client::new();
    let res = client
        .post(&req_url)
        .json(&worker_req)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to rollback balance: {}", e)))?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Balance rollback failed with status: {}",
            res.status()
        )));
    }

    Ok(())
}

/// Verify signature for videogen request
fn verify_videogen_request(
    sender: Principal,
    req: &VideoGenRequestWithSignature,
) -> Result<(), ServerFnError> {
    let msg = videogen_request_msg(req.request.clone());
    
    req.signature
        .clone()
        .verify_identity(sender, msg)
        .map_err(|_| ServerFnError::new("Invalid signature"))?;

    Ok(())
}


/// Simple generate video function that accepts a signed request
/// The caller is responsible for creating and signing the request
#[server]
pub async fn generate_video_with_balance_check(
    signed_request: VideoGenRequestWithSignature,
) -> Result<VideoGenResponse, ServerFnError> {
    let user_principal = signed_request.request.principal;

    // Verify signature
    verify_videogen_request(user_principal, &signed_request)?;

    // Deduct balance before generating video - returns original balance for rollback
    let original_balance = deduct_videogen_balance(user_principal, VIDEOGEN_COST_SATS).await?;

    // Call off-chain agent using VideoGenClient with bearer token
    let bearer_token = std::env::var("VIDEOGEN_API_KEY")
        .map_err(|_| ServerFnError::new("VIDEOGEN_API_KEY environment variable not set"))?;
    
    let client = VideoGenClient::with_bearer_token(
        OFF_CHAIN_AGENT_URL.as_str().to_string(),
        bearer_token,
    );
    
    let video_response = client
        .generate(signed_request.request)
        .await
        .map_err(|e| {
            // If video generation fails, rollback the balance deduction
            let rollback_principal = user_principal;
            let rollback_balance = original_balance.clone();
            let rollback_cost = VIDEOGEN_COST_SATS;
            
            // Spawn rollback task - we can't await here since we're in map_err
            tokio::spawn(async move {
                if let Err(rollback_err) = rollback_videogen_balance(rollback_principal, rollback_balance, rollback_cost).await {
                    eprintln!("Failed to rollback balance for user {}: {}", rollback_principal, rollback_err);
                }
            });
            
            ServerFnError::new(format!("Failed to call off-chain agent: {:?}", e))
        })?;

    Ok(video_response)
}