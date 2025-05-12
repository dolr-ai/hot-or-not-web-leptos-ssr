use candid::{Nat, Principal};
use component::{
    auth_providers::handle_user_login,
    back_btn::BackButton,
    icons::{information_icon::Information, notification_icon::NotificationIcon},
    title::TitleText,
    tooltip::Tooltip,
};
use futures::TryFutureExt;
use hon_worker_common::SatsBalanceInfo;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use log;
use state::canisters::authenticated_canisters;
use utils::{send_wrap, try_or_redirect_opt};
use yral_canisters_common::{utils::token::balance::TokenBalance, Canisters};

pub mod result;

macro_rules! format_sats {
    ($num:expr) => {
        TokenBalance::new($num, 0).humanize_float_truncate_to_dp(0)
    };
}

/// Details for withdrawal functionality
type Details = SatsBalanceInfo;

async fn load_withdrawal_details(user_canister: Principal) -> Result<Details, String> {
    let url: reqwest::Url = hon_worker_common::WORKER_URL
        .parse()
        .expect("Url to be valid");
    let balance_info = url
        .join(&format!("/balance/{user_canister}"))
        .expect("Url to be valid");

    let balance_info: SatsBalanceInfo = reqwest::get(balance_info)
        .await
        .map_err(|_| "failed to load balance".to_string())?
        .json()
        .await
        .map_err(|_| "failed to read response body".to_string())?;

    Ok(balance_info)
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
            <span class="text-neutral-400 text-sm">Total Cent balance</span>
            <div class="flex items-center gap-3 min-h-14 py-0.5">
                <img class="size-9" src="/img/hotornot/sats.webp" alt="sats icon" />
                <span class="font-bold text-4xl">{format_sats!(balance)}</span>
            </div>
        </div>
    }
}

#[component]
pub fn HonWithdrawal() -> impl IntoView {
    let auth_wire = authenticated_canisters();
    let details_res = Resource::new(
        move || (),
        move |_| {
            send_wrap(async move {
                let cans_wire = auth_wire.await?;
                load_withdrawal_details(cans_wire.user_canister)
                    .map_err(ServerFnError::new)
                    .await
            })
        },
    );
    let sats = RwSignal::new(0usize);
    let formated_dolrs = move || {
        format!(
            "{} BTC",
            TokenBalance::new(sats().into(), 8).humanize_float_truncate_to_dp(5)
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

    let auth_wire = authenticated_canisters();
    let send_claim = Action::new_local(move |&()| {
        let auth_wire = auth_wire;
        async move {
            let auth_wire = auth_wire.await.map_err(ServerFnError::new)?;

            let cans = Canisters::from_wire(auth_wire.clone(), expect_context())
                .map_err(ServerFnError::new)?;

            handle_user_login(cans.clone(), None).await?;

            Err::<(), _>(ServerFnError::new("Not implmented yet"))
        }
    });
    let is_claiming = send_claim.pending();
    let claim_res = send_claim.value();
    Effect::new(move |_| {
        if let Some(res) = claim_res() {
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
                                        <Tooltip icon=Information title="Withdrawal Tokens" description="Only sats earned above your airdrop amount can be withdrawn." />
                                    </div>
                                    <input disabled=is_claiming on:input=on_input type="text" inputmode="decimal" class="bg-neutral-800 h-10 w-40 rounded focus:outline focus:outline-1 focus:outline-primary-600 text-right px-4 text-lg" />
                                </div>
                                <div class="flex justify-between">
                                    <div class="flex gap-2 items-center">
                                        <span>You get</span>
                                    </div>
                                    <input disabled type="text" inputmode="decimal" class="bg-neutral-800 h-10 w-40 rounded focus:outline focus:outline-1 focus:outline-primary-600 text-right px-4 text-lg text-neutral-400" value=formated_dolrs />
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
                                let no_input = sats() == 0usize;
                                let is_claiming = is_claiming();
                                let message = if no_input {
                                    "Enter Amount"
                                } else {
                                    match (can_withdraw, is_claiming) {
                                        (false, _) => "Not enough winnings",
                                        (_, true) => "Claiming...",
                                        (_, _) => "Withdraw Now!"
                                    }
                                };
                                Some(view! {
                                    <button
                                        disabled=no_input || !can_withdraw
                                        class=("pointer-events-none", is_claiming)
                                        class="rounded-lg px-5 py-2 text-sm text-center font-bold bg-brand-gradient disabled:bg-brand-gradient-disabled"
                                        on:click=move |_ev| {send_claim.dispatch(());}
                                    >{message}</button>
                                })
                            }}
                            </Suspense>
                        </div>
                        <span class="text-sm">1 Sats = 0.00000001 BTC</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
