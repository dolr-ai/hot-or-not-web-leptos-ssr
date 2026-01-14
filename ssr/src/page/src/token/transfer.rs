use crate::token::RootType;
use candid::Principal;
use component::buttons::GradientButton;
use component::{back_btn::BackButton, spinner::FullScreenSpinner, title::TitleText};
use leptos::either::Either;
use leptos::html;
use leptos::{ev, prelude::*};
use leptos_icons::*;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_params;
use server_fn::codec::Json;
use state::canisters::{auth_state, unauth_canisters};
use utils::mixpanel::mixpanel_events::*;
use utils::send_wrap;
use utils::{event_streaming::events::TokensTransferred, web::paste_from_clipboard};

use leptos_use::use_event_listener;
use yral_canisters_client::sns_root::ListSnsCanistersArg;
use yral_canisters_common::utils::token::balance::TokenBalance;
use yral_canisters_common::utils::token::TokenMetadata;
use yral_canisters_common::{Canisters, CanistersAuthWire};

use super::{popups::TokenTransferPopup, TokenParams};

#[server(
    input = Json
)]
async fn transfer_token_to_user_principal(
    cans_wire: CanistersAuthWire,
    destination_principal: Principal,
    ledger_canister: Principal,
    root_canister: Principal,
    amount: TokenBalance,
) -> Result<(), ServerFnError> {
    let cans = Canisters::from_wire(cans_wire, expect_context())?;
    // This must be called in a server function so the client can't interrupt a call to add_token
    cans.transfer_token_to_user_principal(
        destination_principal,
        ledger_canister,
        root_canister,
        amount,
    )
    .await?;

    Ok(())
}

#[component]
fn FormError<V: 'static + Send + Sync>(
    #[prop(into)] res: Signal<Result<V, String>>,
) -> impl IntoView {
    let err = Signal::derive(move || res.with(|r| r.as_ref().err().cloned()));

    view! {
        <Show when=move || res.with(|r| r.is_err())>
            <div class="flex flex-row gap-1 items-center w-full text-sm md:text-base">
                <Icon attr:class="text-red-600" icon=icondata::AiInfoCircleOutlined />
                <span class="text-white/60">{move || err().unwrap()}</span>
            </div>
        </Show>
    }
}

#[component]
fn TokenTransferInner(
    root: RootType,
    info: TokenMetadata,
    cans_wire: CanistersAuthWire,
) -> impl IntoView {
    if matches!(root, RootType::BTC { .. }) {
        return Either::Left(view! {
            <div>
                <Redirect path="/" />
            </div>
        });
    }

    let source_addr = Principal::self_authenticating(&cans_wire.id.from_key);

    let destination_ref = NodeRef::<html::Input>::new();
    let paste_destination: Action<_, _> = Action::new_unsync(move |&()| async move {
        let input = destination_ref.get()?;
        let principal = paste_from_clipboard().await?;
        input.set_value(&principal);
        #[cfg(feature = "hydrate")]
        {
            use web_sys::InputEvent;
            _ = input.dispatch_event(&InputEvent::new("input").unwrap());
        }
        Some(())
    });

    let destination_res = RwSignal::new(Ok::<_, String>(None::<Principal>));
    _ = use_event_listener(destination_ref, ev::input, move |_| {
        let Some(input) = destination_ref.get() else {
            return;
        };
        let principal_raw = input.value();
        let principal_res =
            Principal::from_text(principal_raw).map_err(|_| "Invalid principal".to_string());
        destination_res.set(principal_res.map(Some));
    });

    let amount_ref = NodeRef::<html::Input>::new();
    let Some(balance) = info.balance else {
        return Either::Left(view! {
            <div>
                <Redirect path="/" />
            </div>
        });
    };

    let max_amt = if balance
        .map_balance_ref(|b| b > &info.fees)
        .unwrap_or_default()
    {
        balance
            .map_balance_ref(|b| b.clone() - info.fees.clone())
            .unwrap()
    } else {
        TokenBalance::new(0u32.into(), info.decimals)
    };
    let max_amt_c = max_amt.clone();
    let set_max_amt = move || {
        let input = amount_ref.get()?;
        input.set_value(&max_amt.humanize_float());
        #[cfg(feature = "hydrate")]
        {
            use web_sys::InputEvent;
            _ = input.dispatch_event(&InputEvent::new("input").unwrap());
        }
        Some(())
    };

    let amt_res = RwSignal::new(Ok::<_, String>(None::<TokenBalance>));
    _ = use_event_listener(amount_ref, ev::input, move |_| {
        let Some(input) = amount_ref.get() else {
            return;
        };
        let amt_raw = input.value();
        let Ok(amt) = TokenBalance::parse(&amt_raw, info.decimals) else {
            amt_res.set(Err("Invalid amount".to_string()));
            return;
        };
        if amt > max_amt_c {
            amt_res.set(Err(
                "Sorry, there are not enough funds in this account".to_string()
            ));
        } else if amt.e8s == 0_u64 {
            amt_res.set(Err("Cannot send 0 tokens".to_string()));
        } else {
            amt_res.set(Ok(Some(amt)));
        }
    });

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();
    let base = unauth_canisters();

    let mix_fees = info.fees.clone();
    let token_name = info.symbol.clone();

    let send_action = Action::new(move |&()| {
        let root = root.clone();
        let fees = mix_fees.clone();
        let token_name = token_name.clone();
        let cans_wire = cans_wire.clone();
        let base = base.clone();

        send_wrap(async move {
            let cans = Canisters::from_wire(cans_wire.clone(), base)?;

            let destination = destination_res.get_untracked().unwrap().unwrap();

            let amt = amt_res.get_untracked().unwrap().unwrap();

            match root {
                RootType::Other(root) => {
                    let root_canister = cans.sns_root(root).await;
                    log::debug!("{root}");
                    let sns_cans = root_canister
                        .list_sns_canisters(ListSnsCanistersArg {})
                        .await
                        .unwrap();
                    let ledger_canister = sns_cans.ledger.unwrap();
                    log::debug!("ledger_canister: {ledger_canister:?}");

                    transfer_token_to_user_principal(
                        cans_wire.clone(),
                        destination,
                        ledger_canister,
                        root,
                        amt.clone(),
                    )
                    .await?;
                }
                RootType::BTC { ledger, .. } => {
                    cans.transfer_ck_token_to_user_principal(destination, ledger, amt.clone())
                        .await?;
                }
                RootType::USDC { ledger, .. } => {
                    cans.transfer_ck_token_to_user_principal(destination, ledger, amt.clone())
                        .await?;
                }
                RootType::CENTS => return Err(ServerFnError::new("Cents cannot be transferred")),
                RootType::SATS => return Err(ServerFnError::new("Satoshis cannot be transferred")),
            }
            TokensTransferred.send_event(amt.e8s.to_string(), destination, cans.clone());
            let is_logged_in = is_connected.get_untracked();

            let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
            let fees = fees.humanize_float().parse::<f64>().unwrap_or_default();
            let amount_transferred = amt.humanize_float().parse::<f64>().unwrap_or_default();
            MixPanelEvent::track_third_party_wallet_transferred(
                global,
                amount_transferred,
                destination.to_string(),
                token_name,
                fees,
            );

            Ok::<_, ServerFnError>(amt)
        })
    });
    let sending = send_action.pending();

    let valid = move || {
        amt_res.with(|r| matches!(r, Ok(Some(_))))
            && destination_res.with(|r| matches!(r, Ok(Some(_))))
            && !sending()
    };

    let is_btc = info.name.to_lowercase() == "btc";
    let placeholder = if is_btc {
        "Enter OISY wallet principal"
    } else {
        "Enter destination principal"
    };
    let formatted_balance = balance.humanize_float_truncate_to_dp(if is_btc { 5 } else { 2 });

    Either::Right(view! {
        <div class="flex flex-col gap-4 w-dvw min-h-dvh bg-neutral-950">
            <TitleText justify_center=false>
                <div class="grid grid-cols-3 justify-start w-full">
                    <BackButton fallback="/wallet" />
                    <span class="justify-self-center font-bold">Send {info.name}</span>
                </div>
            </TitleText>
            <div class="flex flex-col gap-4 items-center p-4 w-full md:gap-6">
                <div class="flex flex-col gap-2 items-center w-full">
                    <div class="flex flex-row justify-between w-full text-sm font-medium md:text-base text-neutral-400">
                        <span>Source</span>
                    </div>
                    <div class="flex flex-row gap-2 items-center w-full">
                        <p class="text-sm text-white/80 md:text-md">{source_addr.to_string()}</p>
                    </div>
                </div>
                <div class="flex flex-col gap-1 w-full">
                    <div class="flex justify-between">
                        <span class="text-sm md:text-base text-neutral-400">Destination</span>
                        {is_btc
                            .then_some(
                                view! {
                                    <a
                                        target="_blank"
                                        href="https://oisy.com"
                                        class="text-sm font-medium text-blue-500 md:text-base"
                                    >
                                        Open OISY Wallet
                                    </a>
                                },
                            )}
                    </div>
                    <div
                        class=("border-white/15", move || destination_res.with(|r| r.is_ok()))
                        class=("border-red", move || destination_res.with(|r| r.is_err()))
                        class="flex flex-row gap-2 justify-between p-3 w-full rounded-lg border bg-white/5"
                    >
                        <input
                            node_ref=destination_ref
                            class="w-full text-base text-white bg-transparent md:text-lg focus:outline-none placeholder-white/40"
                            placeholder=placeholder
                        />
                        <button on:click=move |_| {
                            paste_destination.dispatch(());
                        }>
                            <Icon
                                attr:class="text-neutral-600 text-lg md:text-xl"
                                icon=icondata::BsClipboard
                            />
                        </button>
                    </div>
                    <FormError res=destination_res />
                </div>
                <div class="flex flex-col gap-1 items-center w-full">
                    <div class="flex flex-row justify-between w-full text-sm md:text-base text-neutral-400">
                        <span>Amount</span>
                        <div class="flex gap-1 items-center">
                            <span class="text-xs font-medium text-neutral-400">
                                Balance: {formatted_balance}
                            </span>
                            <button
                                class="flex gap-2.5 justify-center items-center py-1.5 px-4 rounded-full border border-solid border-neutral-600 bg-neutral-700 text-"
                                on:click=move |_| _ = set_max_amt()
                            >
                                <span class="text-xs font-medium text-neutral">Max</span>
                            </button>
                        </div>
                    </div>
                    <input
                        node_ref=amount_ref
                        class=("border-white/15", move || amt_res.with(|r| r.is_ok()))
                        class=("border-red", move || amt_res.with(|r| r.is_err()))
                        class="p-3 w-full text-base text-white rounded-lg border md:text-lg focus:outline-none bg-white/5 placeholder-white/40"
                    />
                    <FormError res=amt_res />
                </div>
                <div class="flex flex-col w-full text-sm md:text-base text-white/60">
                    <span>Transaction Fee (billed to source)</span>
                    <span>{format!("{} {}", info.fees.humanize_float(), info.symbol)}</span>
                </div>
                <GradientButton
                    classes="w-full md:w-1/2"
                    on_click=move || {
                        send_action.dispatch(());
                    }
                    disabled=Signal::derive(move || !valid())
                >
                    Send
                </GradientButton>
            </div>
            <TokenTransferPopup token_name=info.symbol transfer_action=send_action />
        </div>
    })
}

#[component]
pub fn TokenTransfer() -> impl IntoView {
    let params = use_params::<TokenParams>();
    let auth = auth_state();
    let token_metadata_fetch = auth.derive_resource(params, |cans, params| {
        send_wrap(async move {
            let Ok(params) = params else {
                return Ok::<_, ServerFnError>(None);
            };
            let meta = cans
                .token_metadata_by_root_type(Some(cans.user_principal()), params.token_root.clone())
                .await
                .ok()
                .flatten();

            Ok(meta.map(|m| (m, params.token_root, CanistersAuthWire::from(cans))))
        })
    });

    view! {
        <Title text="YRAL - Token transfer" />
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let res = token_metadata_fetch.await;
                match res {
                    Err(e) => view! { <Redirect path=format!("/error?err={e}") /> }.into_any(),
                    Ok(None) => view! { <Redirect path="/" /> }.into_any(),
                    Ok(Some((info, root, cans_wire))) => {
                        view! { <TokenTransferInner info=info root=root cans_wire /> }.into_any()
                    }
                }
            })}
        </Suspense>
    }
}
