use leptos::prelude::*;

#[server]
pub async fn get_client_ip_from_server() -> Result<Option<String>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let result: Result<HeaderMap, _> = extract().await;

    Ok(match result {
        Ok(headers) => headers
            .get("x-forwarded-for")
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string()),
        Err(_) => None,
    })
}

#[cfg(not(feature = "hydrate"))]
pub async fn get_client_ip() -> Option<String> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let result: Result<HeaderMap, _> = extract().await;

    leptos::logging::log!("Extracted headers: {:?}", result);

    match result {
        Ok(headers) => headers
            .get("x-forwarded-for")
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string()),
        Err(_) => None,
    }
}

#[cfg(feature = "hydrate")]
pub async fn get_client_ip() -> Option<String> {
    // IP not found or expired, fetch from server
    match get_client_ip_from_server().await {
        Ok(Some(server_ip)) => Some(server_ip),
        Ok(None) => None,
        Err(e) => {
            leptos::logging::error!("Failed to get client IP from server: {:?}", e);
            None
        }
    }
}
