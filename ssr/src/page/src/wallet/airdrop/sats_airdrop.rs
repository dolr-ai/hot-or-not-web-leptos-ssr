use super::AirdropStatus;
use candid::Principal;
use hon_worker_common::ClaimRequest;
use leptos::prelude::*;
use yral_identity::Signature;

mod mock;

#[server(input = server_fn::codec::Json)]
pub async fn claim_sats_airdrop(
    user_canister: Principal,
    request: ClaimRequest,
    signature: Signature,
) -> Result<u64, ServerFnError> {
    mock::claim_sats_airdrop(user_canister, request, signature).await
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_sats_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<bool, ServerFnError> {
    mock::is_user_eligible_for_sats_airdrop(user_canister, user_principal).await
}

#[server(input = server_fn::codec::Json)]
pub async fn get_sats_airdrop_status(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<AirdropStatus, ServerFnError> {
    mock::get_sats_airdrop_status(user_canister, user_principal).await
}
