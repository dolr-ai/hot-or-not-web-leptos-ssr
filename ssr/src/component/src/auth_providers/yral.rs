use codee::string::FromToStringCodec;
use consts::NOTIFICATIONS_ENABLED_STORE;
use leptos::{ev, prelude::*};
use leptos_use::{storage::use_local_storage, use_event_listener, use_interval_fn, use_window};
use yral_types::delegated_identity::DelegatedIdentityWire;

pub type YralAuthMessage = Result<DelegatedIdentityWire, String>;

use super::{LoginProvButton, LoginProvCtx, ProviderKind};

#[component]
pub fn YralAuthProvider() -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();
    let current_text = move || {
        if ctx.processing.get() == Some(ProviderKind::YralAuth) {
            "Signing In..."
        } else {
            "Google Sign-In"
        }
    };
    let done_guard = RwSignal::new(false);
    let close_popup_store = StoredValue::new(None::<Callback<()>>);
    let close_popup =
        move || _ = close_popup_store.with_value(|cb| cb.as_ref().map(|close_cb| close_cb.run(())));
    let (_, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let on_click = move || {
        let window = window();
        let origin = window.origin();
        let redirect_uri = format!("{origin}/auth/perform_google_redirect");

        // Open a popup window with the redirect URL
        let target = window
            .open_with_url(&redirect_uri)
            .transpose()
            .and_then(|w| w.ok())
            .unwrap();

        // Check if the target window was closed by the user
        let target_c = target.clone();
        let pause = use_interval_fn(
            move || {
                // Target window was closed by user
                if target.closed().unwrap_or_default() && !done_guard.try_get().unwrap_or(true) {
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
            _ = target_c.close();
            ctx.set_processing.set(None);
            set_notifs_enabled.set(false);
            ctx.login_complete.set(res);
        });
    };

    view! {
        <LoginProvButton
            prov=ProviderKind::YralAuth
            class="flex flex-row justify-center items-center justify-between gap-2 rounded-full bg-neutral-600 pr-4"
            on_click=move |ev| {
                ev.stop_propagation();
                on_click()
            }
        >

            <div class="grid grid-cols-1 place-items-center bg-white p-2 rounded-full">
                // TODO: Add Yral Logo here
                // <Icon attr:class="text-xl rounded-full" icon=YralLogoSymbol />
            </div>
            <span class="text-white">{current_text}</span>
        </LoginProvButton>
    }
}
