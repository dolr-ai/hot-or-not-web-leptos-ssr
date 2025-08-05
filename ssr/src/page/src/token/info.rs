use crate::token::RootType;
use crate::token::TokenInfoParams;
use crate::wallet::airdrop::AirdropPage;
use component::show_any::ShowAny;
use component::{
    back_btn::BackButton, share_popup::*, spinner::FullScreenSpinner, title::TitleText,
};
use leptos_router::components::Redirect;
use leptos_router::hooks::use_params;
use leptos_router::hooks::use_query;
use leptos_router::params::Params;
use state::canisters::auth_state;
use utils::send_wrap;
use utils::web::copy_to_clipboard;
use utils::UsernameOrPrincipal;

use crate::wallet::transactions::Transactions;
use candid::Principal;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};
use yral_canisters_common::cursored_data::transaction::IndexOrLedger;
use yral_canisters_common::utils::token::TokenMetadata;

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
            <span class="text-sm text-white md:text-base">{label}</span>
            <div class="flex justify-between py-4 px-2 w-full text-base rounded-xl md:text-lg bg-white/5 text-white/50">
                <div>{value}</div>
                <ShowAny when=move || copy>
                    <button on:click=copy_clipboard.clone()>
                        <Icon
                            attr:class="w-6 h-6 text-white/50 cursor-pointer hover:text-white/80"
                            icon=icondata::BiCopyRegular
                        />
                    </button>
                </ShowAny>
            </div>
        </div>
    }
}

#[component]
fn TokenDetails(meta: TokenMetadata) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6 p-4 w-full rounded-xl bg-white/5">
            <TokenField label="Ledger Id" value=meta.ledger.to_text() copy=true />
            <TokenField label="Description" value=meta.description />
            <TokenField label="Symbol" value=meta.symbol />
        </div>
    }
}

pub fn generate_share_link(root: &RootType, id: UsernameOrPrincipal) -> String {
    format!("/token/info/{root}/{id}?airdrop_amt=100")
}

#[component]
fn TokenInfoInner(
    root: RootType,
    meta: TokenMetadata,
    id: Option<UsernameOrPrincipal>,
    key_principal: Option<Principal>,
    is_user_principal: bool,
) -> impl IntoView {
    let meta_c1 = meta.clone();
    let meta_c = meta.clone();
    let detail_toggle = RwSignal::new(false);
    let view_detail_icon = Signal::derive(move || {
        if detail_toggle() {
            icondata::AiUpOutlined
        } else {
            icondata::AiDownOutlined
        }
    });
    let share_link = id.map(|id| generate_share_link(&root, id));
    let message = share_link.clone().map(|share_link|format!(
        "Hey! Check out the token: {} I created on YRAL 👇 {}. I just minted my own token—come see and create yours! 🚀 #YRAL #TokenMinter",
        meta.symbol,  share_link
    ));

    let decimals = meta.decimals;

    view! {
        <div class="flex flex-col gap-4 w-dvw min-h-dvh bg-neutral-800">
            <TitleText justify_center=false>
                <div class="grid grid-cols-3 justify-start w-full">
                    <BackButton fallback="/wallet" />
                    <span class="justify-self-center font-bold">Token details</span>
                </div>
            </TitleText>
            <div class="flex flex-col gap-8 items-center px-8 w-full md:px-10">
                <div class="flex flex-col gap-6 justify-self-start items-center w-full md:gap-8">
                    <div class="flex flex-col gap-4 p-4 w-full rounded-xl bg-white/5 drop-shadow-lg">
                        <div class="flex flex-row justify-between items-center">
                            <div class="flex flex-row gap-2 items-center">
                                <div class="relative">
                                    <img
                                        class="object-cover w-14 h-14 rounded-full cursor-pointer md:w-18 md:h-18"
                                        src=meta.logo_b64
                                    />
                                </div>
                                <span class="text-base font-semibold text-white md:text-lg">
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
                                        .into_any()
                                })}
                        </div>

                        <ShowAny when=move || key_principal.clone().is_some()>
                            <div class="flex flex-row justify-between items-center p-1 border-b border-white">
                                <span class="text-xs text-green-500 md:text-sm">Balance</span>
                                <span class="text-lg text-white md:text-xl">
                                    {meta
                                        .balance
                                        .clone()
                                        .map(|balance| {
                                            view! {
                                                <span class="font-bold">
                                                    {format!("{} ", balance.humanize_float_truncate_to_dp(8))}
                                                </span>
                                                <span>{meta_c1.symbol.clone()}</span>
                                            }
                                        })}
                                </span>
                            </div>
                        </ShowAny>
                        <button
                            on:click=move |_| detail_toggle.update(|t| *t = !*t)
                            class="flex flex-row gap-2 justify-center items-center p-1 w-full text-white bg-transparent"
                        >
                            <span class="text-xs md:text-sm">View details</span>
                            <div class="p-1 rounded-full bg-white/15">
                                <Icon
                                    attr:class="text-xs md:text-sm text-white"
                                    icon=view_detail_icon
                                />
                            </div>
                        </button>
                    </div>
                    <ShowAny when=detail_toggle>
                        <TokenDetails meta=meta_c.clone() />
                    </ShowAny>
                </div>
                <ShowAny when=move || is_user_principal>
                    <a
                        href=format!("/token/transfer/{}", root.to_string())
                        class="fixed right-4 left-4 bottom-20 z-50 p-3 text-center text-white rounded-full md:text-lg bg-primary-600"
                    >
                        Send
                    </a>
                </ShowAny>
                {if let Some(key_principal) = key_principal {
                    view! {
                        <Transactions
                            source=IndexOrLedger::Index {
                                key_principal,
                                index: meta.index,
                            }
                            symbol=meta.symbol.clone()
                            decimals
                        />
                    }
                        .into_any()
                } else {
                    view! {
                        <Transactions
                            source=IndexOrLedger::Ledger(meta.ledger)
                            symbol=meta.symbol.clone()
                            decimals
                        />
                    }
                        .into_any()
                }}
            </div>
        </div>
    }.into_any()
}

#[derive(Params, PartialEq, Clone, Serialize, Deserialize)]
pub struct TokenKeyParam {
    id: UsernameOrPrincipal,
}

#[derive(Params, PartialEq, Clone, Serialize, Deserialize, Debug)]
struct AirdropParam {
    airdrop_amt: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct TokenInfoResponse {
    meta: TokenMetadata,
    root: RootType,
    #[serde(default)]
    id: Option<UsernameOrPrincipal>,
    #[serde(default)]
    key_principal: Option<Principal>,
    is_user_principal: bool,
    is_token_viewer_airdrop_claimed: bool,
}

#[component]
pub fn TokenInfo() -> impl IntoView {
    let params = use_params::<TokenInfoParams>();
    let id_param = use_params::<TokenKeyParam>();
    let airdrop_param = use_query::<AirdropParam>();
    let id = move || id_param.get().map(|p| p.id).ok();

    let auth = auth_state();

    let token_metadata_fetch = auth.derive_resource(
        move || (params.get(), id()),
        move |cans, (params_result, id)| {
            send_wrap(async move {
                let params = match params_result {
                    Ok(p) => p,
                    Err(_) => return Ok::<_, ServerFnError>(None),
                };
                let key_principal = match id.as_ref() {
                    Some(UsernameOrPrincipal::Principal(p)) => Some(*p),
                    Some(UsernameOrPrincipal::Username(u)) => {
                        let meta = cans.get_user_metadata(u.clone()).await.ok().flatten();
                        meta.map(|m| m.user_principal)
                    }
                    _ => None,
                };

                let meta = cans
                    .token_metadata_by_root_type(key_principal, params.token_root.clone())
                    .await
                    .ok()
                    .flatten();

                let token_root = &params.token_root;
                let res = match (meta, token_root) {
                    (Some(m), RootType::Other(root)) => {
                        let Some(token_owner) = m.token_owner.clone() else {
                            return Ok(Some(TokenInfoResponse {
                                meta: m,
                                root: token_root.clone(),
                                id,
                                key_principal,
                                is_user_principal: Some(cans.user_principal()) == key_principal,
                                is_token_viewer_airdrop_claimed: true,
                            }));
                        };
                        let is_airdrop_claimed = cans
                            .get_airdrop_status(
                                token_owner.canister_id,
                                *root,
                                cans.user_principal(),
                            )
                            .await
                            .unwrap_or(true);

                        Some(TokenInfoResponse {
                            meta: m,
                            root: token_root.clone(),
                            id,
                            key_principal,
                            is_user_principal: Some(cans.user_principal()) == key_principal,
                            is_token_viewer_airdrop_claimed: is_airdrop_claimed,
                        })
                    }
                    (Some(m), _) => Some(TokenInfoResponse {
                        meta: m,
                        root: token_root.clone(),
                        id,
                        key_principal,
                        is_user_principal: Some(cans.user_principal()) == key_principal,
                        is_token_viewer_airdrop_claimed: true,
                    }),
                    _ => None,
                };

                Ok(res)
            })
        },
    );

    view! {
        <Title text="YRAL - Token Info" />
        <Suspense fallback=FullScreenSpinner>
            {move || {
                token_metadata_fetch
                    .get()
                    .map(|info| {
                        match info {
                            Ok(
                                Some(
                                    TokenInfoResponse {
                                        meta,
                                        root,
                                        id,
                                        key_principal,
                                        is_user_principal,
                                        is_token_viewer_airdrop_claimed,
                                    }
                                )
                            ) => {
                                if let Ok(AirdropParam { airdrop_amt }) = airdrop_param.get() {
                                    if !is_token_viewer_airdrop_claimed
                                        && meta.token_owner.clone().map(|t| t.principal_id)
                                            == key_principal && !is_user_principal
                                    {
                                        return view! {
                                            <AirdropPage airdrop_amount=airdrop_amt meta />
                                        }
                                            .into_any();
                                    }
                                }
                                view! {
                                    <TokenInfoInner
                                        root
                                        id
                                        key_principal
                                        meta
                                        is_user_principal=is_user_principal
                                    />
                                }
                                    .into_any()
                            }
                            _ => view! { <Redirect path="/wallet" /> }.into_any(),
                        }
                    })
            }}

        </Suspense>
    }
}
