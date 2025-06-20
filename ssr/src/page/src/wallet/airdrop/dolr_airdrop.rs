use candid::Nat;
use candid::Principal;
use leptos::prelude::*;

#[cfg(not(feature = "stdb-backend"))]
mod mock;
#[cfg(feature = "stdb-backend")]
mod real;

#[cfg(not(feature = "backend-admin"))]
pub async fn send_airdrop_to_user(
    _user_principal: Principal,
    _amount: Nat,
) -> Result<(), ServerFnError> {
    log::error!("trying to send dolr but no backend admin is available");

    Err(ServerFnError::new("backend admin not available"))
}

#[cfg(feature = "backend-admin")]
pub async fn send_airdrop_to_user(
    user_principal: Principal,
    amount: Nat,
) -> Result<yral_canisters_client::sns_ledger::TransferResult, ServerFnError> {
    use consts::DOLR_AI_LEDGER_CANISTER;
    use state::admin_canisters::AdminCanisters;
    use yral_canisters_client::sns_ledger::{Account, SnsLedger};
    let admin: AdminCanisters = expect_context();

    let ledger = SnsLedger(
        DOLR_AI_LEDGER_CANISTER.parse().unwrap(),
        admin.get_agent().await,
    );

    let res = ledger
        .icrc_1_transfer(yral_canisters_client::sns_ledger::TransferArg {
            to: Account {
                owner: user_principal,
                subaccount: None,
            },
            fee: None,
            memo: None,
            from_subaccount: None,
            created_at_time: None,
            amount,
        })
        .await?;

    Ok(res)
}

#[server(endpoint = "dolr_airdrop_eligibility", input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<super::AirdropStatus, ServerFnError> {
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
