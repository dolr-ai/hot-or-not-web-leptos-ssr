use candid::{Nat, Principal};
use component::buttons::HighlightedLinkButton;
use component::{
    auth_providers::handle_user_login, back_btn::BackButton,
    icons::notification_icon::NotificationIcon, icons::telegram_icon::TelegramIcon,
    spinner::SpinnerFit, title::TitleText,
};
use consts::{CKBTC_LEDGER_CANISTER, SATS_CKBTC_CANISTER};
use futures::TryFutureExt;
use global_constants::{MAX_WITHDRAWAL_PER_TXN_SATS, MIN_WITHDRAWAL_PER_TXN_SATS};
use hon_worker_common::{HoNGameWithdrawReq, SatsBalanceInfo};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use log;
use state::{canisters::auth_state, server::HonWorkerJwt};
use utils::send_wrap;
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

    if req.amount < MIN_WITHDRAWAL_PER_TXN_SATS as u128
        || req.amount > MAX_WITHDRAWAL_PER_TXN_SATS as u128
    {
        log::error!(
            "Invalid withdraw amount, min amount: {}, max amount: {}, amount: {}",
            MIN_WITHDRAWAL_PER_TXN_SATS,
            MAX_WITHDRAWAL_PER_TXN_SATS,
            req.amount
        );
        return Err(ServerFnError::new(format!(
            "Invalid withdraw amount, min amount: {}, max amount: {}, amount: {}",
            MIN_WITHDRAWAL_PER_TXN_SATS, MAX_WITHDRAWAL_PER_TXN_SATS, req.amount
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
        <div id="back-nav" class="flex flex-col gap-20 items-center pb-16 w-full">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/" />
                    <span class="text-2xl font-bold">Withdraw</span>
                    <a
                        href="/wallet/notifications"
                        aria_disabled=true
                        class="text-xl font-semibold"
                    >
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
        <div id="total-balance" class="flex flex-col gap-1 items-center self-center">
            <span class="text-sm text-neutral-400">Total Sats balance</span>
            <div class="flex gap-3 items-center py-0.5 min-h-14">
                <img class="rounded-full size-9" src="/img/hotornot/sats.svg" alt="sats icon" />
                <span class="text-4xl font-bold">{format_sats!(balance)}</span>
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
    let balance = Resource::new(
        move || details_res.get().map(|r| r.ok().map(|d| d.balance.into())),
        |res| async move {
            if let Some(res) = res {
                res.unwrap_or_default()
            } else {
                Nat::from(0_usize)
            }
        },
    );

    let zero = Nat::from(0_usize);

    use state::canisters::unauth_canisters;
    let cans = unauth_canisters();
    let treasury_balance = Resource::new(
        || (),
        move |_| {
            let cans = cans.clone();
            let ledger = CKBTC_LEDGER_CANISTER.parse().unwrap();
            let treasury = SATS_CKBTC_CANISTER.parse().unwrap();

            send_wrap(async move {
                match cans.icrc1_balance_of(treasury, ledger).await {
                    Ok(balance) => balance,
                    Err(_) => Nat::from(0_usize),
                }
            })
        },
    );

    view! {
        <div class="flex overflow-x-hidden flex-col items-center pt-2 pb-12 w-full min-h-screen text-white bg-black">
            <Header />
            <div class="w-full">
                <div class="flex flex-col justify-center items-center px-4 pb-6 mx-auto mt-4 max-w-md">
                <Suspense fallback=|| {
                    view! {
                        <div class="size-14">
                            <SpinnerFit />
                        </div>
                    }
                }>
                {
                    let is_treasury_empty = treasury_balance.get().map(|balance| balance == 0_usize).unwrap_or(true);
                    log::info!(format!("Treasury balance, {}", treasury_balance.get()))
                    if is_treasury_empty {
                        view! {
                            <div class="flex flex-col gap-4 items-center max-w-sm mx-auto px-12">
                                <img src="/img/hotornot/bank-empty.webp" alt="Treasury is empty" class="size-40" />
                                <span class="text-lg font-semibold">Oh no!</span>
                                <span class="text-sm text-neutral-400 text-center">"The withdrawal pot is currently empty! Please come back in a while for the next top-up."</span>
                                <span class="text-sm text-neutral-400">"Need assistance?"</span>
                                <HighlightedLinkButton
                                    alt_style=true
                                    target="_blank".to_string()
                                    disabled=false
                                    classes="max-w-96 w-full mx-auto py-[12px] px-[20px]".to_string()
                                    href="https://t.me/HotOrNot_app".to_string()
                                >
                                    <div class="flex items-center gap-2">
                                        <TelegramIcon classes="size-6 text-pink-600".to_string() />
                                        <span>"Chat with us"</span>
                                    </div>
                                </HighlightedLinkButton>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            {balance
                                .get()
                                .map(|balance| view! { <BalanceDisplay balance=balance.clone() /> })}
                            <div class="flex flex-col gap-5 mt-8 w-full">
                                <span class="text-sm">Choose how much to redeem:</span>
                                <div
                                    id="input-card"
                                    class="flex flex-col gap-8 p-3 rounded-lg bg-neutral-900"
                                >
                                    <div class="flex flex-col gap-3">
                                        <div class="flex justify-between">
                                            <div class="flex gap-2 items-center">
                                                <span>You withdraw</span>
                                            </div>
                                            <input
                                                min=MIN_WITHDRAWAL_PER_TXN_SATS
                                                max=MAX_WITHDRAWAL_PER_TXN_SATS
                                                placeholder=format!("Min: {}", MIN_WITHDRAWAL_PER_TXN_SATS)
                                                disabled=is_claiming
                                                on:input=on_input
                                                type="text"
                                                inputmode="decimal"
                                                class="px-4 w-44 h-10 text-lg text-right rounded bg-neutral-800 focus:outline focus:outline-1 focus:outline-primary-600"
                                            />
                                        </div>
                                        <div class="flex justify-between">
                                            <div class="flex gap-2 items-center">
                                                <span>You get</span>
                                            </div>
                                            <input
                                                disabled
                                                type="text"
                                                inputmode="decimal"
                                                class="px-4 w-44 h-10 text-lg text-right rounded bg-neutral-800 text-neutral-400 focus:outline focus:outline-1 focus:outline-primary-600"
                                                value=formated_dolrs
                                            />
                                        </div>
                                    </div>
                                    {move || {
                                        let balance = balance.get().unwrap_or_else(|| Nat::from(0_usize));
                                        let can_withdraw = true;
                                        let invalid_input = sats() < MIN_WITHDRAWAL_PER_TXN_SATS as usize
                                            || sats() > MAX_WITHDRAWAL_PER_TXN_SATS as usize;
                                        let invalid_balance = sats() > balance || balance == zero;
                                        let is_claiming = is_claiming();
                                        let message = if invalid_balance {
                                            "Not enough balance".to_string()
                                        } else if invalid_input {
                                            format!(
                                                "Enter valid amount, min: {MIN_WITHDRAWAL_PER_TXN_SATS} max: {MAX_WITHDRAWAL_PER_TXN_SATS}",
                                            )
                                        } else {
                                            match (can_withdraw, is_claiming) {
                                                (false, _) => "Not enough winnings".to_string(),
                                                (_, true) => "Claiming...".to_string(),
                                                (_, _) => "Withdraw Now!".to_string(),
                                            }
                                        };
                                        view! {
                                            // all of the money can be withdrawn
                                            <button
                                                disabled=invalid_input || !can_withdraw
                                                class=("pointer-events-none", is_claiming)
                                                class="py-2 px-5 text-sm font-bold text-center rounded-lg bg-brand-gradient disabled:bg-brand-gradient-disabled"
                                                on:click=move |_ev| {
                                                    send_claim.dispatch(());
                                                }
                                            >
                                                {message}
                                            </button>
                                        }
                                    }}
                                </div>
                                <span class="text-sm">
                                    "1 Sats = "{consts::SATS_TO_BTC_CONVERSION_RATIO}
                                </span>
                            </div>
                        }.into_any()
                    }

                }
                </Suspense>
                </div>
            </div>
        </div>
    }
}
