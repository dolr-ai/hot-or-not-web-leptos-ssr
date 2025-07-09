use candid::Principal;
use leptos::prelude::*;

#[server(input = server_fn::codec::Json)]
pub async fn claim_sats_airdrop(
    user_canister: Principal,
    request: ClaimRequest,
    signature: Signature,
) -> Result<u64, ServerFnError> {
    Ok(100)
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_sats_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<bool, ServerFnError> {
    Ok(true)
}
