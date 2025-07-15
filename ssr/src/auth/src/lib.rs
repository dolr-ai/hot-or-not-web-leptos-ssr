// TEMP
#![allow(dead_code)]

#[cfg(feature = "ssr")]
pub mod server_impl;

use candid::Principal;
use ic_agent::{
    identity::{Delegation, Secp256k1Identity, SignedDelegation},
    Identity,
};
use leptos::prelude::*;
use leptos::{server, server_fn::codec::Json};
use rand_chacha::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use web_time::Duration;
use yral_canisters_common::utils::time::current_epoch;

use consts::auth::DELEGATION_MAX_AGE;
use yral_types::delegated_identity::DelegatedIdentityWire;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnonymousIdentity {
    pub identity: DelegatedIdentityWire,
    pub refresh_token: String,
}

fn delegate_identity_with_max_age(
    from: &impl Identity,
    max_age: Duration,
) -> DelegatedIdentityWire {
    let to_secret = k256::SecretKey::random(&mut OsRng);
    let to_identity = Secp256k1Identity::from_private_key(to_secret.clone());
    let expiry = current_epoch() + max_age;
    let expiry_ns = expiry.as_nanos() as u64;
    let delegation = Delegation {
        pubkey: to_identity.public_key().unwrap(),
        expiration: expiry_ns,
        targets: None,
    };
    let sig = from.sign_delegation(&delegation).unwrap();
    let signed_delegation = SignedDelegation {
        delegation,
        signature: sig.signature.unwrap(),
    };

    let mut delegation_chain = from.delegation_chain();
    delegation_chain.push(signed_delegation);

    DelegatedIdentityWire {
        from_key: sig.public_key.unwrap(),
        to_secret: to_secret.to_jwk(),
        delegation_chain,
    }
}

pub fn delegate_identity(from: &impl Identity) -> DelegatedIdentityWire {
    delegate_identity_with_max_age(from, DELEGATION_MAX_AGE)
}

pub fn delegate_short_lived_identity(from: &impl Identity) -> DelegatedIdentityWire {
    let max_age = Duration::from_secs(24 * 60 * 60); // 1 day
    delegate_identity_with_max_age(from, max_age)
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct RefreshTokenLegacy {
    principal: Principal,
    expiry_epoch_ms: u128,
}

/// Generate an anonymous identity if refresh token is not set
#[server]
pub async fn generate_anonymous_identity_if_required(
) -> Result<Option<AnonymousIdentity>, ServerFnError> {
    server_impl::generate_anonymous_identity_if_required_impl().await
}

/// this server function is purely a side effect and only sets the refresh token cookie
#[server(endpoint = "set_anonymous_identity_cookie", input = Json, output = Json)]
pub async fn set_anonymous_identity_cookie(
    refresh_jwt: Option<String>,
) -> Result<(), ServerFnError> {
    server_impl::set_anonymous_identity_cookie_impl(refresh_jwt).await
}

/// Extract the identity from refresh token,
/// returns None if refresh token doesn't exist
#[server(endpoint = "extract_identity", input = Json, output = Json)]
pub async fn extract_identity() -> Result<Option<DelegatedIdentityWire>, ServerFnError> {
    server_impl::extract_identity_impl().await
}

#[server]
pub async fn logout_identity() -> Result<DelegatedIdentityWire, ServerFnError> {
    server_impl::logout_identity_impl().await
}
