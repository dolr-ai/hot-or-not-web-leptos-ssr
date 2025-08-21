use leptos::prelude::*;
use leptos::server;

#[server(endpoint = "healthz")]
pub async fn healthz() -> Result<(), ServerFnError> {
    Ok(())
}
