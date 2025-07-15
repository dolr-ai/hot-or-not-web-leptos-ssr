use auth::logout_identity;
use codee::string::FromToStringCodec;
use component::loading::Loading;
use consts::{DEVICE_ID, NOTIFICATIONS_ENABLED_STORE};
use leptos::prelude::*;
use leptos_router::components::Redirect;
use leptos_use::storage::use_local_storage;
use state::canisters::auth_state;
use utils::event_streaming::events::{LogoutClicked, LogoutConfirmation};

#[component]
pub fn Logout() -> impl IntoView {
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    LogoutClicked.send_event(ev_ctx);

    let auth_res = OnceResource::new_blocking(logout_identity());

    let (_, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let (_, set_device_id, _) = use_local_storage::<String, FromToStringCodec>(DEVICE_ID);

    view! {
        <Loading text="Logging out...".to_string()>
            <Suspense>
                {move || Suspend::new(async move {
                    let res = auth_res.await;
                    match res {
                        Ok(id) => {
                            auth.set_new_identity(id, false);
                            set_notifs_enabled(false);
                            set_device_id(uuid::Uuid::new_v4().to_string());
                            LogoutConfirmation.send_event(ev_ctx);
                            view! { <Redirect path="/menu" /> }
                        }
                        Err(e) => {
                            view! { <Redirect path=format!("/error?err={e}") /> }
                        }
                    }
                })}
            </Suspense>
        </Loading>
    }
}
