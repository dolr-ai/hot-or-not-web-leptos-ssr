use leptos::{prelude::ServerFnError, server};

#[server]
pub async fn trigger_server_error() -> Result<(), ServerFnError> {
    log::info!("---> Intentionally causing an error on the server!");

    // Option 1: Cause a panic (Sentry will capture this)
    panic!("Sentry Server Test: This is a deliberate panic from a server function!");

    // Option 2: Return a ServerFnError (Sentry should also capture this if configured)
    // Err(ServerFnError::ServerError("Sentry Server Test: Deliberate error returned from server function!".to_string()))
}