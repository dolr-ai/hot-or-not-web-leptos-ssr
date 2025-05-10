use leptos::task::spawn_local;
use leptos_meta::*;

use candid::Nat;
use candid::Principal;
use codee::string::FromToStringCodec;
use component::overlay::PopupOverlay;
use consts::ICPUMP_LISTING_PAGE_SIZE;
use consts::USER_PRINCIPAL_STORE;
use futures::StreamExt;
use leptos::html::Div;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_use::use_cookie;
use leptos_use::use_intersection_observer_with_options;
use leptos_use::use_media_query;
use leptos_use::UseIntersectionObserverOptions;
use serde::Deserialize;
use serde::Serialize;
use state::canisters::authenticated_canisters;
use std::collections::VecDeque;
use utils::event_streaming::events::CentsAdded;
use utils::send_wrap;
use yral_canisters_common::utils::token::TokenOwner;
use yral_canisters_common::Canisters;

use component::buttons::HighlightedLinkButton;
use component::icons::airdrop_icon::AirdropIcon;
use component::icons::arrow_left_right_icon::ArrowLeftRightIcon;
use component::icons::chevron_right_icon::ChevronRightIcon;
use component::icons::eye_hide_icon::EyeHiddenIcon;
use component::icons::send_icon::SendIcon;
use component::icons::share_icon::ShareIcon;
use component::share_popup::ShareContent;
use utils::host::get_host;
use utils::token::firestore::init_firebase;
use utils::token::firestore::listen_to_documents;
use utils::token::icpump::get_paginated_token_list;
use utils::token::icpump::TokenListItem;

use crate::wallet::airdrop::AirdropPopup;
use component::overlay::ShadowOverlay;

pub mod ai;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessedTokenListResponse {
    pub token_details: TokenListItem,
    pub root: Principal,
    pub token_owner: Option<TokenOwner>,
    pub is_airdrop_claimed: bool,
}

#[cfg(any(feature = "local-bin", feature = "local-lib"))]
pub async fn process_token_list_item(
    token_list_item: Vec<TokenListItem>,
    key_principal: Principal,
) -> Vec<ProcessedTokenListResponse> {
    use yral_canisters_common::utils::token::TokenOwner;
    token_list_item
        .into_iter()
        .map(|item| {
            let root_principal = Principal::from_text(
                item.link
                    .trim_end_matches('/')
                    .split('/')
                    .next_back()
                    .ok_or(ServerFnError::new("Not root given"))
                    .unwrap_or_default(),
            )
            .unwrap_or(Principal::anonymous());
            ProcessedTokenListResponse {
                token_details: item,
                root: root_principal,
                token_owner: Some(TokenOwner {
                    principal_id: key_principal,
                    canister_id: key_principal,
                }),
                is_airdrop_claimed: false,
            }
        })
        .collect()
}

#[cfg(not(any(feature = "local-bin", feature = "local-lib")))]
pub async fn process_token_list_item(
    token_list_item: Vec<TokenListItem>,
    key_principal: Principal,
) -> Vec<ProcessedTokenListResponse> {
    use futures::stream::FuturesOrdered;
    use state::canisters::unauth_canisters;

    let mut fut = FuturesOrdered::new();

    for token in token_list_item {
        fut.push_back(async move {
            let cans = unauth_canisters();
            let root_principal = Principal::from_text(
                token
                    .link
                    .trim_end_matches('/')
                    .split('/')
                    .next_back()
                    .ok_or(ServerFnError::new("Not root given"))
                    .unwrap_or_default(),
            )
            .unwrap_or(Principal::anonymous());

            let token_owner_canister_id = cans
                .get_token_owner(root_principal)
                .await
                .unwrap_or_default();
            let is_airdrop_claimed = if let Some(token_owner) = &token_owner_canister_id {
                cans.get_airdrop_status(token_owner.canister_id, root_principal, key_principal)
                    .await
                    .unwrap_or(true)
            } else {
                true
            };
            // let token_owner = cans.individual_user(token_owner_canister_id.unwrap()).await;
            // token_owner_principal_id: token_owner.get_profile_details().await.unwrap().principal_id,
            ProcessedTokenListResponse {
                token_details: token,
                root: root_principal,
                is_airdrop_claimed,
                token_owner: token_owner_canister_id,
            }
        });
    }

    fut.collect().await
}

#[component]
pub fn ICPumpListingFeed() -> impl IntoView {
    let page = RwSignal::new(1);
    let end = RwSignal::new(false);
    let loading = RwSignal::new(true);
    let (curr_principal, _) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);
    let token_list: RwSignal<Vec<ProcessedTokenListResponse>> = RwSignal::new(vec![]);
    let new_token_list: RwSignal<VecDeque<ProcessedTokenListResponse>> =
        RwSignal::new(VecDeque::new());

    let fetch_res = Resource::new(
        move || page.get(),
        move |page| {
            send_wrap(async move {
                let cans = authenticated_canisters().await;
                let cans = Canisters::from_wire(cans.unwrap(), expect_context()).unwrap();
                new_token_list.set(VecDeque::new());

                loading.set(true);

                let mut fetched_token_list = process_token_list_item(
                    get_paginated_token_list(page).await.unwrap(),
                    cans.user_principal(),
                )
                .await;

                if fetched_token_list.len() < ICPUMP_LISTING_PAGE_SIZE {
                    end.set(true);
                }

                token_list.update(|t| {
                    t.append(&mut fetched_token_list);
                });

                loading.set(false);
            })
        },
    );

    Effect::new(move |_| {
        fetch_res.refetch();
        if let Some(principal) = curr_principal.get() {
            spawn_local(async move {
                let (_app, firestore) = init_firebase();
                let mut stream = listen_to_documents(&firestore);
                while let Some(doc) = stream.next().await {
                    let doc = process_token_list_item(doc, principal).await;
                    for item in doc {
                        new_token_list.try_update(move |list| {
                            list.push_front(item.clone());
                        });
                    }
                }
            })
        }
    });

    let target = NodeRef::<Div>::new();

    use_intersection_observer_with_options(
        target,
        move |entries, _| {
            let is_intersecting = entries.first().map(|entry| entry.is_intersecting());

            let loading = loading.get_untracked();
            let end = end.get_untracked();

            if let (Some(true), false, false) = (is_intersecting, loading, end) {
                page.update(|p| {
                    *p += 1;
                });
            }
        },
        UseIntersectionObserverOptions::default().thresholds(vec![0.1]),
    );

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <For
                each=move || new_token_list.get()
                key=|t| t.token_details.token_symbol.clone()
                children=move |t| {
                    view! {
                        <TokenCard
                            is_new_token=true
                            details=t.token_details
                            is_airdrop_claimed=t.is_airdrop_claimed
                            root=t.root
                            token_owner=t.token_owner
                        />
                    }
                }
            />
            {move || {
                token_list
                    .get()
                    .iter()
                    .map(|t| {
                        view! {
                            <TokenCard
                                details=t.token_details.clone()
                                is_airdrop_claimed=t.is_airdrop_claimed
                                root=t.root
                                token_owner=t.token_owner.clone()
                            />
                        }
                    })
                    .collect_view()
            }}

            <Show when=move || loading.get()>
                <TokenCardLoadingFeed />
            </Show>
        </div>

        <div class="w-full p-4" node_ref=target></div>
    }
}

#[component]
pub fn ICPumpLanding() -> impl IntoView {
    // TODO: add the pump-ai icon here, as shown in the new ui for pnd game
    view! {
        <Title text="ICPump - Home" />
        <div class="min-h-screen bg-black text-white  flex flex-col gap-4 px-4 md:px-8 py-6 font-kumbh">
            <div class="flex lg:flex-row gap-4 flex-col items-center justify-center">
                <div class="lg:left-0 lg:top-0 flex items-center gap-4">
                    <div>Follow us:</div>
                    <div class="flex items-center gap-4">
                        <XIcon
                            href="https://x.com/Yral_app".to_string()
                            classes="w-10 h-10".to_string()
                        />
                        <InstagramIcon
                            href="https://instagram.com/yral_app".to_string()
                            classes="w-10 h-10".to_string()
                        />
                        <TelegramIcon
                            href="https://t.me/+c-LTX0Cp-ENmMzI1".to_string()
                            classes="w-10 h-10".to_string()
                        />
                    </div>
                </div>
                <HighlightedLinkButton
                    classes="max-w-96 lg:max-w-[32.5%]".to_string()
                    href="/token/create".to_string()
                >
                    "Create a new coin"
                </HighlightedLinkButton>
            </div>
            <div class="flex flex-col gap-8 pb-24">
                <ICPumpListingFeed />
            </div>

        </div>
    }
}

#[component]
pub fn TokenCardFallback() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 pt-3 pb-4 px-3 md:px-4 w-full text-xs rounded-lg bg-neutral-900/90 font-kumbh">
            <div class="flex gap-3 items-stretch">
                <div class="w-[7rem] h-[7rem] rounded-[4px] shrink-0 bg-white/15 animate-pulse"></div>
                <div class="flex flex-col justify-between overflow-hidden w-full gap-2">
                    <div class="flex flex-col gap-2">
                        <div class="flex gap-4 justify-between items-center w-full">
                            <div class="h-7 w-32 bg-white/15 animate-pulse rounded"></div>
                            <div class="h-7 w-16 bg-white/15 animate-pulse rounded"></div>
                        </div>
                        <div class="h-12 w-full bg-white/15 animate-pulse rounded"></div>
                    </div>
                    <div class="flex gap-2 justify-between items-center">
                        <div class="h-5 w-48 bg-white/15 animate-pulse rounded"></div>
                        <div class="h-5 w-24 bg-white/15 animate-pulse rounded"></div>
                    </div>
                </div>
            </div>
            <div class="flex gap-4 justify-between items-center p-2">
                <div class="flex flex-col items-center gap-1">
                    <div class="w-[1.875rem] h-[1.875rem] bg-white/15 animate-pulse rounded"></div>
                    <div class="w-10 h-3 bg-white/15 animate-pulse rounded"></div>
                </div>
                <div class="flex flex-col items-center gap-1">
                    <div class="w-[1.875rem] h-[1.875rem] bg-white/15 animate-pulse rounded"></div>
                    <div class="w-14 h-3 bg-white/15 animate-pulse rounded"></div>
                </div>
                <div class="flex flex-col items-center gap-1">
                    <div class="w-[1.875rem] h-[1.875rem] bg-white/15 animate-pulse rounded"></div>
                    <div class="w-12 h-3 bg-white/15 animate-pulse rounded"></div>
                </div>
                <div class="flex flex-col items-center gap-1">
                    <div class="w-[1.875rem] h-[1.875rem] bg-white/15 animate-pulse rounded"></div>
                    <div class="w-10 h-3 bg-white/15 animate-pulse rounded"></div>
                </div>
                <div class="flex flex-col items-center gap-1">
                    <div class="w-[1.875rem] h-[1.875rem] bg-white/15 animate-pulse rounded"></div>
                    <div class="w-12 h-3 bg-white/15 animate-pulse rounded"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TokenCard(
    details: TokenListItem,
    #[prop(optional, default = false)] is_new_token: bool,
    root: Principal,
    is_airdrop_claimed: bool,
    token_owner: Option<TokenOwner>,
) -> impl IntoView {
    let show_nsfw = RwSignal::new(false);

    let share_link = RwSignal::new("".to_string());
    let share_link_coin = format!("/token/info/{}/{}", root, details.user_id);
    let symbol = details.token_symbol.clone();
    let token_symbol_c = details.token_symbol.clone();
    let token_symbol_c2 = token_symbol_c.clone();
    let share_message = move || {
        format!(
        "Hey! Check out the token: {} I created on YRAL 👇 {}. I just minted my own token—come see and create yours! 🚀 #YRAL #TokenMinter",
        token_symbol_c2,
        share_link.get(),
    )
    };
    let pop_up = RwSignal::new(false);
    let airdrop_popup = RwSignal::new(false);
    let base_url = get_host();

    let claimed = RwSignal::new(is_airdrop_claimed);
    let buffer_signal = RwSignal::new(false);
    let cans_res = authenticated_canisters();
    let token_owner_c = token_owner.clone();
    let airdrop_action = Action::new_local(move |&()| {
        let cans_res = cans_res;
        let token_owner_cans_id = token_owner_c.clone().unwrap().canister_id;
        let token_symbol = token_symbol_c.clone();
        airdrop_popup.set(true);
        async move {
            if claimed.get() && !buffer_signal.get() {
                return Ok(());
            }
            buffer_signal.set(true);
            let cans_wire = cans_res
                .get_untracked()
                .ok_or_else(|| ServerFnError::new("Auth Failed"))??;
            let cans = Canisters::from_wire(cans_wire, expect_context())?;
            let token_owner = cans.individual_user(token_owner_cans_id).await;

            token_owner
                .request_airdrop(
                    root,
                    None,
                    Into::<Nat>::into(100u64) * 10u64.pow(8),
                    cans.user_canister(),
                )
                .await?;

            let user = cans.individual_user(cans.user_canister()).await;
            user.add_token(root).await?;

            if token_symbol == "COYNS" || token_symbol == "CENTS" {
                CentsAdded.send_event("airdrop".to_string(), 100);
            }

            buffer_signal.set(false);
            claimed.set(true);
            Ok::<_, ServerFnError>(())
        }
    });

    let airdrop_disabled =
        Signal::derive(move || token_owner.is_some() && claimed.get() || token_owner.is_none());
    view! {
        <div
            class:tada=is_new_token
            class="flex flex-col gap-2 py-3 px-3 w-full text-xs rounded-lg transition-colors md:px-4 hover:bg-gradient-to-b group bg-neutral-900/90 font-kumbh hover:from-neutral-600 hover:to-neutral-800"
        >
            <div class="flex gap-3 items-stretch">
                <div
                    style="box-shadow: 0px 0px 4px rgba(255, 255, 255, 0.16);"
                    class="overflow-hidden relative w-[7rem] h-[7rem] rounded-[4px] shrink-0"
                >
                    <Show when=move || details.is_nsfw && !show_nsfw.get()>
                        <button
                            on:click=move |_| show_nsfw.set(!show_nsfw.get())
                            class="flex absolute inset-0 justify-center items-center w-full h-full z-[2] backdrop-blur-[4px] bg-black/50 rounded-[4px]"
                        >
                            <div class="flex flex-col gap-1 items-center text-xs">
                                <EyeHiddenIcon classes="w-6 h-6".to_string() />
                                <span class="uppercase">nsfw</span>
                            </div>
                        </button>
                    </Show>
                    <img
                        alt=details.token_name.clone()
                        src=details.logo.clone()
                        class="w-full h-full"
                    />
                </div>
                <div class="flex flex-col justify-between overflow-hidden w-full">
                    <div class="flex flex-col gap-2">
                        <div class="flex gap-4 justify-between items-center w-full text-lg">
                            <span class="font-medium shrink line-clamp-1">{details.name.clone()}</span>
                            <span class="font-bold shrink-0">{symbol}</span>
                        </div>
                        <span class="text-sm line-clamp-2 text-neutral-400">
                            {details.description.clone()}
                        </span>
                    </div>
                    <div class="flex gap-2 justify-between items-center text-sm font-medium group-hover:text-white text-neutral-600">
                        <span class="line-clamp-1">"Created by" {details.user_id.clone()}</span>
                        <span class="shrink-0">{details.formatted_created_at}</span>
                    </div>
                </div>
            </div>

            <div class="flex gap-4 justify-between items-center p-2">
                <ActionButton label="Send".to_string() href=format!("/token/transfer/{root}")>
                    <SendIcon class="w-full h-full" />
                </ActionButton>
                <ActionButton label="Buy/Sell".to_string() href="#".to_string() disabled=true>
                    <Icon attr:class="w-full h-full" icon=ArrowLeftRightIcon />
                </ActionButton>
                <ActionButtonLink disabled=airdrop_disabled on:click=move |_|{airdrop_action.dispatch(());} label="Airdrop".to_string()>
                    <Icon attr:class="h-full w-full" icon=AirdropIcon />
                </ActionButtonLink>
                <ActionButton label="Share".to_string() href="#".to_string()>
                    <Icon
                    attr:class="w-full h-full"
                        icon=ShareIcon
                        on:click=move |_| {
                            pop_up.set(true);
                            share_link.set(share_link_coin.clone())
                        }
                    />
                </ActionButton>
                <ActionButton label="Details".to_string() href=details.link>
                    <Icon attr:class="w-4 h-4" icon=ChevronRightIcon />
                </ActionButton>
            </div>
            <PopupOverlay show=pop_up>
                <ShareContent
                    share_link=format!("{base_url}{}", share_link())
                    message=share_message()
                    show_popup=pop_up
                />
            </PopupOverlay>
            <ShadowOverlay show=airdrop_popup >
                <div class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 max-w-[560px] max-h-[634px] min-w-[343px] min-h-[480px] backdrop-blur-lg rounded-lg">
                    <div class="rounded-lg z-[500]">
                        <AirdropPopup
                            name=details.name.clone()
                            logo=details.logo.clone()
                            buffer_signal
                            claimed
                            airdrop_popup
                        />
                    </div>
                </div>
            </ShadowOverlay>
        </div>
    }.into_any()
}

#[component]
pub fn TokenCardLoadingFeed() -> impl IntoView {
    let is_lg_screen = use_media_query("(min-width: 1024px)");
    let is_md_screen = use_media_query("(min-width: 768px)");

    let num_cards = RwSignal::new(6);

    Effect::new(move |_| {
        num_cards.set(match (is_lg_screen.get(), is_md_screen.get()) {
            (true, _) => 6,
            (_, true) => 4,
            _ => 2,
        });
    });

    move || {
        (0..num_cards())
            .map(|_| view! { <TokenCardLoading /> })
            .collect_view()
    }
}

#[component]
pub fn TokenCardLoading() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 py-3 px-3 w-full rounded-lg md:px-4 group bg-neutral-900/90">
            <div class="flex gap-3">
                <div class="w-[7rem] h-[7rem] bg-loading rounded-[4px] relative shrink-0"></div>

                <div class="w-full bg-loading rounded-[4px]"></div>
            </div>

            <div class="h-[4.125rem] bg-loading rounded-[4px]"></div>
        </div>
    }
}

#[component]
pub fn PageSelector(page: RwSignal<u64>, end_of_list: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="flex gap-1 items-start text-sm font-medium text-gray-400">
            <button
                class="flex justify-center items-center w-8 h-8 rounded-lg bg-neutral-800"
                on:click=move |_| {
                    page.update(|page| *page -= 1);
                    end_of_list.set(false);
                }
                disabled=move || page.get() == 1
            >
                <Icon attr:class="w-4 h-4 rotate-180" icon=ChevronRightIcon />
            </button>
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-white bg-blue-500">
                {page}
            </div>
            <button
                class="flex justify-center items-center w-8 h-8 rounded-lg bg-neutral-800"
                on:click=move |_| {
                    page.update(|page| *page += 1);
                }
                disabled=move || end_of_list.get()
            >
                <Icon attr:class="w-4 h-4" icon=ChevronRightIcon />
            </button>
        </div>
    }
}

#[component]
pub fn ActionButton(
    href: String,
    label: String,
    children: Children,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <a
            aria-disabled=move || disabled().to_string()
            href=href
            class=move || {
                format!(
                    "flex flex-col gap-1 justify-center items-center text-xs transition-colors {}",
                    if !disabled.get() {
                        "group-hover:text-white text-neutral-300"
                    } else {
                        "text-neutral-600 pointer-events-none"
                    },
                )
            }
        >
            <div class="w-[1.125rem] h-[1.125rem] flex items-center justify-center">
                {children()}
            </div>

            <div class="text-[0.625rem] font-medium leading-4">{label}</div>
        </a>
    }
}

#[component]
pub fn ActionButtonLink(
    label: String,
    children: Children,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <button
            disabled=disabled
            class="flex flex-col gap-1 justify-center items-center text-xs transition-colors enabled:group-hover:text-white enabled:text-neutral-300 disabled:group-hover:cursor-default disabled:text-neutral-600"
        >
            <div class="w-[1.125rem] h-[1.125rem] flex items-center justify-center">
                {children()}
            </div>

            <div>{label}</div>
        </button>
    }
}

#[component]
pub fn TelegramIcon(href: String, classes: String) -> impl IntoView {
    view! {
        <a href=href target="_blank">
            <svg class=classes viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect
                    width="40"
                    height="40"
                    rx="20"
                    fill="white"
                    style="fill:white;fill-opacity:1;"
                />
                <path
                    d="M29.6778 11.8012L25.7231 28.1008C25.537 28.8685 24.6763 29.264 23.9706 28.8995L18.9691 26.3173L16.604 30.1867C15.9604 31.2413 14.332 30.7838 14.332 29.5509V25.2395C14.332 24.906 14.4716 24.5881 14.712 24.3555L24.4592 15.0503C24.4514 14.934 24.3273 14.8332 24.2033 14.9185L12.5718 23.014L8.66359 20.9978C7.74858 20.5248 7.78735 19.1988 8.73338 18.7879L28.0029 10.3899C28.9257 9.9867 29.9182 10.8164 29.6778 11.8012Z"
                    fill="black"
                    style="fill:black;fill-opacity:1;"
                />
            </svg>

        </a>
    }
}

#[component]
pub fn XIcon(href: String, classes: String) -> impl IntoView {
    view! {
        <a href=href target="_blank">

            <svg class=classes viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect
                    width="40"
                    height="40"
                    rx="20"
                    fill="white"
                    style="fill:white;fill-opacity:1;"
                />
                <path
                    d="M22.0682 18.3383L30.1527 9.14286H28.2372L21.2143 17.1255L15.6092 9.14286H9.14282L17.6208 21.2151L9.14282 30.8571H11.0583L18.4702 22.4252L24.3908 30.8571H30.8571L22.0682 18.3383ZM19.4438 21.3211L18.5835 20.1182L11.7491 10.5559H14.6917L20.2089 18.2758L21.0656 19.4787L28.2363 29.513H25.2937L19.4438 21.3211Z"
                    fill="black"
                    style="fill:black;fill-opacity:1;"
                />
            </svg>

        </a>
    }
}

#[component]
pub fn InstagramIcon(href: String, classes: String) -> impl IntoView {
    view! {
        <a href=href target="_blank">
            <svg class=classes viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect
                    width="40"
                    height="40"
                    rx="20"
                    fill="white"
                    style="fill:white;fill-opacity:1;"
                />
                <path
                    d="M25.493 9.14286H14.5069C13.0847 9.1442 11.7211 9.70978 10.7154 10.7155C9.70975 11.7211 9.14417 13.0847 9.14282 14.507V25.493C9.14417 26.9153 9.70975 28.2789 10.7154 29.2845C11.7211 30.2902 13.0847 30.8558 14.5069 30.8571H25.493C26.9151 30.8553 28.2784 30.2895 29.2839 29.2839C30.2895 28.2784 30.8552 26.9151 30.8571 25.493V14.507C30.8552 13.0849 30.2895 11.7216 29.2839 10.716C28.2784 9.71049 26.9151 9.14474 25.493 9.14286ZM25.381 15.9964C25.1083 15.9961 24.8417 15.9149 24.6151 15.763C24.3885 15.6112 24.212 15.3956 24.1079 15.1435C24.0038 14.8913 23.9768 14.614 24.0304 14.3465C24.0839 14.079 24.2156 13.8334 24.4087 13.6408C24.6018 13.4481 24.8477 13.3171 25.1153 13.2642C25.3829 13.2113 25.6602 13.239 25.9121 13.3437C26.164 13.4484 26.3792 13.6254 26.5304 13.8524C26.6817 14.0794 26.7623 14.3461 26.7619 14.6189C26.7617 14.8 26.7258 14.9793 26.6563 15.1466C26.5868 15.3138 26.485 15.4658 26.3568 15.5937C26.2285 15.7216 26.0764 15.823 25.909 15.8921C25.7415 15.9612 25.5622 15.9967 25.381 15.9964ZM20 14.8293C21.0226 14.8293 22.0223 15.1325 22.8727 15.7007C23.723 16.2689 24.3857 17.0764 24.7771 18.0213C25.1684 18.9661 25.2708 20.0057 25.0713 21.0088C24.8718 22.0118 24.3793 22.9331 23.6562 23.6562C22.9331 24.3794 22.0117 24.8718 21.0087 25.0714C20.0057 25.2709 18.966 25.1685 18.0212 24.7771C17.0764 24.3858 16.2688 23.723 15.7007 22.8727C15.1325 22.0224 14.8292 21.0227 14.8292 20C14.8312 18.6292 15.3766 17.3152 16.3459 16.3459C17.3152 15.3767 18.6292 14.8313 20 14.8293Z"
                    fill="black"
                    style="fill:black;fill-opacity:1;"
                />
                <path
                    d="M19.9999 23.1112C20.6153 23.1112 21.2168 22.9288 21.7284 22.5869C22.2401 22.245 22.6389 21.7591 22.8743 21.1906C23.1098 20.6221 23.1714 19.9965 23.0514 19.393C22.9313 18.7895 22.635 18.2351 22.1999 17.8C21.7648 17.3649 21.2104 17.0686 20.6069 16.9485C20.0034 16.8285 19.3778 16.8901 18.8093 17.1256C18.2408 17.3611 17.7549 17.7598 17.413 18.2715C17.0711 18.7831 16.8887 19.3846 16.8887 20C16.8896 20.8249 17.2176 21.6157 17.8009 22.199C18.3842 22.7823 19.175 23.1103 19.9999 23.1112Z"
                    fill="black"
                    style="fill:black;fill-opacity:1;"
                />
            </svg>

        </a>
    }
}
