use codee::string::FromToStringCodec;
use consts::NOTIFICATIONS_ENABLED_STORE;
use leptos::html::Input;
use leptos::web_sys::{Notification, NotificationPermission};
use leptos::{ev, prelude::*};
use leptos_icons::*;
use leptos_use::storage::use_local_storage;
use leptos_use::use_event_listener;
use state::canisters::auth_state;
use utils::mixpanel::mixpanel_events::*;
use utils::notifications::{
    get_device_registeration_token, get_fcm_token, notification_permission_granted,
};
use yral_metadata_client::MetadataClient;

use crate::toggle::Toggle;

#[component]
pub fn NotificationToggle(
    #[prop(optional)] show_icon: bool,
    #[prop(optional)] show_label: bool,
    #[prop(optional)] icon: Option<icondata::Icon>,
    #[prop(optional)] label_text: Option<String>,
    #[prop(optional)] custom_class: Option<String>,
) -> impl IntoView {
    // Default values
    let show_icon = show_icon;
    let show_label = show_label;
    let icon = icon.unwrap_or(icondata::BiCommentDotsRegular);
    let label_text = label_text.unwrap_or_else(|| "Enable Notifications".to_string());
    let custom_class = custom_class.unwrap_or_else(|| "flex items-center justify-between w-full".to_string());

    // Notifications state management
    let (notifs_enabled, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let notifs_enabled_signal = Signal::derive(move || {
        notifs_enabled.get()
            && matches!(Notification::permission(), NotificationPermission::Granted)
    });

    let toggle_ref = NodeRef::<Input>::new();
    let auth = auth_state();

    // Main notification toggle action
    let on_toggle_action: Action<(), ()> = Action::new_unsync(move |()| async move {
        // Check if user is authenticated
        let Ok(cans) = auth.auth_cans().await else {
            log::error!("User must be authenticated to enable notifications");
            return;
        };

        let metaclient: MetadataClient<false> = MetadataClient::default();
        let browser_permission = Notification::permission();
        let notifs_enabled_val = notifs_enabled.get_untracked();

        // Track analytics
        let global =
            MixpanelGlobalProps::try_get(&cans, auth.is_logged_in_with_oauth().get_untracked());
        MixPanelEvent::track_enable_notifications(global, notifs_enabled_val);

        // Handle notification toggle logic
        if notifs_enabled_val && matches!(browser_permission, NotificationPermission::Default) {
            // Request permission if in default state
            match notification_permission_granted().await {
                Ok(true) => {
                    match get_fcm_token().await {
                        Ok(token) => {
                            match metaclient.register_device(cans.identity(), token).await {
                                Ok(_) => {
                                    log::info!("Device registered successfully");
                                    set_notifs_enabled(true);
                                }
                                Err(e) => {
                                    log::error!("Failed to register device: {e:?}");
                                    set_notifs_enabled(false);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to get FCM token: {e:?}");
                            set_notifs_enabled(false);
                        }
                    }
                }
                Ok(false) => {
                    log::warn!("User did not grant notification permission");
                    set_notifs_enabled(false);
                }
                Err(e) => {
                    log::error!("Failed to check notification permission: {e:?}");
                    set_notifs_enabled(false);
                }
            }
        } else if notifs_enabled_val {
            // Unregister device
            match get_device_registeration_token().await {
                Ok(token) => {
                    match metaclient.unregister_device(cans.identity(), token).await {
                        Ok(_) => {
                            log::info!("Device unregistered successfully");
                            set_notifs_enabled(false);
                        }
                        Err(e) => {
                            // Check if it's a device not found error by examining the error message
                            if format!("{e:?}").contains("DeviceNotFound") {
                                log::info!("Device not found, skipping unregister");
                                set_notifs_enabled(false);
                            } else {
                                log::error!("Failed to unregister device: {e:?}");
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get device token for unregister: {e:?}");
                    set_notifs_enabled(false);
                }
            }
        } else {
            // Register device
            match get_device_registeration_token().await {
                Ok(token) => {
                    match metaclient.register_device(cans.identity(), token).await {
                        Ok(_) => {
                            log::info!("Device registered successfully");
                            set_notifs_enabled(true);
                        }
                        Err(e) => {
                            log::error!("Failed to register device: {e:?}");
                            set_notifs_enabled(false);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to get device token: {e:?}");
                    set_notifs_enabled(false);
                }
            }
        }
    });

    // Listen for toggle changes
    _ = use_event_listener(toggle_ref, ev::change, move |_| {
        on_toggle_action.dispatch(());
    });

    if show_icon || show_label {
        view! {
            <div class=custom_class>
                <div class="flex flex-row gap-4 items-center flex-1">
                    {show_icon.then(|| view! { <Icon attr:class="text-2xl flex-shrink-0" icon=icon /> })}
                    {show_label.then(|| view! { <span class="text-wrap">{label_text}</span> })}
                </div>
                <div class="flex-shrink-0">
                    <Toggle checked=notifs_enabled_signal node_ref=toggle_ref />
                </div>
            </div>
        }.into_any()
    } else {
        view! {
            <div class=custom_class>
                <div class="flex-shrink-0">
                    <Toggle checked=notifs_enabled_signal node_ref=toggle_ref />
                </div>
            </div>
        }.into_any()
    }
}