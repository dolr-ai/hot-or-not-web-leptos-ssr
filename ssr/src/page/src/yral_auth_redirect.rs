use component::auth_providers::yral::YralAuthMessage;
use component::loading::Loading;
use ic_agent::identity::DelegatedIdentity;
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use leptos_router::params::Params;

use openidconnect::CsrfToken;
use serde::{Deserialize, Serialize};
use server_fn::codec::Json;
use state::canisters::auth_state;
use utils::route::go_to_root;
use yral_canisters_common::yral_auth_login_hint;
use yral_types::delegated_identity::DelegatedIdentityWire;

#[server]
async fn yral_auth_redirector(login_hint: String) -> Result<(), ServerFnError> {
    use auth::server_impl::yral::yral_auth_url_impl;
    use auth::server_impl::yral::YralOAuthClient;

    let oauth2: YralOAuthClient = expect_context();

    let url = yral_auth_url_impl(oauth2, login_hint, None).await?;
    leptos_axum::redirect(&url);
    Ok(())
}

#[server(input = Json, output = Json)]
async fn perform_yral_oauth(oauth: OAuthQuery) -> Result<DelegatedIdentityWire, ServerFnError> {
    use auth::server_impl::yral::perform_yral_auth_impl;
    use auth::server_impl::yral::YralOAuthClient;

    let oauth2: YralOAuthClient = expect_context();
    perform_yral_auth_impl(oauth.state, oauth.code, oauth2).await
}

#[derive(Params, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OAuthQuery {
    pub code: String,
    pub state: String,
}

#[component]
pub fn IdentitySender(identity_res: YralAuthMessage) -> impl IntoView {
    Effect::new(move |_| {
        let _id = &identity_res;
        #[cfg(feature = "hydrate")]
        {
            use web_sys::Window;

            let win = window();
            let origin = win.origin();
            let opener = win.opener().unwrap();
            if opener.is_null() {
                go_to_root();
            }
            let opener = Window::from(opener);
            let msg = serde_json::to_string(&_id).unwrap();
            _ = opener.post_message(&msg.into(), &origin);
        }
    });

    view! {
        <div class="h-dvh w-dvw bg-black flex flex-col justify-center items-center gap-10">
            <img class="h-56 w-56 object-contain animate-pulse" src="/img/yral/logo.webp"/>
            <span class="text-2xl text-white/60">Good things come to those who wait...</span>
        </div>
    }
}

async fn handle_oauth_query(oauth_query: OAuthQuery) -> YralAuthMessage {
    let delegated = perform_yral_oauth(oauth_query)
        .await
        .map_err(|e| e.to_string())?;
    Ok(delegated)
}

#[server]
async fn handle_oauth_query_for_external_client(
    client_redirect_uri: String,
    oauth_query: OAuthQuery,
) -> Result<(), ServerFnError> {
    leptos_axum::redirect(&format!(
        "{}?code={}&state={}",
        client_redirect_uri, oauth_query.code, oauth_query.state
    ));
    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
enum RedirectHandlerReturnType {
    Identity(YralAuthMessage),
    ExternalClient(Result<(), String>),
}

#[derive(Serialize, Deserialize)]
struct OAuthState {
    pub csrf_token: CsrfToken,
    pub client_redirect_uri: Option<String>,
}

#[component]
pub fn YralAuthRedirectHandler() -> impl IntoView {
    let query = use_query::<OAuthQuery>();
    let identity_resource = Resource::new_blocking(query, |query_res| async move {
        let Ok(oauth_query) = query_res else {
            return Err("Invalid query".to_string());
        };

        handle_oauth_query(oauth_query).await
    });

    view! {
        <Loading text="Logging out...".to_string()>
            <Suspense>
            {move || Suspend::new(async move {
                let identity_res = identity_resource.await;
                view! { <IdentitySender identity_res /> }
            })}
            </Suspense>
        </Loading>
    }
}

#[component]
pub fn YralAuthRedirector() -> impl IntoView {
    let auth = auth_state();
    let yral_auth_redirect = Resource::new_blocking(
        || (),
        move |_| async move {
            let cans = auth.cans_wire().await?;
            let id = DelegatedIdentity::try_from(cans.id)?;
            let login_hint = yral_auth_login_hint(&id)?;
            yral_auth_redirector(login_hint).await
        },
    );

    let do_close = RwSignal::new(false);
    Effect::new(move |_| {
        if !do_close() {
            return;
        }
        let window = window();
        _ = window.close();
    });

    view! {
        <Suspense>
            {move || {
                if let Some(Err(_)) = yral_auth_redirect.get() {
                    do_close.set(true)
                }
                None::<()>
            }}

        </Suspense>
    }
}
