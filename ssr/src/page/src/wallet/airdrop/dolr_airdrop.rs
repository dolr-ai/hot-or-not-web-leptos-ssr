use candid::Principal;
use leptos::prelude::*;

#[cfg(not(feature = "neon-postgres"))]
mod mock;
#[cfg(feature = "neon-postgres")]
mod real;

#[server(endpoint = "dolr_airdrop_eligibility", input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<super::AirdropStatus, ServerFnError> {
    #[cfg(not(feature = "neon-postgres"))]
    use mock::is_user_eligible_for_dolr_airdrop as call;
    #[cfg(feature = "neon-postgres")]
    use real::is_user_eligible_for_dolr_airdrop as call;

    call(user_canister, user_principal).await
}

#[server(endpoint = "claim_dolr_airdrop", input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<u64, ServerFnError> {
    #[cfg(not(feature = "neon-postgres"))]
    use mock::claim_dolr_airdrop as call;
    #[cfg(feature = "neon-postgres")]
    use real::claim_dolr_airdrop as call;

    call(user_canister, user_principal).await
}
