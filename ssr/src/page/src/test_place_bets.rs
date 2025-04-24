use candid::Principal;
use consts::PUMP_AND_DUMP_WORKER_URL;
use http::StatusCode;
use leptos::{component, prelude::*, view, IntoView};
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use state::canisters::authenticated_canisters;
use utils::send_wrap;
use yral_canisters_common::{
    utils::vote::{HonBetArg, VerifiableHonBetReq, VoteKind},
    Canisters,
};

#[derive(Params, PartialEq, Eq, Clone, Debug)]
struct TestParams {
    post_canister_id: Principal,
    post_id: u64,
}

#[component]
pub fn TestPlaceBets() -> impl IntoView {
    let params = use_params::<TestParams>();
    let auth_wire = authenticated_canisters();
    let send_bet = Action::new(move |&()| {
        let auth_wire = auth_wire;
        send_wrap(async move {
            let auth_wire = auth_wire.await.map_err(ServerFnError::new)?;

            let cans = Canisters::from_wire(auth_wire.clone(), expect_context())
                .map_err(ServerFnError::new)?;

            let TestParams {
                post_canister_id,
                post_id,
            } = params.read().clone().unwrap();

            let req = VerifiableHonBetReq::new(
                cans.identity(),
                HonBetArg {
                    bet_amount: 100 * 1000000,
                    bet_direction: VoteKind::Hot,
                    post_id,
                    post_canister_id,
                },
            )
            .map_err(ServerFnError::new)?;
            let claim_url = PUMP_AND_DUMP_WORKER_URL
                .join("/place_hon_bet")
                .expect("Url to be valid");
            let client = reqwest::Client::new();
            let res = client
                .post(claim_url)
                .json(&req)
                .send()
                .await
                .map_err(ServerFnError::new)?;

            if res.status() != StatusCode::OK {
                return Err(ServerFnError::new("Request failed"));
            }

            Ok::<(), ServerFnError>(())
        })
    });

    view! {
        <button class="text-white" on:click=move |_| {send_bet.dispatch(());}>send</button>
    }
}
