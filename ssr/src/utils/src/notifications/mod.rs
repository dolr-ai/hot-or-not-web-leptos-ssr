use leptos::prelude::ServerFnError;
use wasm_bindgen::prelude::*;
use yral_metadata_types::DeviceRegistrationToken;

pub mod device_id;

#[wasm_bindgen(module = "/src/notifications/setup-firebase-messaging-inline.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getToken)]
    pub async fn get_token() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = getNotificationPermission)]
    pub async fn get_notification_permission() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = deleteFcmToken)]
    pub async fn delete_fcm_token_js() -> Result<JsValue, JsValue>;
}

/// Deletes the FCM token for this device/browser
pub async fn delete_fcm_token() -> Result<bool, ServerFnError> {
    let deleted = delete_fcm_token_js()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;
    deleted.as_bool().ok_or(ServerFnError::new(
        "Failed to parse delete_fcm_token result",
    ))
}

/// Checks if notification permission is granted.
pub async fn notification_permission_granted() -> Result<bool, ServerFnError> {
    let permission = get_notification_permission()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?
        .as_bool()
        .ok_or(ServerFnError::new("Failed to get notification permission"))?;
    Ok(permission)
}

/// Gets the FCM token, assumes permission is already granted.
pub async fn get_fcm_token() -> Result<DeviceRegistrationToken, ServerFnError> {
    let token = get_token()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?
        .as_string()
        .ok_or(ServerFnError::new("Failed to get token"))?;
    Ok(DeviceRegistrationToken { token })
}

/// Checks permission, then gets the FCM token if allowed.
pub async fn get_device_registeration_token() -> Result<DeviceRegistrationToken, ServerFnError> {
    let permission = notification_permission_granted().await?;
    if !permission {
        // TODO: show a notification to the user to allow notifications
        log::warn!("Notification permission not granted");
        return Err(ServerFnError::new("Notification permission not granted"));
    }
    get_fcm_token().await
}
