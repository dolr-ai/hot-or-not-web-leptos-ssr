use candid::Principal;
use leptos::*;
use leptos_icons::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::page::token::TokenInfoParams;
use crate::state::canisters::authenticated_canisters;
use crate::{
    component::{back_btn::BackButton, share_popup::*, spinner::FullScreenSpinner, title::Title},
    page::wallet::{transactions::Transactions, txn::IndexOrLedger},
    utils::{
        token::{token_metadata_by_root, TokenMetadata},
        web::copy_to_clipboard,
    },
};
#[component]
fn TokenField(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional, default = false)] copy: bool,
) -> impl IntoView {
    let copy_payload = value.clone();
    let copy_clipboard = move |_| {
        copy_to_clipboard(&copy_payload);
    };
    view! {
        <div class="flex flex-col gap-1 w-full">
            <span class="text-white text-sm md:text-base">{label}</span>
            <div class="bg-white/5 text-base md:text-lg text-white/50 px-2 py-4 rounded-xl w-full flex justify-between">
                <div>{value}</div>
                <Show when=move || copy>
                    <button on:click=copy_clipboard.clone()>
                        <Icon
                            class="w-6 h-6 text-white/50 cursor-pointer hover:text-white/80"
                            icon=icondata::BiCopyRegular
                        />
                    </button>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn TokenDetails(meta: TokenMetadata) -> impl IntoView {
    view! {
        <div class="flex flex-col w-full gap-6 p-4 rounded-xl bg-white/5">
            <TokenField label="Ledger Id" value=meta.ledger.to_text() copy=true />
            <TokenField label="Description" value=meta.description />
            <TokenField label="Symbol" value=meta.symbol />
        </div>
    }
}

#[component]
fn TokenInfoInner(
    root: Principal,
    meta: TokenMetadata,
    key_principal: Option<Principal>,
    is_user_principal: bool,
) -> impl IntoView {
    let meta_c1 = meta.clone();
    let meta_c = meta.clone();
    let detail_toggle = create_rw_signal(false);
    let view_detail_icon = Signal::derive(move || {
        if detail_toggle() {
            icondata::AiUpOutlined
        } else {
            icondata::AiDownOutlined
        }
    });
    let share_link = key_principal
        .map(|key_principal| format!("/token/info/{root}/{key_principal}?airdrop_amt=100"));
    let message = share_link.clone().map(|share_link|format!(
        "Hey! Check out the token: {} I created on YRAL 👇 {}. I just minted my own token—come see and create yours! 🚀 #YRAL #TokenMinter",
        meta.symbol,  share_link
    ));

    view! {
        <div class="w-dvw min-h-dvh bg-neutral-800  flex flex-col gap-4">
            <Title justify_center=false>
                <div class="grid grid-cols-3 justify-start w-full">
                    <BackButton fallback="/wallet" />
                    <span class="font-bold justify-self-center">Token details</span>
                </div>
            </Title>
            <div class="flex flex-col w-full items-center px-8 md:px-10 gap-8">
                <div class="flex flex-col justify-self-start w-full gap-6 md:gap-8 items-center">
                    <div class="flex flex-col gap-4 w-full bg-white/5 p-4 drop-shadow-lg rounded-xl">
                        <div class="flex flex-row justify-between items-center">
                            <div class="flex flex-row gap-2 items-center">
                                <img
                                    class="object-cover h-14 w-14 md:w-18 md:h-18 rounded-full"
                                    src=meta.logo_b64
                                />
                                <span class="text-base md:text-lg font-semibold text-white">
                                    {meta.name}
                                </span>
                            </div>
                            {share_link
                                .zip(message)
                                .map(|(share_link, message)| {
                                    view! {
                                        <ShareButtonWithFallbackPopup
                                            share_link
                                            message
                                            style="w-12 h-12".into()
                                        />
                                    }
                                })}
                        </div>

                        <Show when= move|| key_principal.clone().is_some()>
                            <div class="flex flex-row justify-between border-b p-1 border-white items-center">
                                <span class="text-xs md:text-sm text-green-500">Balance</span>
                                <span class="text-lg md:text-xl text-white">
                                    {meta
                                        .balance.clone()
                                        .map(|balance| {
                                            view! {
                                                <span class="font-bold">
                                                    {format!("{} ", balance.humanize_float_truncate_to_dp(2))}
                                                </span>
                                                <span>{meta_c1.symbol.clone()}</span>
                                    }
                                    })}
                                </span>
                            </div>
                        </Show>
                        <button
                            on:click=move |_| detail_toggle.update(|t| *t = !*t)
                            class="w-full bg-transparent p-1 flex flex-row justify-center items-center gap-2 text-white"
                        >
                            <span class="text-xs md:text-sm">View details</span>
                            <div class="p-1 bg-white/15 rounded-full">
                                <Icon class="text-xs md:text-sm text-white" icon=view_detail_icon />
                            </div>
                        </button>
                    </div>
                    <Show when=detail_toggle>
                        <TokenDetails meta=meta_c.clone() />
                    </Show>
                </div>
                    <Show when= move || is_user_principal>
                        <a
                            href=format!("/token/transfer/{root}")
                            class="fixed bottom-20 left-4 right-4 p-3 bg-primary-600 text-white text-center md:text-lg rounded-full z-50"
                        >
                            Send
                        </a>
                    </Show>
                {if key_principal.is_some() {
                    view! { <Transactions source=IndexOrLedger::Index(meta.index) key_principal symbol=meta.symbol.clone()/> }
                } else {
                    view! {
                        <Transactions
                            source=IndexOrLedger::Ledger(meta.ledger)
                            key_principal=None
                            symbol=meta.symbol.clone()
                        />
                    }
                }}
            </div>
        </div>
    }
}
#[derive(Params, PartialEq, Clone, Serialize, Deserialize)]
pub struct TokenKeyParam {
    key_principal: String,
}
#[component]
pub fn TokenInfo() -> impl IntoView {
    let params = use_params::<TokenInfoParams>();
    let key_principal = use_params::<TokenKeyParam>();
    let key_principal = move || {
        key_principal.with(|p| {
            let TokenKeyParam { key_principal, .. } = p.as_ref().ok()?;
            Principal::from_text(key_principal).ok()
        })
    };
    let token_metadata_fetch = authenticated_canisters().derive(
        move || (params(), key_principal()),
        move |cans_wire, (params, key_principal)| async move {
            let Ok(params) = params else {
                return Ok::<_, ServerFnError>(None);
            };
            let cans = cans_wire?.canisters()?;
            let meta = token_metadata_by_root(&cans, key_principal, params.token_root).await?;

            Ok(meta.map(|m| {
                (
                    m,
                    params.token_root,
                    key_principal,
                    Some(cans.user_principal()) == key_principal,
                )
            }))
        },
    );

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || {
                token_metadata_fetch()
                    .and_then(|info| info.ok())
                    .map(|info| {
                        match info {
                            Some((metadata, root, key_principal, is_user_principal)) => {
                                view! {
                                    <TokenInfoInner
                                        root
                                        key_principal
                                        meta=metadata
                                        is_user_principal=is_user_principal
                                    />
                                }
                            }
                            None => view! { <Redirect path="/" /> },
                        }
                    })
            }}

        </Suspense>
    }
}
