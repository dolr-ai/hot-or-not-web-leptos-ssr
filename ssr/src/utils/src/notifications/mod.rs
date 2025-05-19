use leptos::prelude::ServerFnError;
use wasm_bindgen::prelude::*;
use yral_metadata_types::DeviceRegistrationToken;

pub mod device_id;

#[wasm_bindgen(module = "/src/notifications/setup-firebase-messaging-inline.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getToken)]
    async fn get_token() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = getNotificationPermission)]
    async fn get_notification_permission() -> Result<JsValue, JsValue>;
}

pub async fn get_device_registeration_token() -> Result<DeviceRegistrationToken, ServerFnError> {
    let permission = get_notification_permission()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?
        .as_bool()
        .ok_or(ServerFnError::new("Failed to get notification permission"))?;
    if !permission {
        // TODO: show a notification to the user to allow notifications
        log::warn!("Notification permission not granted");
        return Err(ServerFnError::new("Notification permission not granted"));
    }

    let token = get_token()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?
        .as_string()
        .ok_or(ServerFnError::new("Failed to get token"))?;

    Ok(DeviceRegistrationToken { token })
}
