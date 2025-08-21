use consts::auth::REFRESH_MAX_AGE;
use leptos::{ev, prelude::*};
use leptos_router::components::Outlet;
// use leptos_router::hooks::use_navigate;
use leptos_use::{use_cookie_with_options, use_event_listener, use_window, UseCookieOptions};

use codee::string::FromToStringCodec;
use consts::{ACCOUNT_CONNECTED_STORE, NOTIFICATIONS_ENABLED_STORE, NOTIFICATION_MIGRATED_STORE};
use leptos_use::storage::use_local_storage;
use state::audio_state::AudioState;
use state::canisters::AuthState;
use utils::event_streaming::events::PageVisit;
use utils::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps};
use utils::notifications::get_fcm_token;
use utils::sentry::{set_sentry_user, set_sentry_user_canister};
use yral_metadata_client::MetadataClient;

#[derive(Clone)]
pub struct Notification(pub RwSignal<Option<serde_json::Value>>);

#[component]
fn CtxProvider(children: Children) -> impl IntoView {
    let auth = AuthState::default();
    provide_context(auth);

    let location = leptos_router::hooks::use_location();
    // let navigate = use_navigate();
    // // Monitor auth errors and navigate to logout if needed
    // Effect::new(move |_| {
    //     if let Some(Err(_)) = auth.user_identity.get() {
    //         navigate("/logout", Default::default());
    //     }
    // });

    Effect::new(move |_| {
        let user_canister = auth.canisters_resource.read().as_ref().and_then(|c| {
            let cans = c.as_ref().ok()?;
            Some(cans.user_canister().to_string())
        });
        set_sentry_user_canister(user_canister);
    });

    let window_target = use_window();

    let notification = Notification(RwSignal::new(None));

    let _ = use_event_listener(
        window_target,
        ev::Custom::new("firebaseForegroundMessage"),
        move |event: leptos::web_sys::CustomEvent| {
            let payload = event.detail();
            notification.0.set(payload.as_string().and_then(|s| {
                log::info!("Payload: {s}");
                serde_json::from_str(&s).ok()
            }));
        },
    );
    provide_context(notification);

    Effect::new(move |_| {
        let user_principal = auth.user_principal.get();
        let user_principal = user_principal.and_then(|c| c.ok()).map(|c| c.to_text());
        set_sentry_user(user_principal);
    });

    // migrates account connected local storage to cookie
    let (_, set_new_account_connected_store) = use_cookie_with_options::<bool, FromToStringCodec>(
        ACCOUNT_CONNECTED_STORE,
        UseCookieOptions::default()
            .path("/")
            .max_age(REFRESH_MAX_AGE.as_millis() as i64),
    );
    let (old_account_connected_store, _, clear_from_storage) =
        use_local_storage::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
    Effect::new(move |_| {
        if old_account_connected_store.get() {
            set_new_account_connected_store(Some(true));
            clear_from_storage();
        }
    });

    Effect::new(move |_| {
        let pathname = location.pathname.get();
        let is_logged_in = auth.is_logged_in_with_oauth();
        let Some(principal) = auth.user_principal_if_available() else {
            return;
        };
        PageVisit.send_event(principal, is_logged_in.get_untracked(), pathname.clone());
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx()) {
            MixPanelEvent::track_page_viewed(pathname, global);
        }
    });

    // Reset AudioState to muted when navigating away from video pages
    Effect::new(move |_| {
        let pathname = location.pathname.get();
        // Check if we're navigating away from video pages
        let is_video_page = pathname.contains("/hot-or-not/")
            || pathname.contains("/post/")
            || pathname.contains("/profile/") && pathname.contains("/post/");

        if !is_video_page {
            AudioState::reset_to_muted();
        }
    });

    let (notifs_enabled, _, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let (migrated, set_migrated, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATION_MIGRATED_STORE);

    let migrate_notification_proj = Action::new_local(move |_| async move {
        let metaclient: MetadataClient<false> = MetadataClient::default();

        let cans = auth.auth_cans().await.unwrap();
        let token = get_fcm_token().await.unwrap();

        metaclient
            .register_device(cans.identity(), token)
            .await
            .inspect_err(|e| log::error!("Failed to migrate notification project: {e:?}"))
            .unwrap();
        log::info!("Migrated notification project");
        set_migrated(true);
    });

    Effect::new(move |_| {
        if !migrated.get()
            && notifs_enabled.get()
            && matches!(
                leptos::web_sys::Notification::permission(),
                leptos::web_sys::NotificationPermission::Granted
            )
        {
            migrate_notification_proj.dispatch(());
        }
    });

    children()
}

#[component]
pub fn BaseRoute() -> impl IntoView {
    view! {
        <CtxProvider>
            <Outlet />
        </CtxProvider>
    }
}
