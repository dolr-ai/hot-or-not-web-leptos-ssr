use candid::Principal;
use consts::OFF_CHAIN_AGENT_URL;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserRequest {
    pub user_principal: Principal,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// Deletes a user from the off-chain agent
#[cfg(feature = "ssr")]
pub async fn delete_user_offchain(user_principal: Principal) -> Result<(), ServerFnError> {
    use std::env;

    let client = reqwest::Client::new();

    // Construct the DELETE endpoint URL
    let delete_url = OFF_CHAIN_AGENT_URL
        .join("api/v1/user/user")
        .map_err(|e| ServerFnError::new(format!("Failed to construct URL: {e}")))?;

    // Get the authentication token from environment
    let auth_token = env::var("OFF_CHAIN_AGENT_AUTH_TOKEN")
        .or_else(|_| env::var("GRPC_AUTH_TOKEN"))
        .map_err(|_| ServerFnError::new("OFF_CHAIN_AGENT_AUTH_TOKEN not found"))?;

    // Create the request body
    let request_body = DeleteUserRequest { user_principal };

    // Make the DELETE request
    let response = client
        .delete(delete_url.as_str())
        .bearer_auth(&auth_token)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Request failed: {e}")))?;

    // Check if the request was successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to delete user. Status: {status}, Error: {error_text}"
        )));
    }

    // Parse the response
    let delete_response: DeleteUserResponse = response
        .json()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse response: {e}")))?;

    if !delete_response.success {
        return Err(ServerFnError::new(format!(
            "User deletion failed: {}",
            delete_response
                .message
                .unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    Ok(())
}

/// Server function to delete a user
#[server]
pub async fn delete_user(user_principal: Principal) -> Result<(), ServerFnError> {
    delete_user_offchain(user_principal).await
}
