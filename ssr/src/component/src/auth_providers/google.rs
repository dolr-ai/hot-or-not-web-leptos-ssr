use leptos::{ev, prelude::*};
use leptos_icons::*;
use leptos_use::{use_event_listener, use_interval_fn, use_window};
use utils::{host::show_preview_component, icon_gen};
use yral_types::delegated_identity::DelegatedIdentityWire;
pub type GoogleAuthMessage = Result<DelegatedIdentityWire, String>;

use super::{LoginProvButton, LoginProvCtx, ProviderKind};

icon_gen!(
    GoogleLogoSymbol,
    view_box = "0 0 48 48",
    r###"<path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"></path><path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"></path><path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"></path><path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"></path><path fill="none" d="M0 0h48v48H0z"></path>"###
);

#[component]
pub fn GoogleAuthProvider() -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();
    let current_text = move || {
        if ctx.processing.get() == Some(ProviderKind::Google) {
            "Signing In..."
        } else {
            "Google Sign-In"
        }
    };
    let done_guard = RwSignal::new(false);
    let close_popup_store = StoredValue::new(None::<Callback<()>>);
    let close_popup =
        move || _ = close_popup_store.with_value(|cb| cb.as_ref().map(|close_cb| close_cb.run(())));

    let on_click = move || {
        let window = window();
        let origin = window.origin();
        let redirect_uri = if show_preview_component() {
            format!("{origin}/preview/auth/perform_google_redirect")
        } else {
            format!("{origin}/auth/perform_google_redirect")
        };
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
            let res = match serde_json::from_str::<GoogleAuthMessage>(&data)
                .map_err(|e| e.to_string())
                .and_then(|r| r)
            {
                Ok(res) => res,
                Err(e) => {
                    log::warn!("error processing {:?}. msg {data}", e);
                    close_popup();
                    return;
                }
            };
            done_guard.set(true);
            (pause.pause)();
            _ = target_c.close();
            ctx.set_processing.set(None);
            ctx.login_complete.set(res);
        });
    };

    view! {
        <LoginProvButton
            prov=ProviderKind::Google
            class="flex flex-row justify-center items-center justify-between gap-2 rounded-full bg-neutral-600 pr-4"
            on_click=move |ev| {
                ev.stop_propagation();
                on_click()
            }
        >

            <div class="grid grid-cols-1 place-items-center bg-white p-2 rounded-full">
                <Icon attr:class="text-xl rounded-full" icon=GoogleLogoSymbol />
            </div>
            <span class="text-white">{current_text}</span>
        </LoginProvButton>
    }
}
