use ic_agent::identity::DelegatedIdentity;
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use serde::{Deserialize, Serialize};
use yral_types::delegated_identity::DelegatedIdentityWire;

use crate::google_redirect::{IdentitySender, OAuthQuery};
use component::loading::Loading;
use utils::host::get_host;

#[server]
async fn preview_server_set_refersh_token_cookie(
    delegated_identity_wire: DelegatedIdentityWire,
) -> Result<(), ServerFnError> {
    use auth::server_impl::update_user_identity;
    use axum_extra::extract::{cookie::Key, SignedCookieJar};
    use leptos_axum::{extract_with_state, ResponseOptions};

    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    let response_options: ResponseOptions = expect_context();

    let delegated_identity: DelegatedIdentity =
        DelegatedIdentity::try_from(delegated_identity_wire)?;

    update_user_identity(&response_options, jar, &delegated_identity)
}

async fn get_google_auth_url(host: String) -> Result<String, ServerFnError> {
    let client_redirect_uri = format!("https://{host}/preview/auth/google_redirect");
    let url =
        format!("https://yral.com/api/google_auth_url?client_redirect_uri={client_redirect_uri}");

    let client = reqwest::Client::new();

    let mut request = client.get(url);

    request = {
        #[cfg(target_arch = "wasm32")]
        {
            request.fetch_credentials_include()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            request
        }
    };

    let redirect_url: String = request.send().await?.json().await?;

    Ok(redirect_url)
}

#[server]
async fn preview_google_auth_redirector(redirect_url: String) -> Result<(), ServerFnError> {
    leptos_axum::redirect(&redirect_url);
    Ok(())
}

#[component]
pub fn PreviewGoogleRedirector() -> impl IntoView {
    let host = get_host();
    let google_redirect = LocalResource::new(move || {
        let host = host.clone();
        async move {
            let url = get_google_auth_url(host).await?;
            Ok::<String, ServerFnError>(url)
        }
    });

    LocalResource::new(move || async move {
        let url_res = google_redirect.get();
        let url_res = url_res.map(|u| u.take()).transpose()?;
        if let Some(redirect_url) = url_res {
            preview_google_auth_redirector(redirect_url).await?;
            return Ok(());
        }
        Ok::<(), ServerFnError>(())
    });

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
                if let Some(Err(err)) = google_redirect.get().map(|res| res.take()) {
                    log::info!("Error Redirecting {err}")
                }
                None::<()>
            }}

        </Suspense>
    }
}

#[derive(Serialize, Deserialize)]
struct GoogleAuthRequestBody {
    oauth: OAuthQuery,
}

async fn preview_handle_oauth_query(
    oauth_query: OAuthQuery,
) -> Result<DelegatedIdentityWire, ServerFnError> {
    let client: reqwest::Client = reqwest::Client::new();

    let yral_url = "https://yral.com/api/perform_google_auth".to_string();

    let oauth_request_body = GoogleAuthRequestBody { oauth: oauth_query };

    let mut request = client.post(yral_url).json(&oauth_request_body);

    request = {
        #[cfg(target_arch = "wasm32")]
        {
            request.fetch_credentials_include()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            request
        }
    };

    let response = request.send().await?;

    if response.status().is_success() {
        let identity_wire: DelegatedIdentityWire = response.json().await?;
        Ok(identity_wire)
    } else {
        let err = response.text().await?;
        Err(ServerFnError::new(err))
    }
}

#[component]
pub fn PreviewGoogleRedirectHandler() -> impl IntoView {
    let query_res = use_query::<OAuthQuery>();
    let identity_resource = LocalResource::new(move || async move {
        if let Err(e) = query_res.get() {
            return Err(format!("Invalid Params {e}"));
        }

        let oauth_query = query_res.get().unwrap();

        let delegated_identity_wire = preview_handle_oauth_query(oauth_query)
            .await
            .map_err(|e| e.to_string())?;
        preview_server_set_refersh_token_cookie(delegated_identity_wire.clone())
            .await
            .map_err(|e| e.to_string())?;
        Ok(delegated_identity_wire)
    });

    view! {
        <Loading text="Logging out...".to_string()>
            <Suspense>
                {move || {
                    identity_resource
                        .get()
                        .map(|identity_res| {
                            view! { <IdentitySender identity_res=identity_res.take() /> }
                        })
                }}

            </Suspense>
        </Loading>
    }
}
