pub mod store;
#[cfg(feature = "oauth-ssr")]
pub mod yral;

use axum::response::IntoResponse;
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    CookieJar, SignedCookieJar,
};
use candid::Principal;
use http::header;
use ic_agent::{identity::Secp256k1Identity, Identity};
use k256::elliptic_curve::JwkEcKey;
use leptos::prelude::*;
use leptos_axum::{extract, extract_with_state, ResponseOptions};
use rand_chacha::rand_core::OsRng;
use yral_canisters_common::utils::time::current_epoch;

use consts::{
    auth::{REFRESH_MAX_AGE, REFRESH_TOKEN_COOKIE},
    ACCOUNT_CONNECTED_STORE, USER_CANISTER_ID_STORE,
};

use crate::{delegate_identity, AnonymousIdentity};

use self::store::{KVStore, KVStoreImpl};
use yral_types::delegated_identity::DelegatedIdentityWire;

use super::RefreshTokenLegacy;

fn set_cookies(resp: &ResponseOptions, jar: impl IntoResponse) {
    let resp_jar = jar.into_response();
    for cookie in resp_jar
        .headers()
        .get_all(header::SET_COOKIE)
        .into_iter()
        .cloned()
    {
        resp.append_header(header::SET_COOKIE, cookie);
    }
}

pub fn extract_principal_from_cookie_legacy(
    jar: &SignedCookieJar,
) -> Result<Option<Principal>, ServerFnError> {
    let Some(cookie) = jar.get(REFRESH_TOKEN_COOKIE) else {
        return Ok(None);
    };
    let token: RefreshTokenLegacy = serde_json::from_str(cookie.value())?;
    if current_epoch().as_millis() > token.expiry_epoch_ms {
        return Ok(None);
    }
    Ok(Some(token.principal))
}

async fn fetch_identity_from_kv(
    kv: &KVStoreImpl,
    principal: Principal,
) -> Result<Option<k256::SecretKey>, ServerFnError> {
    let Some(identity_jwk) = kv.read(principal.to_text()).await? else {
        return Ok(None);
    };

    Ok(Some(k256::SecretKey::from_jwk_str(&identity_jwk)?))
}

pub async fn try_extract_identity_legacy(
    jar: &SignedCookieJar,
    kv: &KVStoreImpl,
) -> Result<Option<k256::SecretKey>, ServerFnError> {
    let Some(principal) = extract_principal_from_cookie_legacy(jar)? else {
        return Ok(None);
    };
    fetch_identity_from_kv(kv, principal).await
}

async fn generate_and_save_identity_legacy(
    kv: &KVStoreImpl,
) -> Result<Secp256k1Identity, ServerFnError> {
    let base_identity_key = k256::SecretKey::random(&mut OsRng);
    let base_identity = Secp256k1Identity::from_private_key(base_identity_key.clone());
    let principal = base_identity.sender().unwrap();

    let base_jwk = base_identity_key.to_jwk_string();
    kv.write(principal.to_text(), base_jwk.to_string()).await?;
    Ok(base_identity)
}

fn identity_from_jwk(id: &JwkEcKey) -> Result<Secp256k1Identity, ServerFnError> {
    let base_identity_key = k256::SecretKey::from_jwk(id)?;
    let base_identity: Secp256k1Identity =
        Secp256k1Identity::from_private_key(base_identity_key.clone());
    Ok(base_identity)
}

pub fn update_user_identity(
    response_opts: &ResponseOptions,
    mut jar: SignedCookieJar,
    refresh_jwt: String,
) -> Result<(), ServerFnError> {
    let refresh_max_age = REFRESH_MAX_AGE;

    let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, refresh_jwt))
        .http_only(true)
        .secure(true)
        .path("/")
        .same_site(SameSite::None)
        .partitioned(true)
        .max_age(refresh_max_age.try_into().unwrap());

    jar = jar.add(refresh_cookie);
    set_cookies(response_opts, jar);
    Ok(())
}

async fn extract_identity_legacy(
    jar: &SignedCookieJar,
    refresh_token: &Cookie<'static>,
) -> Result<Option<DelegatedIdentityWire>, ServerFnError> {
    if serde_json::from_str::<RefreshTokenLegacy>(refresh_token.value()).is_err() {
        return Ok(None);
    }

    let kv: KVStoreImpl = expect_context();
    let Some(id) = try_extract_identity_legacy(jar, &kv).await? else {
        return Ok(None);
    };
    let base_identity = Secp256k1Identity::from_private_key(id);

    let id = delegate_identity(&base_identity);

    Ok(Some(id))
}

pub async fn extract_identity_impl() -> Result<Option<DelegatedIdentityWire>, ServerFnError> {
    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;

    #[cfg(not(feature = "oauth-ssr"))]
    {
        let kv: KVStoreImpl = expect_context();
        let base_identity = if let Some(identity) = try_extract_identity_legacy(&jar, &kv).await? {
            Secp256k1Identity::from_private_key(identity)
        } else {
            return Ok(None);
        };

        Ok(Some(delegate_identity(&base_identity)))
    }

    #[cfg(feature = "oauth-ssr")]
    {
        use openidconnect::{reqwest::async_http_client, RefreshToken};
        use yral::YralOAuthClient;

        let Some(refresh_token) = jar.get(REFRESH_TOKEN_COOKIE) else {
            return Ok(None);
        };

        if let Some(id) = extract_identity_legacy(&jar, &refresh_token).await? {
            return Ok(Some(id));
        }

        let oauth2: YralOAuthClient = expect_context();
        let token_res = oauth2
            .exchange_refresh_token(&RefreshToken::new(refresh_token.value().to_string()))
            .request_async(async_http_client)
            .await?;

        let id_token = token_res
            .extra_fields()
            .id_token()
            .expect("Yral Auth V2 must return an ID token");
        let id_claims = id_token.claims(&yral::token_verifier(), yral::no_op_nonce_verifier)?;
        let identity = id_claims.additional_claims().ext_delegated_identity.clone();

        Ok(Some(identity))
    }
}

pub async fn logout_identity_impl() -> Result<DelegatedIdentityWire, ServerFnError> {
    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    let resp: ResponseOptions = expect_context();

    #[cfg(not(feature = "oauth-ssr"))]
    {
        let kv: KVStoreImpl = expect_context();
        let identity = generate_and_save_identity_legacy(&kv).await?;

        let refresh_token = serde_json::to_string(&RefreshTokenLegacy {
            principal: identity.sender().unwrap(),
            expiry_epoch_ms: (current_epoch() + REFRESH_MAX_AGE).as_millis(),
        })
        .unwrap();

        update_user_identity(&resp, jar, refresh_token)?;

        let delegated = delegate_identity(&identity);

        Ok(delegated)
    }

    #[cfg(feature = "oauth-ssr")]
    {
        use openidconnect::{reqwest::async_http_client, OAuth2TokenResponse};
        let oauth_client: yral::YralOAuthClient = expect_context();
        let token = oauth_client
            .exchange_client_credentials()
            .request_async(async_http_client)
            .await?;

        let id_token = token
            .extra_fields()
            .id_token()
            .expect("Yral Auth V2 must return an ID token");
        let refresh_token = token
            .refresh_token()
            .expect("Yral Auth V2 must return a refresh token");

        let id_claims = id_token.claims(&yral::token_verifier(), yral::no_op_nonce_verifier)?;
        let identity = id_claims.additional_claims().ext_delegated_identity.clone();
        update_user_identity(&resp, jar, refresh_token.secret().clone())?;

        Ok(identity)
    }
}

pub async fn generate_anonymous_identity_if_required_impl(
) -> Result<Option<AnonymousIdentity>, ServerFnError> {
    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    #[cfg(not(feature = "oauth-ssr"))]
    {
        if extract_principal_from_cookie_legacy(&jar)?.is_some() {
            return Ok(None);
        }

        let kv: KVStoreImpl = expect_context();
        let identity = generate_and_save_identity_legacy(&kv).await?;
        Ok(Some(AnonymousIdentity {
            identity: delegate_identity(&identity).into(),
            refresh_token: serde_json::to_string(&RefreshTokenLegacy {
                principal: identity.sender().unwrap(),
                expiry_epoch_ms: (current_epoch() + REFRESH_MAX_AGE).as_millis(),
            })
            .unwrap(),
        }))
    }

    #[cfg(feature = "oauth-ssr")]
    {
        if jar.get(REFRESH_TOKEN_COOKIE).is_some() {
            return Ok(None);
        }

        use openidconnect::{reqwest::async_http_client, OAuth2TokenResponse};
        let oauth_client: yral::YralOAuthClient = expect_context();
        let token = oauth_client
            .exchange_client_credentials()
            .request_async(async_http_client)
            .await;
        let token = match token {
            Ok(token) => token,
            Err(e) => {
                eprintln!("Request token error {e:?}");
                return Err(ServerFnError::new(format!(
                    "Failed to exchange client credentials: {e}",
                )));
            }
        };

        let id_token = token
            .extra_fields()
            .id_token()
            .expect("Yral Auth V2 must return an ID token");
        let refresh_token = token
            .refresh_token()
            .expect("Yral Auth V2 must return a refresh token");

        let id_claims = id_token.claims(&yral::token_verifier(), yral::no_op_nonce_verifier)?;
        let identity = id_claims.additional_claims().ext_delegated_identity.clone();

        Ok(Some(AnonymousIdentity {
            identity,
            refresh_token: refresh_token.secret().to_string(),
        }))
    }
}

pub async fn set_anonymous_identity_cookie_impl(
    refresh_jwt: Option<String>,
) -> Result<(), ServerFnError> {
    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;

    let resp: ResponseOptions = expect_context();

    if let Some(refresh_jwt) = refresh_jwt {
        update_user_identity(&resp, jar, refresh_jwt)?;
        return Ok(());
    }

    // TODO: remove this after 30 days
    #[cfg(feature = "oauth-ssr")]
    {
        use yral::migrate_identity_to_yral_auth;

        let Ok(Some(user_principal)) = extract_principal_from_cookie_legacy(&jar) else {
            return Ok(());
        };
        let unsigned_jar: CookieJar = extract().await?;

        let is_connected = unsigned_jar
            .get(ACCOUNT_CONNECTED_STORE)
            .map(|cookie| cookie.value() == "true")
            .unwrap_or_default();
        let user_canister = unsigned_jar
            .get(USER_CANISTER_ID_STORE)
            .and_then(|cookie| cookie.value().parse().ok());
        let new_cookie =
            migrate_identity_to_yral_auth(user_principal, user_canister, !is_connected).await?;

        update_user_identity(&resp, jar, new_cookie)?;
    }

    Ok(())
}
