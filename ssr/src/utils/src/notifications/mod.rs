use candid::Principal;
use leptos::{prelude::ServerFnError, server};
use serde_json::json;
use wasm_bindgen::prelude::*;
use yral_metadata_types::{
    AndroidConfig, AndroidNotification, ApnsConfig, ApnsFcmOptions, DeviceRegistrationToken,
    NotificationPayload, SendNotificationReq, WebpushConfig, WebpushFcmOptions,
};

#[wasm_bindgen(module = "/src/notifications/inline-js/setup-firebase-messaging-inline.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = getToken)]
    pub async fn get_token() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = getNotificationPermission)]
    pub async fn get_notification_permission() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_name = deleteFcmToken)]
    pub async fn delete_fcm_token_js() -> Result<JsValue, JsValue>;
}

pub async fn delete_fcm_token() -> Result<bool, ServerFnError> {
    let deleted = delete_fcm_token_js()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;
    deleted.as_bool().ok_or(ServerFnError::new(
        "Failed to parse delete_fcm_token result",
    ))
}

pub async fn notification_permission_granted() -> Result<bool, ServerFnError> {
    let permission = get_notification_permission()
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?
        .as_bool()
        .ok_or(ServerFnError::new("Failed to get notification permission"))?;
    Ok(permission)
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

const METADATA_SERVER_URL: &str = "https://yral-metadata.fly.dev";

#[derive(Clone)]
pub struct NotificationClient;

pub enum NotificationType {
    Liked(Principal, u64),
}
impl NotificationClient {
    pub async fn send_liked_notification(
        &self,
        liked_by: Principal,
        post_id: u64,
        creator: Principal,
        creator_cans: Principal,
    ) -> Result<(), ServerFnError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/notifications/{}/send",
            METADATA_SERVER_URL,
            creator.to_text()
        );

        let res = client
            .post(&url)
            .bearer_auth(std::env::var("YRAL_METADATA_NOTIFICATION_API_KEY").expect("YRAL_METADATA_NOTIFICATION_API_KEY is not set"))
            .json(&SendNotificationReq{
                notification: Some(NotificationPayload{
                    title: Some("Liked your post".to_string()),
                    body: Some(format!("{} liked your post", liked_by.to_text())),
                    image: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                }),
                android: Some(AndroidConfig{
                    notification: Some(AndroidNotification{
                        icon: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                        image: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                        click_action: Some(format!("https://yral.com/hot-or-not/{}/{}", creator_cans.to_text(), post_id)),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                webpush: Some(WebpushConfig{
                    fcm_options: Some(WebpushFcmOptions{
                        link: Some(format!("https://yral.com/hot-or-not/{}/{}", creator_cans.to_text(), post_id)),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                apns: Some(ApnsConfig{
                    fcm_options: Some(ApnsFcmOptions{
                        image: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                        ..Default::default()
                    }),
                    payload: Some(json!({
                        "aps": {
                            "alert": {
                                "title": "Liked your post".to_string(),
                                "body": format!("{} liked your post", liked_by.to_text()),
                            },
                            "sound": "default",
                        },
                        "url": format!("https://yral.com/hot-or-not/{}/{}", creator_cans.to_text(), post_id)
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .send()
            .await;

        match res {
            Ok(response) => {
                if !response.status().is_success() {
                    if let Ok(body) = response.text().await {
                        log::error!("Response body: {body}");
                    }
                }

                Ok(())
            }
            Err(req_err) => {
                log::error!("Error sending notification request for video: {req_err}");
                Err(ServerFnError::new(format!(
                    "Error sending notification request for video: {req_err}"
                )))
            }
        }
    }
}

#[server]
pub async fn send_liked_notification(
    liked_by: Principal,
    post_id: u64,
    creator: Principal,
    creator_cans: Principal,
) -> Result<(), ServerFnError> {
    NotificationClient
        .send_liked_notification(liked_by, post_id, creator, creator_cans)
        .await
}
