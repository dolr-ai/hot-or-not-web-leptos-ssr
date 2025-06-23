use leptos::prelude::ServerFnError;
use yral_types::delegated_identity::DelegatedIdentityWire;

pub async fn initiate_delete_user(identity: DelegatedIdentityWire) -> Result<(), ServerFnError> {
    use reqwest::Client;
    use serde_json::json;

    let client = Client::new();
    let body = json!({
        "delegated_identity_wire": identity
    });

    let response = client
        .delete("https://icp-off-chain-agent.fly.dev/api/v1/user")
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::ServerError(format!(
            "Delete user failed with status: {}",
            response.status()
        )))
    }
}
