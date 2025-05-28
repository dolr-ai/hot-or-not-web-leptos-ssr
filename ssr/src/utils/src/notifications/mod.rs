use leptos::prelude::ServerFnError;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use yral_metadata_types::DeviceRegistrationToken;

pub mod device_id;

#[wasm_bindgen(module = "/src/notifications/setup-firebase-messaging-inline.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getToken)]
    pub async fn get_token() -> Result<JsValue, JsValue>;
}

pub async fn notification_permission_granted() -> Result<bool, ServerFnError> {
    let promise = leptos::web_sys::Notification::request_permission().map_err(|e| {
        ServerFnError::new(format!("Failed to request notification permission: {e:?}"))
    })?;
    let js_value = JsFuture::from(promise).await.map_err(|e| {
        ServerFnError::new(format!(
            "Failed to await notification permission promise: {e:?}"
        ))
    })?;

    let permission_string = js_value
        .as_string()
        .ok_or_else(|| ServerFnError::new("Failed to convert permission to string"))?;
    Ok(permission_string == "granted")
}

pub async fn get_fcm_token() -> Result<DeviceRegistrationToken, ServerFnError> {
    let token = get_token()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?
        .as_string()
        .ok_or(ServerFnError::new("Failed to get token"))?;
    Ok(DeviceRegistrationToken { token })
}

pub async fn get_device_registeration_token() -> Result<DeviceRegistrationToken, ServerFnError> {
    let permission = notification_permission_granted().await?;
    if !permission {
        log::warn!("Notification permission not granted");
        return Err(ServerFnError::new("Notification permission not granted"));
    }
    get_fcm_token().await
}
