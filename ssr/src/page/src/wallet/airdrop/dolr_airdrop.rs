use candid::Principal;
use leptos::prelude::*;

#[cfg(not(feature = "stdb-backend"))]
mod mock;
#[cfg(feature = "stdb-backend")]
mod real;

#[server(endpoint = "dolr_airdrop_eligibility", input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<bool, ServerFnError> {
    #[cfg(not(feature = "stdb-backend"))]
    use mock::is_user_eligible_for_dolr_airdrop as call;
    #[cfg(feature = "stdb-backend")]
    use real::is_user_eligible_for_dolr_airdrop as call;

    call(user_canister, user_principal).await
}

#[server(endpoint = "claim_dolr_airdrop", input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<u64, ServerFnError> {
    #[cfg(not(feature = "stdb-backend"))]
    use mock::claim_dolr_airdrop as call;
    #[cfg(feature = "stdb-backend")]
    use real::claim_dolr_airdrop as call;

    call(user_canister, user_principal).await
}
