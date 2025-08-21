use candid::Principal;
use leptos::prelude::*;
use state::canisters::unauth_canisters;
use videogen_common::TokenType;
use yral_canisters_common::utils::token::balance::TokenBalance;
use yral_canisters_common::utils::token::load_sats_balance;

/// Load balance for any token type
pub async fn load_token_balance(
    user_principal: Principal,
    token_type: TokenType,
) -> Result<TokenBalance, ServerFnError> {
    match token_type {
        TokenType::Sats => {
            let balance_info = load_sats_balance(user_principal).await?;
            Ok(TokenBalance::new(balance_info.balance.into(), 0))
        }
        TokenType::Dolr => {
            let canisters = unauth_canisters();
            let balance = canisters
                .icrc1_balance_of(
                    user_principal,
                    consts::DOLR_AI_LEDGER_CANISTER.parse().unwrap(),
                )
                .await?;
            Ok(TokenBalance::new(balance, 8))
        }
        videogen_common::TokenType::Free => {
            // Free requests don't have a balance - return 0 as a placeholder
            // The UI will handle displaying appropriate text for free generation
            Ok(TokenBalance::new(0u64.into(), 0))
        }
    }
}
