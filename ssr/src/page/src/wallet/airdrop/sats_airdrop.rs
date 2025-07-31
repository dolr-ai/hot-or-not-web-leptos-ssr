use candid::Principal;
use hon_worker_common::ClaimRequest;
use leptos::prelude::*;
use yral_identity::Signature;

use super::AirdropStatus;

#[cfg(not(feature = "sats-airdrop"))]
mod mock;
#[cfg(feature = "sats-airdrop")]
mod real;

#[server(input = server_fn::codec::Json)]
pub async fn claim_sats_airdrop(
    user_canister: Principal,
    request: ClaimRequest,
    signature: Signature,
) -> Result<u64, ServerFnError> {
    #[cfg(not(feature = "sats-airdrop"))]
    use mock::claim_sats_airdrop as call;
    #[cfg(feature = "sats-airdrop")]
    use real::claim_sats_airdrop as call;

    call(user_canister, request, signature).await
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_sats_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<bool, ServerFnError> {
    #[cfg(not(feature = "sats-airdrop"))]
    use mock::is_user_eligible_for_sats_airdrop as call;
    #[cfg(feature = "sats-airdrop")]
    use real::is_user_eligible_for_sats_airdrop as call;

    call(user_canister, user_principal).await
}

#[server(input = server_fn::codec::Json)]
pub async fn get_sats_airdrop_status(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<AirdropStatus, ServerFnError> {
    #[cfg(not(feature = "sats-airdrop"))]
    use mock::get_sats_airdrop_status as call;
    #[cfg(feature = "sats-airdrop")]
    use real::get_sats_airdrop_status as call;

    call(user_canister, user_principal).await
}
