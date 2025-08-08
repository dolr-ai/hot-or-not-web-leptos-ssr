use candid::Principal;
use consts::WHITELIST_FOR_SATS_CLEARING;
use hon_worker_common::{SatsBalanceUpdateRequestV2, WORKER_URL};
use leptos::prelude::*;
use num_bigint::Sign;
use reqwest::Url;
use state::{canisters::auth_state, server::HonWorkerJwt};
use utils::{send_wrap, try_or_redirect_opt};
use yral_canisters_client::individual_user_template::{Result7, SessionType};
use yral_canisters_common::{utils::token::load_sats_balance, Canisters};

use crate::wallet::tokens::BalanceFetcherType;

#[server(input = server_fn::codec::Json)]
pub async fn clear_sats(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<(), ServerFnError> {
    let cans: Canisters<false> = expect_context();
    let user = cans.individual_user(user_canister).await;
    let profile_owner = user.get_profile_details_v_2().await?;
    let sess = user.get_session_type().await?;
    if profile_owner.principal_id != user_principal {
        leptos::logging::log!(
            "sats clearing({user_principal}): doesn't match expected = {}",
            profile_owner.principal_id
        );
        return Err(ServerFnError::new(""));
    }
    if !WHITELIST_FOR_SATS_CLEARING.contains(user_principal.to_text().as_str()) {
        leptos::logging::log!("sats clearing({user_principal}): not whitelisted");
        return Err(ServerFnError::new(""));
    }
    if !matches!(sess, Result7::Ok(SessionType::RegisteredSession)) {
        leptos::logging::log!("sats clearing({user_principal}): not logged in");
        return Err(ServerFnError::new(""));
    }

    let balance = load_sats_balance(user_principal).await?.balance;

    let Some(jwt): Option<HonWorkerJwt> = use_context() else {
        leptos::logging::log!("sats clearing({user_principal}): no token");
        return Err(ServerFnError::new(""));
    };

    let req_url: Url = WORKER_URL.parse().expect("url to be valid");
    let req_url = req_url
        .join(&format!("/v2/update_balance/{user_principal}"))
        .expect("url to be valid");
    let delta = num_bigint::BigInt::from_biguint(Sign::Minus, balance.clone());
    if delta > 0.into() {
        leptos::logging::log!("sats clearing({user_principal}): balance is negative?");
        return Err(ServerFnError::new(""));
    }
    let worker_req = SatsBalanceUpdateRequestV2 {
        previous_balance: balance,
        delta,
        is_airdropped: false,
    };
    let client = reqwest::Client::new();
    let res = client
        .post(req_url)
        .json(&worker_req)
        .header("Authorization", format!("Bearer {}", jwt.0))
        .send()
        .await?;

    if !res.status().is_success() {
        let (status, text) = (res.status().as_u16(), res.text().await?);
        leptos::logging::log!("sats clearing({user_principal}): worker error({status}): {text}");
        return Err(ServerFnError::new(""));
    }

    Ok(())
}

#[component]
pub fn ClearSats() -> impl IntoView {
    let auth = auth_state();
    let balance = Resource::new_blocking(
        || (),
        move |_| async move {
            let fetcher = BalanceFetcherType::Sats;
            let cans = send_wrap(auth.auth_cans()).await?;
            let user_canister = cans.user_canister();
            let user_principal = cans.user_principal();
            if !WHITELIST_FOR_SATS_CLEARING.contains(user_principal.to_text().as_str()) {
                return Err(ServerFnError::new("who dis?"));
            }
            let logged_in = send_wrap(
                send_wrap(cans.individual_user(user_canister))
                    .await
                    .get_session_type(),
            )
            .await?;
            if !matches!(logged_in, Result7::Ok(SessionType::RegisteredSession)) {
                return Err(ServerFnError::new("invalid session"));
            }
            let balance =
                send_wrap(fetcher.fetch(Default::default(), user_canister, user_principal)).await?;

            let res = (balance, user_canister, user_principal);

            Ok::<_, ServerFnError>(res)
        },
    );

    let action = Action::new_unsync(
        move |&(user_canister, user_principal): &(Principal, Principal)| async move {
            clear_sats(user_canister, user_principal).await
        },
    );

    let value = action.value();

    view! {
        <div class="text-white">
            <Suspense>
            {move || Suspend::new(async move {
                let (balance, user_canister, user_principal) = try_or_redirect_opt!(balance.await.inspect_err(|err| {
                    leptos::logging::log!("balance fetching for sat clears failed: {err:?}");
                }).map_err(|_| "not found"));

                Some(
                    view! {
                        <p>user principal: {user_principal.to_text()}</p>
                        <p>user canister : {user_canister.to_text()}</p>
                        <p>balance: {balance.humanize()}</p>
                        {move || value.get().map(|res| {
                            match res {
                                Ok(_) => view! { <p>balance was cleared</p>}.into_any(),
                                Err(err) => view! { <p>Couldnt clear balance: {format!("{err:#?}")}</p>}.into_any(),
                            }
                        })}
                        <button class="bg-red text-white border-white" on:click=move |_| { action.dispatch((user_canister, user_principal)); }>
                            Clear My Sats Balance
                        </button>
                    }
                )
            })}
            </Suspense>
        </div>
    }
}
