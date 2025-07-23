use candid::Principal;
use hon_worker_common::ClaimRequest;
use leptos::prelude::*;
use yral_identity::Signature;

#[server(input = server_fn::codec::Json)]
pub async fn claim_sats_airdrop(
    _user_canister: Principal,
    _request: ClaimRequest,
    _signature: Signature,
) -> Result<u64, ServerFnError> {
    Ok(100)
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_sats_airdrop(
    _user_canister: Principal,
    _user_principal: Principal,
) -> Result<bool, ServerFnError> {
    Ok(true)
}
