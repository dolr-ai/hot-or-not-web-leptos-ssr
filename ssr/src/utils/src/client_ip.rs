#[cfg(feature = "ssr")]
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
