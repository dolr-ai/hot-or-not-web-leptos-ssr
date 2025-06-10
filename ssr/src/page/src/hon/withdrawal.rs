use candid::{Nat, Principal};
use component::{
    auth_providers::handle_user_login, back_btn::BackButton,
    icons::notification_icon::NotificationIcon, title::TitleText,
};
use consts::{MAX_WITHDRAWAL_PER_TXN, MIN_WITHDRAWAL_PER_TXN};
use futures::TryFutureExt;
use hon_worker_common::{HoNGameWithdrawReq, SatsBalanceInfo};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use log;
use state::{canisters::auth_state, server::HonWorkerJwt};
use utils::{send_wrap, try_or_redirect_opt};
use yral_canisters_client::individual_user_template::{Result7, SessionType};
use yral_canisters_common::{utils::token::balance::TokenBalance, Canisters};
use yral_identity::Signature;

pub mod result;

macro_rules! format_sats {
    ($num:expr) => {
        TokenBalance::new($num, 0).humanize_float_truncate_to_dp(0)
    };
}

/// Details for withdrawal functionality
type Details = SatsBalanceInfo;

async fn load_withdrawal_details(user_principal: Principal) -> Result<Details, String> {
    let url: reqwest::Url = hon_worker_common::WORKER_URL
        .parse()
        .expect("Url to be valid");
    let balance_info = url
        .join(&format!("/balance/{user_principal}"))
        .expect("Url to be valid");

    let balance_info: SatsBalanceInfo = reqwest::get(balance_info)
        .await
        .map_err(|_| "failed to load balance".to_string())?
        .json()
        .await
        .map_err(|_| "failed to read response body".to_string())?;

    Ok(balance_info)
}

#[server(input = server_fn::codec::Json)]
async fn withdraw_sats_for_ckbtc(
    receiver_canister: Principal,
    req: hon_worker_common::WithdrawRequest,
    sig: Signature,
) -> Result<(), ServerFnError> {
    use hon_worker_common::WORKER_URL;

    // TODO: yral-auth-v2, we can do this verification with a JWT
    let cans: Canisters<false> = expect_context();

    if req.amount < MIN_WITHDRAWAL_PER_TXN as u128 || req.amount > MAX_WITHDRAWAL_PER_TXN as u128 {
        log::error!(
            "Invalid withdraw amount, min amount: {}, max amount: {}, amount: {}",
            MIN_WITHDRAWAL_PER_TXN,
            MAX_WITHDRAWAL_PER_TXN,
            req.amount
        );
        return Err(ServerFnError::new(format!(
            "Invalid withdraw amount, min amount: {}, max amount: {}, amount: {}",
            MIN_WITHDRAWAL_PER_TXN, MAX_WITHDRAWAL_PER_TXN, req.amount
        )));
    }

    let user = cans.individual_user(receiver_canister).await;
    let profile_owner = user.get_profile_details_v_2().await?;
    if profile_owner.principal_id != req.receiver {
        log::error!(
            "Not allowed to withdraw due to principal mismatch: owner={} != receiver={}",
            profile_owner.principal_id,
            req.receiver
        );
        return Err(ServerFnError::new("Not allowed to withdraw"));
    }

    let sess = user.get_session_type().await?;
    if !matches!(sess, Result7::Ok(SessionType::RegisteredSession)) {
        log::error!("Not allowed to withdraw due to invalid session: {sess:?}");
        return Err(ServerFnError::new("Not allowed to withdraw"));
    }

    log::info!("creating withdraw request");

    let worker_req = HoNGameWithdrawReq {
        request: req,
        signature: sig,
    };
    let req_url = format!("{WORKER_URL}withdraw");
    let client = reqwest::Client::new();
    let jwt = expect_context::<HonWorkerJwt>();
    let res = client
        .post(&req_url)
        .json(&worker_req)
        .header("Authorization", format!("Bearer {}", jwt.0))
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(ServerFnError::new(format!(
            "worker error[{}]: {}",
            res.status().as_u16(),
            res.text().await?
        )));
    }

    Ok(())
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <div id="back-nav" class="flex flex-col items-center w-full gap-20 pb-16">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/" />
                    <span class="font-bold text-2xl">Withdraw</span>
                    <a href="/wallet/notifications" aria_disabled=true class="text-xl font-semibold">
                        <NotificationIcon show_dot=false class="w-8 h-8 text-neutral-600" />
                    </a>
                </div>
            </TitleText>
        </div>
    }
}

#[component]
fn BalanceDisplay(#[prop(into)] balance: Nat) -> impl IntoView {
    view! {
        <div id="total-balance" class="self-center flex flex-col items-center gap-1">
            <span class="text-neutral-400 text-sm">Total Sats balance</span>
            <div class="flex items-center gap-3 min-h-14 py-0.5">
                <img class="size-9 rounded-full" src="/img/hotornot/sats.svg" alt="sats icon" />
                <span class="font-bold text-4xl">{format_sats!(balance)}</span>
            </div>
        </div>
    }
}

#[component]
pub fn HonWithdrawal() -> impl IntoView {
    let auth = auth_state();
    let details_res = auth.derive_resource(
        || (),
        move |cans, _| {
            send_wrap(async move {
                let principal = cans.user_principal();

                load_withdrawal_details(principal)
                    .map_err(ServerFnError::new)
                    .await
            })
        },
    );

    let sats = RwSignal::new(0usize);
    let formated_dolrs = move || {
        format!(
            "{} BTC",
            TokenBalance::new(sats().into(), 8).humanize_float_truncate_to_dp(8)
        )
    };

    let on_input = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev);
        let value: Option<usize> = value
            .parse()
            .inspect_err(|err| {
                log::error!("Couldn't parse value: {err}");
            })
            .ok();
        let value = value.unwrap_or(0);

        sats.set(value);
    };

    let send_claim = Action::new_local(move |&()| {
        async move {
            let cans = auth.auth_cans(expect_context()).await?;

            // TODO: do we still need this?
            handle_user_login(cans.clone(), auth.event_ctx(), None).await?;

            let req = hon_worker_common::WithdrawRequest {
                receiver: cans.user_principal(),
                amount: sats.get_untracked() as u128,
            };
            let sig = hon_worker_common::sign_withdraw_request(cans.identity(), req.clone())?;

            withdraw_sats_for_ckbtc(cans.user_canister(), req, sig).await
        }
    });
    let is_claiming = send_claim.pending();
    let claim_res = send_claim.value();
    Effect::new(move |_| {
        if let Some(res) = claim_res.get() {
            let nav = use_navigate();
            match res {
                Ok(_) => {
                    nav(
                        &format!("/hot-or-not/withdraw/success?sats={}", sats()),
                        Default::default(),
                    );
                }
                Err(err) => {
                    nav(
                        &format!("/hot-or-not/withdraw/failure?sats={}&err={err}", sats()),
                        Default::default(),
                    );
                }
            }
        }
    });
    view! {
        <div class="min-h-screen w-full flex flex-col text-white pt-2 pb-12 bg-black items-center overflow-x-hidden">
            <Header />
            <div class="w-full">
                <div class="flex flex-col items-center justify-center max-w-md mx-auto px-4 mt-4 pb-6">
                    <Suspense>
                    {move || {
                        let balance: Nat = try_or_redirect_opt!(details_res.get()?).balance.into();
                        Some(view! {
                            <BalanceDisplay balance />
                        })
                    }}
                    </Suspense>
                    <div class="flex flex-col gap-5 mt-8 w-full">
                        <span class="text-sm">Choose how much to redeem:</span>
                        <div id="input-card" class="rounded-lg bg-neutral-900 p-3 flex flex-col gap-8">
                            <div class="flex flex-col gap-3">
                                <div class="flex justify-between">
                                    <div class="flex gap-2 items-center">
                                        <span>You withdraw</span>
                                    </div>
                                    <input min={MIN_WITHDRAWAL_PER_TXN} max={MAX_WITHDRAWAL_PER_TXN} placeholder=format!("Min: {}", MIN_WITHDRAWAL_PER_TXN) disabled=is_claiming on:input=on_input type="text" inputmode="decimal" class="bg-neutral-800 h-10 w-44 rounded focus:outline focus:outline-1 focus:outline-primary-600 text-right px-4 text-lg" />
                                </div>
                                <div class="flex justify-between">
                                    <div class="flex gap-2 items-center">
                                        <span>You get</span>
                                    </div>
                                    <input disabled type="text" inputmode="decimal" class="bg-neutral-800 h-10 w-44 rounded focus:outline focus:outline-1 focus:outline-primary-600 text-right px-4 text-lg text-neutral-400" value=formated_dolrs />
                                </div>
                            </div>
                            <Suspense fallback=|| view! {
                                <button
                                    disabled
                                    class="rounded-lg px-5 py-2 text-sm text-center font-bold bg-brand-gradient-disabled"
                                >Please Wait</button>
                            }>
                            {move || {
                                let can_withdraw = true; // all of the money can be withdrawn
                                let invalid_input = sats() < MIN_WITHDRAWAL_PER_TXN as usize || sats() > MAX_WITHDRAWAL_PER_TXN as usize;
                                let is_claiming = is_claiming();
                                let message = if invalid_input {
                                    format!("Enter valid Amount. Min: {MIN_WITHDRAWAL_PER_TXN} Max: {MAX_WITHDRAWAL_PER_TXN}")
                                } else {
                                    match (can_withdraw, is_claiming) {
                                        (false, _) => "Not enough winnings".to_string(),
                                        (_, true) => "Claiming...".to_string(),
                                        (_, _) => "Withdraw Now!".to_string()
                                    }
                                };
                                Some(view! {
                                    <button
                                        disabled=invalid_input || !can_withdraw
                                        class=("pointer-events-none", is_claiming)
                                        class="rounded-lg px-5 py-2 text-sm text-center font-bold bg-brand-gradient disabled:bg-brand-gradient-disabled"
                                        on:click=move |_ev| {send_claim.dispatch(());}
                                    >{message}</button>
                                })
                            }}
                            </Suspense>
                        </div>
                        <span class="text-sm">1 Sats = {crate::consts::SATS_TO_BTC_CONVERSION_RATIO} BTC</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
