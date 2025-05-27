use consts::auth::REFRESH_MAX_AGE;
use leptos::{ev, prelude::*};
use leptos_router::components::Outlet;
use leptos_use::{use_cookie_with_options, use_event_listener, use_window, UseCookieOptions};

use codee::string::FromToStringCodec;
use consts::ACCOUNT_CONNECTED_STORE;
use leptos_use::storage::use_local_storage;
use state::canisters::AuthState;
use utils::event_streaming::events::PageVisit;
use utils::sentry::{set_sentry_user, set_sentry_user_canister};
use web_sys::CustomEvent;

#[derive(Clone)]
pub struct Notification(pub RwSignal<Option<serde_json::Value>>);

#[component]
fn CtxProvider(children: Children) -> impl IntoView {
    let auth = AuthState::default();
    provide_context(auth);

    let location = leptos_router::hooks::use_location();

    Effect::new(move |_| {
        let maybe_user_canister = auth.user_canister.get();
        let user_canister = maybe_user_canister
            .and_then(|c| c.ok())
            .map(|c| c.to_text());
        set_sentry_user_canister(user_canister);
    });

    let window_target = use_window();

    let notification = Notification(RwSignal::new(None));

    let _ = use_event_listener(
        window_target,
        ev::Custom::new("firebaseForegroundMessage"),
        move |event: CustomEvent| {
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
        PageVisit.send_event(principal, is_logged_in.get_untracked(), pathname);
    });

    children()
}

#[component]
pub fn BaseRoute() -> impl IntoView {
    view! {
        <CtxProvider>
            <Outlet/>
        </CtxProvider>
    }
}
