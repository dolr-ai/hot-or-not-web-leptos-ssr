use leptos::prelude::*;

#[server]
pub async fn get_client_ip_from_server() -> Result<Option<String>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let result: Result<HeaderMap, _> = extract().await;

    leptos::logging::log!("Extracted headers: {:?}", result);

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
    use codee::string::JsonSerdeCodec;
    use consts::CLIENT_IP_STORE;
    use leptos::prelude::{GetUntracked, Set};
    use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
    use serde::{Deserialize, Serialize};
    use web_time::{Duration, SystemTime, UNIX_EPOCH};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct IpWithExpiry {
        ip: String,
        expires_at: u64, // Unix timestamp in seconds
    }

    // Set expiry to 24 hours
    const IP_EXPIRY_DURATION_SECS: u64 = 24 * 60 * 60;

    let (stored_data, set_stored_data, _) = use_local_storage_with_options::<
        Option<IpWithExpiry>,
        JsonSerdeCodec,
    >(CLIENT_IP_STORE, UseStorageOptions::default());

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    // Check if we have valid stored data
    if let Some(data) = stored_data.get_untracked() {
        if data.expires_at > current_time && !data.ip.is_empty() {
            leptos::logging::log!("Using cached client IP: {:?}", data.ip);
            return Some(data.ip);
        }
        leptos::logging::log!("Cached IP expired or empty, fetching new one");
    }

    // IP not found or expired, fetch from server
    match get_client_ip_from_server().await {
        Ok(Some(server_ip)) => {
            // Store the IP with expiry
            let ip_data = IpWithExpiry {
                ip: server_ip.clone(),
                expires_at: current_time + IP_EXPIRY_DURATION_SECS,
            };
            set_stored_data.set(Some(ip_data));
            Some(server_ip)
        }
        Ok(None) => None,
        Err(e) => {
            leptos::logging::error!("Failed to get client IP from server: {:?}", e);
            None
        }
    }
}
