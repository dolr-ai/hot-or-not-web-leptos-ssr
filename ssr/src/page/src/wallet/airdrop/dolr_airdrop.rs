use candid::Principal;
use leptos::prelude::*;

#[cfg(not(feature = "dolr-airdrop"))]
mod mock;
// #[cfg(feature = "dolr-airdrop")]
// mod real;

#[server(endpoint = "dolr_airdrop_eligibility", input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    _user_canister: Principal,
    _user_principal: Principal,
) -> Result<super::AirdropStatus, ServerFnError> {
    // #[cfg(not(feature = "dolr-airdrop"))]
    // use mock::is_user_eligible_for_dolr_airdrop as call;
    // #[cfg(feature = "dolr-airdrop")]
    // use real::is_user_eligible_for_dolr_airdrop as call;

    // call(user_canister, user_principal).await

    Ok(super::AirdropStatus::Claimed)
}

#[server(endpoint = "claim_dolr_airdrop", input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    _user_canister: Principal,
    _user_principal: Principal,
) -> Result<u64, ServerFnError> {
    // #[cfg(not(feature = "dolr-airdrop"))]
    // use mock::claim_dolr_airdrop as call;
    // #[cfg(feature = "dolr-airdrop")]
    // use real::claim_dolr_airdrop as call;

    // call(user_canister, user_principal).await

    Err(ServerFnError::new("Temporarily disabled"))
}
