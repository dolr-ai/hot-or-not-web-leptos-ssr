use std::env;

use leptos::prelude::*;
use serde_json::json;

#[cfg(all(feature = "ssr", feature = "ga4"))]
use tracing::instrument;

pub mod events;

#[cfg(feature = "ssr")]
pub mod warehouse_events {
    tonic::include_proto!("warehouse_events");
}

#[derive(Clone, Default)]
pub struct EventHistory {
    pub event_name: RwSignal<String>,
}

#[cfg(feature = "ga4")]
#[server]
pub async fn send_event_ssr(event_name: String, params: String) -> Result<(), ServerFnError> {
    use super::host::get_host;

    let params = serde_json::from_str::<serde_json::Value>(&params).map_err(|e| {
        log::error!("Error parsing params: {e:?}");
        ServerFnError::new(e.to_string())
    })?;

    let host_str = get_host();
    let mut params = params.clone();
    params["host"] = json!(host_str);

    if params["page_location"].is_null() {
        params["page_location"] = json!(format!("https://{}", host_str));
    }

    // Warehouse
    send_event_warehouse(&event_name, &params).await;

    Ok(())
}

#[cfg(feature = "ga4")]
pub fn send_event_ssr_spawn(event_name: String, params: String) -> Result<(), ServerFnError> {
    use leptos::task::spawn_local;

    let mut params = serde_json::from_str::<serde_json::Value>(&params).map_err(|e| {
        log::error!("Error parsing params: {e:?}");
        ServerFnError::new(e.to_string())
    })?;
    params["page_location"] = json!(window().location().href().map_err(|e| {
        let error_msg = format!("Error getting page location: {e:?}");
        log::error!("{error_msg}");
        ServerFnError::new(error_msg)
    })?);
    let params = serde_json::to_string(&params).map_err(|e| {
        log::error!("Error serializing params: {e:?}");
        ServerFnError::new(e.to_string())
    })?;

    spawn_local(async move {
        let _ = send_event_ssr(event_name, params).await;
    });

    Ok(())
}

#[cfg(all(feature = "ga4", feature = "ssr"))]
#[instrument]
pub async fn send_event_warehouse(event_name: &str, params: &serde_json::Value) {
    use super::host::get_host;

    let event_name = event_name.to_string();

    let mut params = params.clone();
    if params["host"].is_null() {
        let host_str = get_host();
        params["host"] = json!(host_str);
    }

    let res = stream_to_offchain_agent(event_name, &params).await;
    if let Err(e) = res {
        log::error!("Error sending event to warehouse: {e:?}");
    }
}

#[cfg(feature = "ga4")]
#[server]
pub async fn send_event_warehouse_ssr(
    event_name: String,
    params: String,
) -> Result<(), ServerFnError> {
    let params = serde_json::from_str::<serde_json::Value>(&params).map_err(|e| {
        log::error!("Error parsing params: {e:?}");
        ServerFnError::new(e.to_string())
    })?;
    send_event_warehouse(&event_name, &params).await;

    Ok(())
}

#[cfg(feature = "ga4")]
pub fn send_event_warehouse_ssr_spawn(event_name: String, params: String) {
    use leptos::task::spawn_local;

    spawn_local(async move {
        let _ = send_event_warehouse_ssr(event_name, params).await;
    });
}

#[cfg(all(feature = "ga4", feature = "ssr"))]
#[instrument]
pub async fn stream_to_offchain_agent(
    event: String,
    params: &serde_json::Value,
) -> Result<(), ServerFnError> {
    use tonic::metadata::MetadataValue;
    use tonic::transport::Channel;
    use tonic::Request;

    let channel: Channel = expect_context();

    let mut off_chain_agent_grpc_auth_token = env::var("GRPC_AUTH_TOKEN").expect("GRPC_AUTH_TOKEN");
    // removing whitespaces and new lines for proper parsing
    off_chain_agent_grpc_auth_token.retain(|c| !c.is_whitespace());

    let token: MetadataValue<_> = format!("Bearer {off_chain_agent_grpc_auth_token}").parse()?;

    let mut client =
        warehouse_events::warehouse_events_client::WarehouseEventsClient::with_interceptor(
            channel,
            move |mut req: Request<()>| {
                req.metadata_mut().insert("authorization", token.clone());
                Ok(req)
            },
        );

    let params = params.to_string();
    let request = tonic::Request::new(warehouse_events::WarehouseEvent { event, params });

    client.send_event(request).await?;

    Ok(())
}
