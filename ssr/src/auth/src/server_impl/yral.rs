use std::{ops::Deref, sync::LazyLock};

use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar, SignedCookieJar,
};
use candid::Principal;
use consts::LoginProvider;
use global_constants::USERNAME_MAX_LEN;
use leptos::prelude::*;
use leptos_axum::{extract_with_state, ResponseOptions};
use openidconnect::{
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreErrorResponseType,
        CoreGenderClaim, CoreIdTokenVerifier, CoreJsonWebKey, CoreJsonWebKeyType,
        CoreJsonWebKeyUse, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
        CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenIntrospectionResponse,
        CoreTokenType,
    },
    reqwest::async_http_client,
    AdditionalClaims, AuthorizationCode, CsrfToken, EmptyExtraTokenFields, IdTokenFields,
    LoginHint, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, Scope,
    StandardErrorResponse, StandardTokenResponse,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use web_time::Duration;
use yral_types::delegated_identity::DelegatedIdentityWire;

use super::{set_cookies, update_user_identity};

const PKCE_VERIFIER_COOKIE: &str = "google-pkce-verifier";
const CSRF_TOKEN_COOKIE: &str = "google-csrf-token";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YralAuthAdditionalTokenClaims {
    pub ext_is_anonymous: bool,
    pub ext_delegated_identity: DelegatedIdentityWire,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct YralAuthRefreshTokenClaims {
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub sub: Principal,
    pub ext_is_anonymous: bool,
}

impl AdditionalClaims for YralAuthAdditionalTokenClaims {}

pub type YralOAuthClient = openidconnect::Client<
    YralAuthAdditionalTokenClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            YralAuthAdditionalTokenClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
            CoreJsonWebKeyType,
        >,
        CoreTokenType,
    >,
    CoreTokenType,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
>;

pub fn token_verifier() -> CoreIdTokenVerifier<'static> {
    // TODO: use real impl
    CoreIdTokenVerifier::new_insecure_without_verification()
}

#[derive(Serialize, Deserialize)]
struct OAuthState {
    pub csrf_token: CsrfToken,
    pub client_redirect_uri: Option<String>,
}

pub async fn yral_auth_url_impl(
    oauth2: YralOAuthClient,
    login_hint: String,
    provider: LoginProvider,
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
        .set_pkce_challenge(pkce_challenge)
        .set_login_hint(LoginHint::new(login_hint));

    let mut oauth2_request = oauth2_request;
    if provider != LoginProvider::Any {
        let provider = match provider {
            LoginProvider::Google => "google",
            LoginProvider::Apple => "apple",
            LoginProvider::Any => unreachable!(),
        };
        oauth2_request = oauth2_request.add_extra_param("provider", provider);
    }

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

pub fn no_op_nonce_verifier(_: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}

pub async fn perform_yral_auth_impl(
    provided_csrf: String,
    auth_code: String,
    oauth2: YralOAuthClient,
) -> Result<(DelegatedIdentityWire, Option<String>), ServerFnError> {
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

    let id_token_verifier = token_verifier();
    let id_token = token_res
        .extra_fields()
        .id_token()
        .ok_or_else(|| ServerFnError::new("Google did not return an ID token"))?;
    // we don't use a nonce
    let claims = id_token.claims(&id_token_verifier, no_op_nonce_verifier)?;
    let identity: DelegatedIdentityWire = claims.additional_claims().ext_delegated_identity.clone();

    static USERNAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^([a-zA-Z0-9]){3,15}$").unwrap());

    let username = claims.email().and_then(|e| {
        let mail: String = e.deref().clone();
        let mut username = mail.split_once("@")?.0;
        username = username
            .char_indices()
            .nth(USERNAME_MAX_LEN)
            .map(|(i, _)| &username[..i])
            .unwrap_or(username);

        USERNAME_REGEX
            .is_match(username)
            .then(|| username.to_string())
    });

    let jar: SignedCookieJar = extract_with_state(&key).await?;

    let refresh_token = token_res
        .refresh_token()
        .expect("Yral Auth V2 must return a refresh token");

    update_user_identity(&resp, jar, refresh_token.secret().clone())?;

    Ok((identity, username))
}
