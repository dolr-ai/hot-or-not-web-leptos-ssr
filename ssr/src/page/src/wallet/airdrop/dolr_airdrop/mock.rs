use candid::Principal;
use leptos::prelude::*;

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    _user_canister: Principal,
    _user_principal: Principal,
) -> Result<bool, ServerFnError> {
    Ok(false)
}

#[server(input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<u64, ServerFnError> {
    Ok(0)
}
