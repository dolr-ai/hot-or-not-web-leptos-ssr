use auth::logout_identity;
use component::loading::Loading;
use leptos::prelude::*;
use leptos_router::components::Redirect;
use state::canisters::auth_state;
use utils::event_streaming::events::{LogoutClicked, LogoutConfirmation};

#[component]
pub fn Logout() -> impl IntoView {
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    LogoutClicked.send_event(ev_ctx);

    let auth_res = OnceResource::new_blocking(logout_identity());

    view! {
        <Loading text="Logging out...".to_string()>
            <Suspense>
                {move || Suspend::new(async move {
                    let res = auth_res.await;
                    match res {
                        Ok(id) => {
                            auth.set_new_identity(id, false);
                            LogoutConfirmation.send_event(ev_ctx);
                            view! {
                                <Redirect path="/menu" />
                            }
                        },
                        Err(e) => {
                            view! {
                                <Redirect path=format!("/error?err={e}") />
                            }
                        }
                    }
                })}
            </Suspense>
        </Loading>
    }
}
