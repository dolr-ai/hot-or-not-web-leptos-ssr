use hon_worker_common::{sign_claim_request, ClaimRequest};
use leptos::prelude::*;
use page::wallet::airdrop::{claim_sats_airdrop, is_airdrop_claimed};
use state::canisters::auth_state;
use yral_canisters_common::Canisters;

#[component]
pub fn TestAirdrop() -> impl IntoView {
    let auth = auth_state();
    let fetch_claim_status: Action<(), Result<bool, ServerFnError>> =
        Action::new_local(move |&()| async move {
            let cans_wire = auth.cans_wire().await.unwrap();
            let cans = Canisters::from_wire(cans_wire.clone(), expect_context()).unwrap();
            let is_airdrop_claimed = is_airdrop_claimed(cans.user_principal()).await?;

            Ok::<bool, ServerFnError>(is_airdrop_claimed)
        });

    let auth = auth_state();
    let send_claim_request = Action::new_local(move |&()| async move {
        let cans_wire = auth.cans_wire().await.unwrap();
        let cans = Canisters::from_wire(cans_wire.clone(), expect_context()).unwrap();

        let request = ClaimRequest {
            user_principal: cans.user_principal(),
        };
        let signature = sign_claim_request(cans.identity(), request.clone()).unwrap();

        let res = claim_sats_airdrop(cans.user_canister(), request, signature).await?;

        Ok::<_, ServerFnError>(res)
    });

    let claim_status = fetch_claim_status.value();
    let claim_status = move || {
        let v = claim_status.get();

        match v {
            Some(res) => match res {
                Ok(v) => format!("{v}"),
                Err(e) => format!("err = {e:?}"),
            },
            None => "no value".to_string(),
        }
    };

    let claim_response = send_claim_request.value();
    let claim_response = move || {
        let v = claim_response.get();

        match v {
            Some(res) => match res {
                Ok(res) => format!("amount = {res}"),
                Err(err) => format!("server fn error: {err:?}"),
            },
            None => "no value".to_string(),
        }
    };

    view! {
        <div class="text-white">
            <button class="border border-white" on:click=move |_| {fetch_claim_status.dispatch(());}>
                Check Claimed Status
            </button>
            <div>
                is airdrop claimed: <pre>{claim_status}</pre>
            </div>
            <button class="border border-white" on:click=move |_| {send_claim_request.dispatch(());}>
                send request
            </button>
            <div>
                response: <pre>{claim_response}</pre>
            </div>
        </div>
    }
}
