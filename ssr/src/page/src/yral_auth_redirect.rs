use component::auth_providers::yral::YralAuthMessage;
use component::loading::Loading;
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use leptos_router::params::Params;

use openidconnect::CsrfToken;
use serde::{Deserialize, Serialize};
use server_fn::codec::Json;
use utils::route::go_to_root;
use utils::types::NewIdentity;

#[server(input = Json, output = Json)]
async fn perform_yral_oauth(oauth: OAuthQuery) -> Result<NewIdentity, ServerFnError> {
    use auth::server_impl::yral::perform_yral_auth_impl;
    use auth::server_impl::yral::YralOAuthClient;

    let oauth2: YralOAuthClient = expect_context();
    let (id, fallback_username) = perform_yral_auth_impl(oauth.state, oauth.code, oauth2).await?;
    Ok(NewIdentity {
        id_wire: id,
        fallback_username,
    })
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
        <div class="flex flex-col gap-10 justify-center items-center bg-black h-dvh w-dvw">
            <img class="object-contain w-56 h-56 animate-pulse" src="/img/yral/logo.webp" />
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
