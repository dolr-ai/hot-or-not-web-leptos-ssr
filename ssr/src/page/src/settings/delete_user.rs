use consts::OFF_CHAIN_AGENT_URL;
use leptos::prelude::ServerFnError;
use reqwest::Client;
use serde_json::json;
use yral_types::delegated_identity::DelegatedIdentityWire;

pub async fn initiate_delete_user(identity: DelegatedIdentityWire) -> Result<(), ServerFnError> {
    let client = Client::new();
    let body = json!({
        "delegated_identity_wire": identity
    });

    let url = OFF_CHAIN_AGENT_URL.join("api/v1/user").unwrap();

    let response = client.delete(url).json(&body).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::ServerError(format!(
            "Delete user failed with status: {}",
            response.status()
        )))
    }
}
