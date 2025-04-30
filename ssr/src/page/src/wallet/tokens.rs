use crate::icpump::{ActionButton, ActionButtonLink};
use crate::wallet::airdrop::AirdropPopup;
use candid::{Nat, Principal};
use component::icons::information_icon::Information;
use component::icons::padlock_icon::{PadlockClose, PadlockOpen};
use component::icons::{
    airdrop_icon::AirdropIcon, arrow_left_right_icon::ArrowLeftRightIcon,
    chevron_right_icon::ChevronRightIcon, send_icon::SendIcon, share_icon::ShareIcon,
};
use component::infinite_scroller::InfiniteScroller;
use component::overlay::PopupOverlay;
use component::overlay::ShadowOverlay;
use component::share_popup::ShareContent;
use component::tooltip::Tooltip;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_navigate;
use state::app_type::AppType;
use state::canisters::authenticated_canisters;
use state::canisters::unauth_canisters;
use utils::event_streaming::events::account_connected_reader;
use utils::event_streaming::events::CentsAdded;
use utils::host::get_host;
use utils::send_wrap;
use utils::token::icpump::IcpumpTokenInfo;
use yral_canisters_common::cursored_data::token_roots::{TokenListResponse, TokenRootList};
use yral_canisters_common::utils::token::balance::TokenBalance;
use yral_canisters_common::utils::token::{RootType, TokenMetadata, TokenOwner};
use yral_canisters_common::Canisters;
use yral_canisters_common::CENT_TOKEN_NAME;
use yral_pump_n_dump_common::WithdrawalState;

use super::ShowLoginSignal;

#[component]
pub fn TokenViewFallback() -> impl IntoView {
    view! {
        <div class="w-full items-center h-16 rounded-xl border-2 border-neutral-700 bg-white/15 animate-pulse"></div>
    }
}

#[component]
pub fn TokenList(user_principal: Principal, user_canister: Principal) -> impl IntoView {
    let app_type: AppType = expect_context();
    let exclude = match app_type {
        AppType::YRAL | AppType::Pumpdump => vec![RootType::COYNS],
        _ => vec![RootType::CENTS],
    };
    view! {
        <div class="flex flex-col w-full gap-2 mb-2 items-center">
           <Suspense>
                {
                    Suspend::new(async move {
                        let viewer_principal = Canisters::from_wire(authenticated_canisters().await.unwrap(), expect_context()).unwrap().user_principal();

                        let provider = TokenRootList {
                            viewer_principal,
                            canisters: unauth_canisters(),
                            user_canister,
                            user_principal,
                            nsfw_detector: IcpumpTokenInfo,
                            exclude,
                        };

                        view! {
                            <InfiniteScroller
                            provider
                            fetch_count=5
                            children=move |TokenListResponse{token_metadata, airdrop_claimed, root}, _ref| {
                                view! {
                                    <WalletCard user_principal token_metadata=token_metadata is_airdrop_claimed=airdrop_claimed _ref=_ref.unwrap_or_default() is_utility_token=matches!(root, RootType::COYNS | RootType::CENTS)/>
                                }
                            }
                        />
                        }
                    })
                }
            </Suspense>
        </div>
    }
}

#[derive(Clone)]
struct WalletCardOptionsContext {
    is_utility_token: bool,
    root: String,
    token_owner: Option<TokenOwner>,
    user_principal: Principal,
}

#[component]
pub fn WalletCard(
    user_principal: Principal,
    token_metadata: TokenMetadata,
    is_airdrop_claimed: bool,
    #[prop(optional)] is_utility_token: bool,
    #[prop(optional)] _ref: NodeRef<html::Div>,
) -> impl IntoView {
    let root: String = token_metadata
        .root
        .map(|r| r.to_text())
        .unwrap_or(token_metadata.name.to_lowercase());

    let is_cents = token_metadata.name == CENT_TOKEN_NAME;

    let share_link = RwSignal::new("".to_string());

    let symbol = token_metadata.symbol.clone();
    let share_message = move || {
        format!(
        "Hey! Check out the token: {} I created on YRAL 👇 {}. I just minted my own token—come see and create yours! 🚀 #YRAL #TokenMinter",
        token_metadata.symbol.clone(),
        share_link.get(),
    )
    };
    let pop_up = RwSignal::new(false);
    let base_url = get_host();

    provide_context(WalletCardOptionsContext {
        is_utility_token,
        root,
        token_owner: token_metadata.token_owner,
        user_principal,
    });

    // let airdrop_popup = RwSignal::new(false);
    // let buffer_signal = RwSignal::new(false);
    // let claimed = RwSignal::new(is_airdrop_claimed);
    let (is_connected, _) = account_connected_reader();
    let show_login = use_context()
        .map(|ShowLoginSignal(show_login)| show_login)
        .unwrap_or_else(|| RwSignal::new(false));
    let nav = use_navigate();
    let withdraw_handle = move |_| {
        if !is_connected() {
            show_login.set(true);
            return;
        }

        nav("/pnd/withdraw", Default::default());
    };

    let airdrop_popup = RwSignal::new(false);
    let buffer_signal = RwSignal::new(false);
    let claimed = RwSignal::new(is_airdrop_claimed);
    let (is_withdrawable, withdraw_message, withdrawable_balance) = token_metadata
        .withdrawable_state
        .as_ref()
        .map(|state| match state {
            WithdrawalState::Value(bal) => (
                true,
                Some("Cents you can withdraw".to_string()),
                Some(TokenBalance::new(bal.clone() * 100usize, 8).humanize_float_truncate_to_dp(2)),
            ),
            WithdrawalState::NeedMoreEarnings(more) => (
                false,
                Some(format!(
                    "Earn {} Cents more to unlock",
                    TokenBalance::new(more.clone() * 100usize, 8).humanize_float_truncate_to_dp(2)
                )),
                None,
            ),
        })
        .unwrap_or_default();
    view! {
        <div node_ref=_ref class="flex flex-col gap-4 bg-neutral-900/90 rounded-lg w-full font-kumbh text-white p-4">
            <div class="flex flex-col gap-4 p-3 rounded-sm bg-neutral-800/70">
                <div class="w-full flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <img
                            src=token_metadata.logo_b64.clone()
                            alt=token_metadata.name.clone()
                            class="w-8 h-8 rounded-full object-cover"
                        />
                        <div class="text-sm font-medium uppercase truncate">{token_metadata.name.clone()}</div>
                    </div>
                    <div class="flex flex-col items-end">
                        {
                            token_metadata.balance.map(|b| view! {
                                <div class="text-lg font-medium">{b.humanize_float_truncate_to_dp(2)}</div>
                            })
                        }
                        <div class="text-xs">{symbol}</div>
                    </div>
                </div>
                {is_cents.then_some(view! {
                    <div class="border-t border-neutral-700 flex flex-col pt-4 gap-2">
                        <div class="flex items-center">
                            <Icon attr:class="text-neutral-300" icon=if is_withdrawable { PadlockOpen } else { PadlockClose } />
                            <span class="text-neutral-400 text-xs mx-2">{withdraw_message}</span>
                            <Tooltip icon=Information title="Withdrawal Tokens" description="Only Cents earned above your airdrop amount can be withdrawn." />
                            <span class="ml-auto">{withdrawable_balance}</span>
                        </div>
                        <button
                            class="rounded-lg px-5 py-2 text-sm text-center font-bold"
                            class=(["pointer-events-none", "text-primary-300", "bg-brand-gradient-disabled"], !is_withdrawable)
                            class=(["text-neutral-50", "bg-brand-gradient"], is_withdrawable)
                            on:click=withdraw_handle
                        >
                            Withdraw
                        </button>
                    </div>

                })}
            </div>

            <WalletCardOptions pop_up=pop_up.write_only() share_link=share_link.write_only() airdrop_popup buffer_signal claimed/>

            <PopupOverlay show=pop_up >
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
                            name=token_metadata.name.clone()
                            logo=token_metadata.logo_b64.clone()
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
fn WalletCardOptions(
    pop_up: WriteSignal<bool>,
    share_link: WriteSignal<String>,
    airdrop_popup: RwSignal<bool>,
    buffer_signal: RwSignal<bool>,
    claimed: RwSignal<bool>,
) -> impl IntoView {
    use_context().map(|WalletCardOptionsContext { is_utility_token, root, token_owner, user_principal, .. }|{
        let share_link_coin = format!("/token/info/{root}/{user_principal}");
        let token_owner_c = token_owner.clone();
        let root_c = root.clone();
        let cans_res = authenticated_canisters();
        let airdrop_action = Action::new(move |&()| {
            let cans_res = cans_res;
            let token_owner_cans_id = token_owner_c.clone().unwrap().canister_id;
            airdrop_popup.set(true);
            let root = Principal::from_text(root_c.clone()).unwrap();

            send_wrap(async move {
                if claimed.get() && !buffer_signal.get() {
                    return Ok(());
                }
                buffer_signal.set(true);
                let cans_wire = cans_res.await?;
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

                if is_utility_token {
                    CentsAdded.send_event("airdrop".to_string(), 100);
                }

                buffer_signal.set(false);
                claimed.set(true);
                Ok::<_, ServerFnError>(())
            })
        });

        let airdrop_disabled = Signal::derive(move || token_owner.is_some() && claimed.get() || token_owner.is_none());
        view! {
            <div class="flex items-center justify-around">
            <ActionButton disabled=is_utility_token href=format!("/token/transfer/{root}") label="Send".to_string()>
                <SendIcon class="h-full w-full" />
            </ActionButton>
            <ActionButton disabled=true href="#".to_string() label="Buy/Sell".to_string()>
                <Icon attr:class="h-6 w-6" icon=ArrowLeftRightIcon />
            </ActionButton>
            <ActionButtonLink disabled=airdrop_disabled on:click=move |_|{airdrop_action.dispatch(());} label="Airdrop".to_string()>
                <Icon attr:class="h-6 w-6" icon=AirdropIcon />
            </ActionButtonLink>

            <ActionButton disabled=is_utility_token href="#".to_string() label="Share".to_string()>
                <Icon attr:class="h-6 w-6" icon=ShareIcon on:click=move |_| {pop_up.set(true); share_link.set(share_link_coin.clone())}/>
            </ActionButton>
            <ActionButton disabled=is_utility_token href=format!("/token/info/{root}/{user_principal}") label="Details".to_string()>
                <Icon attr:class="h-6 w-6" icon=ChevronRightIcon />
            </ActionButton>
        </div>
        }
    })
}
