use candid::Principal;
use leptos::prelude::*;
use yral_canisters_client::individual_user_template::{Result7, SessionType};
use yral_canisters_common::Canisters;

pub async fn validate_dolr_airdrop_eligibility(
    user_canister: Principal,
    _user_principal: Principal,
) -> Result<(), ServerFnError> {
    let cans = Canisters::default();
    let user = cans.individual_user(user_canister).await;

    let sess = user.get_session_type().await?;
    if !matches!(sess, Result7::Ok(SessionType::RegisteredSession)) {
        return Err(ServerFnError::new("Not allowed to claim: not logged in"));
    }

    // fetch airdrop data from stdb
    // validate constraints

    Ok(())
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<bool, ServerFnError> {
    let res = validate_dolr_airdrop_eligibility(user_canister, user_principal).await;

    match res {
        Ok(_) => Ok(true),
        Err(ServerFnError::ServerError(..)) => Ok(false),
        Err(err) => Err(err),
    }
}

#[server(input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<u64, ServerFnError> {
    let cans: Canisters<false> = expect_context();
    let user = cans.individual_user(user_canister).await;
    let profile_owner = user.get_profile_details_v_2().await?;
    if profile_owner.principal_id != user_principal {
        // ideally should never happen unless its a hacking attempt
        println!(
            "Not allowed to claim due to principal mismatch: owner={} != receiver={user_principal}",
            profile_owner.principal_id,
        );
        return Err(ServerFnError::new(
            "Not allowed to claim: principal mismatch",
        ));
    }
    validate_dolr_airdrop_eligibility(user_canister, user_principal).await?;

    // send dolr with backend admin
    // update info in stdb

    Ok(100)
}
