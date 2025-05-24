use auth::logout_identity;
use codee::string::FromToStringCodec;
use component::loading::Loading;
use consts::{ACCOUNT_CONNECTED_STORE, NOTIFICATIONS_ENABLED_STORE};
use leptos::prelude::*;
use leptos_router::components::Redirect;
use leptos_use::storage::use_local_storage;
use state::auth::auth_state;
use utils::{
    event_streaming::events::{auth_canisters_store, LogoutClicked, LogoutConfirmation},
    try_or_redirect_opt,
};

#[component]
pub fn Logout() -> impl IntoView {
    let canister_store = auth_canisters_store();

    LogoutClicked.send_event(canister_store);
    let auth = auth_state();
    let (_, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);
    let auth_res = Resource::new_blocking(
        || (),
        move |_| async move {
            let id = try_or_redirect_opt!(logout_identity().await);

            LogoutConfirmation.send_event(canister_store);

            let (_, write_account_connected, _) =
                use_local_storage::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
            write_account_connected(false);
            set_notifs_enabled(false);
            Some(id)
        },
    );

    view! {
        <Loading text="Logging out...".to_string()>
            <Suspense>
                {move || {
                    auth_res
                        .get()
                        .flatten()
                        .map(|id| {
                            auth.set(Some(id));
                            view! { <Redirect path="/menu" /> }
                        })
                }}

            </Suspense>
        </Loading>
    }
}
