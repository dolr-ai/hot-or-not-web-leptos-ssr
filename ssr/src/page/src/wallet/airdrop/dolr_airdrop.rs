use candid::Principal;
use leptos::prelude::*;

#[server(endpoint = "dolr_airdrop_eligibility", input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    _user_canister: Principal,
    _user_principal: Principal,
) -> Result<super::AirdropStatus, ServerFnError> {
    Ok(super::AirdropStatus::Claimed)
}

#[server(endpoint = "claim_dolr_airdrop", input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    _user_canister: Principal,
    _user_principal: Principal,
) -> Result<u64, ServerFnError> {
    Err(ServerFnError::new("Temporarily disabled"))
}
