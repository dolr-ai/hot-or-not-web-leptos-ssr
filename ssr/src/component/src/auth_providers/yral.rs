use codee::string::FromToStringCodec;
use consts::NOTIFICATIONS_ENABLED_STORE;
use ic_agent::identity::DelegatedIdentity;
use leptos::{ev, prelude::*};
use leptos_use::{storage::use_local_storage, use_event_listener, use_interval_fn, use_window};
use state::canisters::auth_state;
use yral_canisters_common::yral_auth_login_hint;
use yral_types::delegated_identity::DelegatedIdentityWire;

pub type YralAuthMessage = Result<DelegatedIdentityWire, String>;

use super::{LoginProvButton, LoginProvCtx, ProviderKind};

#[server]
async fn yral_auth_login_url(login_hint: String) -> Result<String, ServerFnError> {
    use auth::server_impl::yral::yral_auth_url_impl;
    use auth::server_impl::yral::YralOAuthClient;

    let oauth2: YralOAuthClient = expect_context();

    let url = yral_auth_url_impl(oauth2, login_hint, None).await?;

    Ok(url)
}

#[component]
pub fn YralAuthProvider() -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();
    let current_text = move || {
        if ctx.processing.get() == Some(ProviderKind::YralAuth) {
            "Signing In..."
        } else {
            "Yral Sign-In"
        }
    };
    let done_guard = RwSignal::new(false);
    let close_popup_store = StoredValue::new(None::<Callback<()>>);
    let close_popup =
        move || _ = close_popup_store.with_value(|cb| cb.as_ref().map(|close_cb| close_cb.run(())));
    let (_, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let auth = auth_state();

    let open_yral_auth = Action::new_unsync_local(
        move |(target, origin): &(leptos::web_sys::Window, String)| {
            let target = target.clone();
            let origin = origin.clone();

            let url_fut = async move {
                let id_wire = auth.user_identity.await?;
                let id = DelegatedIdentity::try_from(id_wire)?;
                let login_hint = yral_auth_login_hint(&id)?;

                yral_auth_login_url(login_hint).await
            };

            async move {
                let url = match url_fut.await {
                    Ok(url) => url,
                    Err(e) => {
                        format!("{origin}/error?err={e}")
                    }
                };
                target
                    .location()
                    .replace(&url)
                    .expect("Failed to open Yral Auth?!");
            }
        },
    );

    let on_click = move || {
        let window = window();
        let origin = window.origin();

        // open a target window
        let target = window.open().transpose().and_then(|w| w.ok()).unwrap();

        // load yral auth url in background
        open_yral_auth.dispatch_local((target.clone(), origin.clone()));

        // Check if the target window was closed by the user
        let target_c = target.clone();
        let pause = use_interval_fn(
            move || {
                // Target window was closed by user
                if target_c.closed().unwrap_or_default() && !done_guard.try_get().unwrap_or(true) {
                    ctx.set_processing.try_set(None);
                }
            },
            500,
        );

        _ = use_event_listener(use_window(), ev::message, move |msg| {
            if msg.origin() != origin {
                return;
            }

            let Some(data) = msg.data().as_string() else {
                log::warn!("received invalid message: {:?}", msg.data());
                return;
            };
            let res = match serde_json::from_str::<YralAuthMessage>(&data)
                .map_err(|e| e.to_string())
                .and_then(|r| r)
            {
                Ok(res) => res,
                Err(e) => {
                    log::warn!("error processing {e:?}. msg {data}");
                    close_popup();
                    return;
                }
            };
            done_guard.set(true);
            (pause.pause)();
            _ = target.close();
            ctx.set_processing.set(None);
            set_notifs_enabled.set(false);
            ctx.login_complete.set(res);
        });
    };

    view! {
        <LoginProvButton
            prov=ProviderKind::YralAuth
            class="flex flex-row gap-2 justify-center justify-between items-center pr-4 rounded-full bg-neutral-600"
            on_click=move |ev| {
                ev.stop_propagation();
                on_click()
            }
        >
            <img class="w-9 h-9 rounded-full" src="/img/yral/logo.webp" />
            <span class="text-white">{current_text}</span>
        </LoginProvButton>
    }
}
