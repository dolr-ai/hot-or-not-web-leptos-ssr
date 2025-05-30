use candid::Principal;
use futures::{stream::FuturesOrdered, TryStreamExt};
use leptos::prelude::*;
use state::canisters::auth_state;
use yral_canisters_client::individual_user_template::DeployedCdaoCanisters;
use yral_canisters_client::individual_user_template::IndividualUserTemplate;

use crate::wallet::tokens::WalletCard;
use component::{bullet_loader::BulletLoader, token_confetti_symbol::TokenConfettiSymbol};
use utils::send_wrap;
use utils::token::icpump::IcpumpTokenInfo;
use yral_canisters_common::{utils::token::TokenMetadata, Canisters, Error as CanistersError};

#[component]
fn CreateYourToken(header_text: &'static str) -> impl IntoView {
    view! {
        <div class="w-full flex flex-col items-center gap-4">
            <span class="text-2xl text-primary-600 text-center">
                {header_text} <br /> <span class="text-white">Meme Coin</span>
            </span>
            <TokenConfettiSymbol class="w-2/3 md:w-1/2 lg:w-1/3 mx-8" />
        </div>
    }
}

async fn token_metadata(
    cans: &Canisters<true>,
    user_principal: Principal,
    deployed_cans: DeployedCdaoCanisters,
) -> Result<TokenMetadata, CanistersError> {
    let governance = deployed_cans.governance;
    let ledger = deployed_cans.ledger;
    let index = deployed_cans.index;
    cans.get_token_metadata(
        &IcpumpTokenInfo,
        Some(user_principal),
        deployed_cans.root,
        governance,
        ledger,
        index,
    )
    .await
}

async fn process_profile_tokens(
    user: IndividualUserTemplate<'_>,
    cans: Canisters<true>,
    user_principal: Principal,
) -> Result<Vec<(TokenMetadata, Option<bool>)>, ServerFnError> {
    let tokens: Vec<_> = user
        .deployed_cdao_canisters()
        .await?
        .into_iter()
        .map(|deployed_cans| {
            let cans = cans.clone();
            async move {
                let token = token_metadata(&cans, user_principal, deployed_cans).await?;
                let is_airdrop_claimed = if let (Some(token_owner), Some(root)) =
                    (token.token_owner.clone(), token.root)
                {
                    Some(
                        cans.get_airdrop_status(token_owner.canister_id, root, user_principal)
                            .await?,
                    )
                } else {
                    None
                };

                Ok::<_, ServerFnError>((token, is_airdrop_claimed))
            }
        })
        .collect::<FuturesOrdered<_>>()
        .try_collect()
        .await?;

    Ok(tokens)
}

#[component]
pub fn ProfileTokens(user_canister: Principal, user_principal: Principal) -> impl IntoView {
    let auth = auth_state();
    let token_list_res = auth.derive_resource(
        || (),
        move |cans, _| {
            send_wrap(async move {
                let user = cans.individual_user(user_canister).await;

                let tokens = process_profile_tokens(user, cans.clone(), user_principal).await?;
                Ok::<_, ServerFnError>((tokens, cans.user_principal() == user_principal))
            })
        },
    );

    view! {
        <div class="flex flex-col w-full items-center gap-4">
            <Suspense fallback=|| {
                view! {
                    <div class="w-full flex justify-center items-center py-9">
                        <BulletLoader />
                    </div>
                }
            }>
                {move || {
                    token_list_res.get()
                        .map(|res| res.unwrap_or((vec![], false)))
                        .map(|(tokens, is_native_profile)| {
                            let empty = tokens.is_empty();
                            view! {
                                {tokens
                                    .into_iter()
                                    .map(|(token, is_airdrop_claimed)| {
                                        view! {
                                            <WalletCard
                                                user_principal
                                                token_metadata=token
                                                is_airdrop_claimed=is_airdrop_claimed.unwrap_or(true)
                                            />
                                        }
                                    })
                                    .collect_view()}
                                {empty
                                    .then(|| {
                                        view! {
                                            <CreateYourToken header_text=if is_native_profile {
                                                "Create your own"
                                            } else {
                                                "They have not created any"
                                            } />
                                        }
                                    })}
                                {is_native_profile
                                    .then(|| {
                                        view! {
                                            <a
                                                href="/token/create"
                                                class="text-xl bg-primary-600 py-4 w-2/3 md:w-1/2 lg:w-1/3 rounded-full text-center text-white"
                                            >
                                                Create
                                            </a>
                                        }
                                    })}
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
