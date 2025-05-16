use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar, SignedCookieJar,
};
use candid::Principal;
use ic_agent::{identity::Secp256k1Identity, Identity};
use leptos::prelude::*;
use leptos_axum::{extract_with_state, ResponseOptions};
use openidconnect::{
    core::CoreAuthenticationFlow, reqwest::async_http_client, AuthorizationCode, CsrfToken, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, Scope,
};
use serde::{Deserialize, Serialize};
use web_time::Duration;
use yral_types::delegated_identity::DelegatedIdentityWire;

// use crate::auth::{
//     server_impl::{
//         fetch_identity_from_kv, store::KVStore, try_extract_identity,
//         update_user_identity_and_delegate,
//     },
//     DelegatedIdentityWire,
// };

use super::{
    fetch_identity_from_kv, set_cookies,
    store::{KVStore, KVStoreImpl},
    try_extract_identity, update_user_identity_and_delegate,
};

const PKCE_VERIFIER_COOKIE: &str = "google-pkce-verifier";
const CSRF_TOKEN_COOKIE: &str = "google-csrf-token";

#[derive(Serialize, Deserialize)]
struct OAuthState {
    pub csrf_token: CsrfToken,
    pub client_redirect_uri: Option<String>,
}

pub async fn google_auth_url_impl(
    oauth2: openidconnect::core::CoreClient,
    client_redirect_uri: Option<String>,
) -> Result<String, ServerFnError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let oauth_state = OAuthState {
        csrf_token: CsrfToken::new_random(),
        client_redirect_uri,
    };

    let oauth2_request = oauth2
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            move || CsrfToken::new(serde_json::to_string(&oauth_state).unwrap()),
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".into()))
        .set_pkce_challenge(pkce_challenge);

    let (auth_url, oauth_csrf_token, _) = oauth2_request.url();

    let key: Key = expect_context();
    let mut jar: PrivateCookieJar = extract_with_state(&key).await?;

    let cookie_life = Duration::from_secs(60 * 10).try_into().unwrap(); // 10 minutes
    let pkce_cookie = Cookie::build((PKCE_VERIFIER_COOKIE, pkce_verifier.secret().clone()))
        .same_site(SameSite::None)
        .path("/")
        .max_age(cookie_life)
        .build();
    jar = jar.add(pkce_cookie);

    let csrf_cookie = Cookie::build((CSRF_TOKEN_COOKIE, oauth_csrf_token.secret().clone()))
        .same_site(SameSite::None)
        .path("/")
        .max_age(cookie_life)
        .build();
    jar = jar.add(csrf_cookie);

    let resp: ResponseOptions = expect_context();
    set_cookies(&resp, jar);

    Ok(auth_url.to_string())
}

fn no_op_nonce_verifier(_: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}

fn principal_lookup_key(sub_id: &str) -> String {
    format!("google-login-{sub_id}")
}

async fn try_extract_identity_from_google_sub(
    kv: &KVStoreImpl,
    sub_id: &str,
) -> Result<Option<Secp256k1Identity>, ServerFnError> {
    let Some(principal_text) = kv.read(principal_lookup_key(sub_id)).await? else {
        return Ok(None);
    };
    let principal = Principal::from_text(principal_text)?;
    let Some(identity_secret) = fetch_identity_from_kv(kv, principal).await? else {
        return Ok(None);
    };

    Ok(Some(Secp256k1Identity::from_private_key(identity_secret)))
}

async fn extract_identity_and_associate_with_google_sub(
    kv: &KVStoreImpl,
    jar: &SignedCookieJar,
    sub_id: &str,
) -> Result<Secp256k1Identity, ServerFnError> {
    let identity_secret = try_extract_identity(jar, kv)
        .await?
        .ok_or_else(|| ServerFnError::new("Attempting google login without an identity"))?;
    let identity = Secp256k1Identity::from_private_key(identity_secret);
    let principal = identity.sender().unwrap();
    kv.write(principal_lookup_key(sub_id), principal.to_text())
        .await?;

    Ok(identity)
}

pub async fn perform_google_auth_impl(
    provided_csrf: String,
    auth_code: String,
    oauth2: openidconnect::core::CoreClient,
) -> Result<DelegatedIdentityWire, ServerFnError> {
    let key: Key = expect_context();
    let mut jar: PrivateCookieJar = extract_with_state(&key).await?;

    let csrf_cookie = jar
        .get(CSRF_TOKEN_COOKIE)
        .ok_or_else(|| ServerFnError::new("CSRF token cookie not found"))?;
    if provided_csrf != csrf_cookie.value() {
        return Err(ServerFnError::new("CSRF token mismatch"));
    }

    let pkce_cookie = jar
        .get(PKCE_VERIFIER_COOKIE)
        .ok_or_else(|| ServerFnError::new("PKCE verifier cookie not found"))?;
    let pkce_verifier = PkceCodeVerifier::new(pkce_cookie.value().to_owned());

    jar = jar.remove(PKCE_VERIFIER_COOKIE);
    jar = jar.remove(CSRF_TOKEN_COOKIE);
    let resp: ResponseOptions = expect_context();
    set_cookies(&resp, jar);

    let token_res = oauth2
        .exchange_code(AuthorizationCode::new(auth_code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await?;

    let id_token_verifier = oauth2.id_token_verifier();
    let id_token = token_res
        .extra_fields()
        .id_token()
        .ok_or_else(|| ServerFnError::new("Google did not return an ID token"))?;
    // we don't use a nonce
    let claims = id_token.claims(&id_token_verifier, no_op_nonce_verifier)?;
    let sub_id = claims.subject();

    let kv: KVStoreImpl = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    let identity = if let Some(identity) = try_extract_identity_from_google_sub(&kv, sub_id).await?
    {
        identity
    } else {
        extract_identity_and_associate_with_google_sub(&kv, &jar, sub_id).await?
    };

    let delegated = update_user_identity_and_delegate(&resp, jar, identity)?;

    Ok(delegated)
}
