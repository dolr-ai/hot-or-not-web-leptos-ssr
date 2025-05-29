//! The wallet is currently optimized by doing the following:
//! - Have a static list of tokens to show
//! - Avoid loading any bits of information that is not presented on the screen
//! - Display information like `name`, `logo`, etc, is also kept static
//! - Each individual piece of dynamic information, like `balance`, is kept in
//!   its own [`leptos::prelude::Resource`]
//!
//! However, in future it may be required that we add a dynamic number
//! of tokens. In which case, to avoid regression, define a new type that
//! encapsulates both statically loaded tokens and the dynamic ones:
//! ```rs
//! enum Token {
//!     Static(TokenType),
//!     Dynamic(...),
//! }
//! ```
//! Which renders to [`FastWalletCard`].
//! Then seed the list with static items before loading the dynamic tokens.
//!
//! This will ensure we keep the near instant loading time while also fetching items dynmically.

use crate::icpump::{ActionButton, ActionButtonLink};
use crate::wallet::airdrop::AirdropPopup;
use candid::{Nat, Principal};
use component::icons::information_icon::Information;
use component::icons::padlock_icon::{PadlockClose, PadlockOpen};
use component::icons::{
    airdrop_icon::AirdropIcon, arrow_left_right_icon::ArrowLeftRightIcon,
    chevron_right_icon::ChevronRightIcon, send_icon::SendIcon, share_icon::ShareIcon,
};
use component::overlay::PopupOverlay;
use component::overlay::ShadowOverlay;
use component::share_popup::ShareContent;
use component::skeleton::Skeleton;
use component::tooltip::Tooltip;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_navigate;
use state::canisters::{auth_state, unauth_canisters};
use utils::event_streaming::events::CentsAdded;
use utils::host::get_host;
use utils::send_wrap;
use yral_canisters_common::utils::token::balance::TokenBalance;
use yral_canisters_common::utils::token::{
    load_cents_balance, load_sats_balance, TokenMetadata, TokenOwner,
};
use yral_canisters_common::{Canisters, CENT_TOKEN_NAME};
use yral_canisters_common::{SATS_TOKEN_NAME, SATS_TOKEN_SYMBOL};
use yral_pump_n_dump_common::WithdrawalState;

use super::ShowLoginSignal;

#[component]
pub fn TokenViewFallback() -> impl IntoView {
    view! {
        <div class="w-full items-center h-16 rounded-xl border-2 border-neutral-700 bg-white/15 animate-pulse"></div>
    }
}

/// Different strategies for loading balances of tokens as [`yral_canisters_common::utils::token::balance::TokenBalance`]
enum BalanceFetcherType {
    /// Load icrc1 tokens like dolr, ckbtc, etc.
    ///
    /// `decimals` must be known, which allows for performance optimizations
    Icrc1 { ledger: Principal, decimals: u8 },
    /// Load Sats balance
    Sats,
    /// Load cents balance
    Cents,
}

impl BalanceFetcherType {
    /// Fetch the balance based on the selected strategy
    ///
    /// Both `user_principal` and `user_canister` must be provided by the
    /// caller, which allows for perfomance optimizations
    async fn fetch(
        &self,
        cans: Canisters<false>,
        user_canister: Principal,
        user_principal: Principal,
    ) -> Result<TokenBalance, ServerFnError> {
        let res = match self {
            BalanceFetcherType::Icrc1 { ledger, decimals } => cans
                .icrc1_balance_of(user_principal, *ledger)
                .await
                .map(|b| TokenBalance::new(b, *decimals))?,
            BalanceFetcherType::Sats => load_sats_balance(user_principal)
                .await
                .map(|info| TokenBalance::new(info.balance.into(), 0))?,
            BalanceFetcherType::Cents => load_cents_balance(user_canister)
                .await
                .map(|info| TokenBalance::new(info.balance, 6))?,
        };

        Ok(res)
    }
}

/// Different strategies for loading withdrawal state of tokens
enum WithdrawalStateFetcherType {
    /// Load withdrawal state for sats
    Sats,
    /// Load withdrawal state for cents
    Cents,
    /// Simply return `Ok(None)`, used for tokens which can't be withdrawn
    Noop,
}

impl WithdrawalStateFetcherType {
    /// Fetch the withdrawal state based on the selected strategy
    ///
    /// Both `user_principal` and `user_canister` must be provided by the
    /// caller, which allows for perfomance optimizations
    async fn fetch(
        &self,
        user_canister: Principal,
        user_principal: Principal,
    ) -> Result<Option<WithdrawalState>, ServerFnError> {
        let res = match self {
            Self::Sats => load_sats_balance(user_principal)
                .await
                .map(|info| Some(WithdrawalState::Value(info.balance.into())))?,
            Self::Cents => load_cents_balance(user_canister).await.map(|info| {
                if info.withdrawable == 0usize {
                    Some(WithdrawalState::NeedMoreEarnings(
                        (info.net_airdrop_reward - info.balance) + 1e6 as usize,
                    ))
                } else {
                    Some(WithdrawalState::Value(info.withdrawable))
                }
            })?,
            Self::Noop => None,
        };

        Ok(res)
    }
}

/// Different tokens shown statically on the wallet page
///
/// Can be used to select balance and withdrawal state fetching strategy.
#[derive(Debug, Clone, Copy)]
enum TokenType {
    Sats,
    Btc,
    Cents,
    Dolr,
    Usdc,
}

impl From<TokenType> for WithdrawalStateFetcherType {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Sats => Self::Sats,
            TokenType::Cents => Self::Cents,
            _ => Self::Noop,
        }
    }
}

/// Display information for each token is loaded statically as a performance
/// optmization
impl From<TokenType> for TokenDisplayInfo {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Sats => Self {
                name: SATS_TOKEN_NAME.into(),
                symbol: SATS_TOKEN_SYMBOL.into(),
                logo: "/img/hotornot/sats.svg".into(),
                root: None,
            },
            TokenType::Btc => Self {
                name: "Bitcoin".into(),
                symbol: "BTC".into(),
                logo: "/img/hotornot/bitcoin.svg".into(),
                root: None,
            },
            TokenType::Cents => Self {
                name: CENT_TOKEN_NAME.into(),
                symbol: CENT_TOKEN_NAME.into(),
                logo: "/img/pumpdump/cents.webp".into(),
                root: None,
            },
            TokenType::Dolr => Self {
                name: "DOLR AI".into(),
                symbol: "DOLR".into(),
                logo: "/img/common/dolr.png".into(),
                // root: 67bll-riaaa-aaaaq-aaauq-cai
                root: Some(Principal::from_slice(&[0, 0, 0, 0, 2, 0, 0, 41, 1, 1])),
            },
            TokenType::Usdc => Self {
                name: "USDC".into(),
                symbol: "USDC".into(),
                logo: "/img/common/usdc.svg".into(),
                root: None,
            },
        }
    }
}

impl From<TokenType> for BalanceFetcherType {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Sats => Self::Sats,
            TokenType::Btc => Self::Icrc1 {
                // ledger: mxzaz-hqaaa-aaaar-qaada-cai
                ledger: Principal::from_slice(&[0, 0, 0, 0, 2, 48, 0, 6, 1, 1]),
                decimals: 8,
            },
            TokenType::Cents => Self::Cents,
            TokenType::Dolr => Self::Icrc1 {
                // ledger: 6rdgd-kyaaa-aaaaq-aaavq-cai
                ledger: Principal::from_slice(&[0, 0, 0, 0, 2, 0, 0, 43, 1, 1]),
                decimals: 8,
            },
            TokenType::Usdc => Self::Icrc1 {
                // ledger: xevnm-gaaaa-aaaar-qafnq-cai
                ledger: Principal::from_slice(&[0, 0, 0, 0, 2, 48, 1, 91, 1, 1]),
                decimals: 6,
            },
        }
    }
}

impl TokenType {
    /// Whether the token is maintained artifically by our platform, unlike
    /// icrc1/2 tokens. For example, `Sats` and `Cents`
    fn is_utility_token(&self) -> bool {
        matches!(self, Self::Sats | Self::Cents)
    }
}

#[component]
pub fn TokenList(user_principal: Principal, user_canister: Principal) -> impl IntoView {
    let balance = |token_type: TokenType| {
        OnceResource::new(async move {
            let fetcher: BalanceFetcherType = token_type.into();
            send_wrap(fetcher.fetch(unauth_canisters(), user_canister, user_principal)).await
        })
    };

    let withdrawal_state = |token_type: TokenType| {
        OnceResource::new(async move {
            let fetcher: WithdrawalStateFetcherType = token_type.into();
            send_wrap(fetcher.fetch(user_canister, user_principal)).await
        })
    };

    let tokens = [
        TokenType::Sats,
        TokenType::Btc,
        TokenType::Cents,
        TokenType::Dolr,
        TokenType::Usdc,
    ];

    view! {
        <div class="flex flex-col w-full gap-2 mb-2 items-center pb-10">
            {tokens.into_iter().map(|token_type| {
                let display_info: TokenDisplayInfo = token_type.into();
                let balance = balance(token_type);
                let withdrawal_state = withdrawal_state(token_type);
                let is_utility_token = token_type.is_utility_token();

                view! {
                    <FastWalletCard user_principal display_info balance withdrawal_state is_utility_token />
                }
            }).collect_view()}
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

enum WithdrawDetails {
    CanWithdraw {
        /// most sensibly formatted amount
        amount: String,
        // an indicator message
        message: String,
    },
    CannotWithdraw {
        /// A reason or a suggestion message
        message: String,
    },
}

struct WithdrawSats;
struct WithdrawCents;

trait WithdrawImpl {
    fn details(&self, state: WithdrawalState) -> WithdrawDetails;

    /// the url to redirect to when user wishes to withdraw
    fn withdraw_url(&self) -> String;

    fn withdraw_cta(&self) -> String;
}

// TODO: use enum_dispatch instead
// when i try adding enum_dispatch, the linker kills itself with a SIGBUS
type Withdrawer = Box<dyn WithdrawImpl>;

impl WithdrawImpl for WithdrawCents {
    fn details(&self, state: WithdrawalState) -> WithdrawDetails {
        match state {
            WithdrawalState::Value(bal) => WithdrawDetails::CanWithdraw {
                amount: TokenBalance::new(bal * 100usize, 8).humanize_float_truncate_to_dp(2),
                message: "Cents you can withdraw".to_string(),
            },
            WithdrawalState::NeedMoreEarnings(more) => WithdrawDetails::CannotWithdraw {
                message: format!(
                    "Earn {} Cents more to unlock",
                    TokenBalance::new(more * 100usize, 8).humanize_float_truncate_to_dp(2)
                ),
            },
        }
    }

    fn withdraw_url(&self) -> String {
        "/pnd/withdraw".into()
    }

    fn withdraw_cta(&self) -> String {
        "Withdraw".into()
    }
}

impl WithdrawImpl for WithdrawSats {
    fn details(&self, state: WithdrawalState) -> WithdrawDetails {
        match state {
            WithdrawalState::Value(bal) => WithdrawDetails::CanWithdraw {
                amount: TokenBalance::new(bal, 0).humanize_float_truncate_to_dp(0),
                message: "Sats you can withdraw".to_string(),
            },
            WithdrawalState::NeedMoreEarnings(more) => WithdrawDetails::CannotWithdraw {
                message: format!(
                    "Earn {} Sats more to unlock",
                    TokenBalance::new(more, 0).humanize_float_truncate_to_dp(0)
                ),
            },
        }
    }

    fn withdraw_url(&self) -> String {
        "/hot-or-not/withdraw".into()
    }

    fn withdraw_cta(&self) -> String {
        "Withdraw to BTC".into()
    }
}

#[derive(Debug, Clone)]
pub struct TokenDisplayInfo {
    pub name: String,
    pub symbol: String,
    pub logo: String,
    // Token root canister
    pub root: Option<Principal>,
}

#[component]
pub fn WithdrawSection(
    withdrawal_state: WithdrawalState,
    #[prop(into)] token_name: String,
) -> impl IntoView {
    let withdrawer = match token_name.as_str() {
        s if s == SATS_TOKEN_NAME => Box::new(WithdrawSats) as Withdrawer,
        s if s == CENT_TOKEN_NAME => Box::new(WithdrawCents),
        _ => unimplemented!("Withdrawing is not implemented for a token"),
    };

    let withdraw_url = withdrawer.withdraw_url();
    let is_connected = auth_state().is_logged_in_with_oauth();
    let show_login = use_context()
        .map(|ShowLoginSignal(show_login)| show_login)
        .unwrap_or_else(|| RwSignal::new(false));
    let nav = use_navigate();
    let withdraw_handle = move |_| {
        if !is_connected() {
            show_login.set(true);
            return;
        }

        nav(&withdraw_url, Default::default());
    };

    let (is_withdrawable, withdraw_message, withdrawable_balance) =
        match withdrawer.details(withdrawal_state.clone()) {
            WithdrawDetails::CanWithdraw { amount, message } => (true, Some(message), Some(amount)),
            WithdrawDetails::CannotWithdraw { message } => (false, Some(message), None),
        };
    let withdraw_cta = withdrawer.withdraw_cta();
    let is_cents = token_name == CENT_TOKEN_NAME;
    view! {
        <div class="border-t border-neutral-700 flex flex-col pt-4 gap-2">
            {is_cents.then_some(view! {
                <div class="flex items-center">
                    <Icon attr:class="text-neutral-300" icon=if is_withdrawable { PadlockOpen } else { PadlockClose } />
                    <span class="text-neutral-400 text-xs mx-2">{withdraw_message}</span>
                    <Tooltip icon=Information title="Withdrawal Tokens" description="Only Cents earned above your airdrop amount can be withdrawn." />
                    <span class="ml-auto">{withdrawable_balance}</span>
                </div>
            })}
            <button
                class="rounded-lg px-5 py-2 text-sm text-center font-bold"
                class=(["pointer-events-none", "text-primary-300", "bg-brand-gradient-disabled"], !is_withdrawable)
                class=(["text-neutral-50", "bg-brand-gradient"], is_withdrawable)
                on:click=withdraw_handle
            >
                {withdraw_cta}
            </button>
        </div>
    }
}

// avoid redirecting in case of error, because that will
// render the whole wallet useless even if only a single system
// is down
#[component]
pub fn FastWalletCard(
    user_principal: Principal,
    display_info: TokenDisplayInfo,
    balance: OnceResource<Result<TokenBalance, ServerFnError>>,
    withdrawal_state: OnceResource<Result<Option<WithdrawalState>, ServerFnError>>,
    #[prop(optional)] is_utility_token: bool,
) -> impl IntoView {
    let TokenDisplayInfo {
        name,
        symbol,
        logo,
        root,
    } = display_info;

    let root: String = root
        .map(|r| r.to_text())
        .unwrap_or_else(|| symbol.to_lowercase());

    // TODO: this pattern is not good. will improve this during refactor phase
    let share_link = RwSignal::new("".to_string());

    let share_message = {
        let symbol = symbol.clone();
        move || {
            format!(
            "Hey! Check out the token: {} I created on YRAL 👇 {}. I just minted my own token—come see and create yours! 🚀 #YRAL #TokenMinter",
            symbol.clone(),
            share_link.get(),
        )
        }
    };
    let pop_up = RwSignal::new(false);
    let base_url = get_host();

    provide_context(WalletCardOptionsContext {
        is_utility_token,
        root,
        // with icpump gone, there shouldn't be any token owners. but lets keep
        // it just in case; and pray the compiler optimizes things away
        token_owner: None,
        user_principal,
    });
    let airdrop_popup = RwSignal::new(false);
    let buffer_signal = RwSignal::new(false);
    let claimed = RwSignal::new(true);
    let name_c = StoredValue::new(name.clone());

    view! {
        <div class="flex flex-col gap-4 bg-neutral-900/90 rounded-lg w-full font-kumbh text-white p-4">
            <div class="flex flex-col gap-4 p-3 rounded-sm bg-neutral-800/70">
                <div class="w-full flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <img
                            src=logo.clone()
                            alt=name.clone()
                            class="w-8 h-8 rounded-full object-cover"
                        />
                        <div class="text-sm font-medium uppercase truncate">{name.clone()}</div>
                    </div>
                    <div class="flex flex-col items-end">
                        <Suspense
                            fallback=move || view! {
                                <Skeleton class="h-3 w-10 rounded-sm text-neutral-600 [--shimmer:#27272A]" />
                            }
                        >
                            {move || Suspend::new(async move {
                                // show error text if balance fails to load for whatever reason
                                // error logs are captured by sentry
                                let bal = balance.await.inspect_err(|err| {log::error!("balance loading error: {err:?}");}).ok();
                                let bal = bal.map(|b| b.humanize_float_truncate_to_dp(8));
                                let err = bal.is_none();
                                let text = bal.unwrap_or_else(|| "err".into());
                                view! {
                                    <div class="text-lg font-medium" class=("text-red-500", err)>{text}</div>
                                }
                            })}
                        </Suspense>
                        <div class="text-xs">{symbol}</div>
                    </div>
                </div>
                <Suspense>
                {move || Suspend::new(async move {
                    // withdraw section wont show in case of error
                    // error logs are captured by sentry
                    let withdrawal_state = withdrawal_state.await.inspect_err(|err| log::error!("withdrawal state loading error: {err:?}")).ok().flatten();
                    let withdrawal_state = withdrawal_state?;
                    Some(view! {
                        <WithdrawSection withdrawal_state token_name=name_c.get_value() />
                    })
                })}
                </Suspense>
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
                            name=name.clone()
                            logo=logo.clone()
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
    let show_withdraw_button = token_metadata.withdrawable_state.is_some();
    let withdrawer = show_withdraw_button.then(|| match token_metadata.name.as_str() {
        s if s == SATS_TOKEN_NAME => Box::new(WithdrawSats) as Withdrawer,
        s if s == CENT_TOKEN_NAME => Box::new(WithdrawCents),
        _ => unimplemented!("Withdrawing is not implemented for a token"),
    });

    let withdraw_url = withdrawer.as_ref().map(|w| w.withdraw_url());

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
    let is_connected = auth_state().is_logged_in_with_oauth();
    let show_login = use_context()
        .map(|ShowLoginSignal(show_login)| show_login)
        .unwrap_or_else(|| RwSignal::new(false));
    let nav = use_navigate();
    let withdraw_handle = move |_| {
        let Some(ref withdraw_url) = withdraw_url else {
            return;
        };
        if !is_connected() {
            show_login.set(true);
            return;
        }

        nav(withdraw_url, Default::default());
    };

    let airdrop_popup = RwSignal::new(false);
    let buffer_signal = RwSignal::new(false);
    let claimed = RwSignal::new(is_airdrop_claimed);
    let (is_withdrawable, withdraw_message, withdrawable_balance) =
        match (token_metadata.withdrawable_state, withdrawer.as_ref()) {
            (Some(ref state), Some(w)) => match w.details(state.clone()) {
                WithdrawDetails::CanWithdraw { amount, message } => {
                    (true, Some(message), Some(amount))
                }
                WithdrawDetails::CannotWithdraw { message } => (false, Some(message), None),
            },
            _ => Default::default(),
        };
    let withdraw_cta = withdrawer.as_ref().map(|w| w.withdraw_cta());

    // overrides
    let name = match token_metadata.name.to_lowercase().as_str() {
        "btc" => "Bitcoin".to_string(),

        _ => token_metadata.name.to_owned(),
    };

    view! {
        <div node_ref=_ref class="flex flex-col gap-4 bg-neutral-900/90 rounded-lg w-full font-kumbh text-white p-4">
            <div class="flex flex-col gap-4 p-3 rounded-sm bg-neutral-800/70">
                <div class="w-full flex items-center justify-between">
                    <div class="flex items-center gap-2">
                        <img
                            src=token_metadata.logo_b64.clone()
                            alt=name.clone()
                            class="w-8 h-8 rounded-full object-cover"
                        />
                        <div class="text-sm font-medium uppercase truncate">{name.clone()}</div>
                    </div>
                    <div class="flex flex-col items-end">
                        {
                            token_metadata.balance.map(|b| view! {
                                <div class="text-lg font-medium">{b.humanize_float_truncate_to_dp(8)}</div>
                            })
                        }
                        <div class="text-xs">{symbol}</div>
                    </div>
                </div>
                {show_withdraw_button.then_some(view! {
                    <div class="border-t border-neutral-700 flex flex-col pt-4 gap-2">
                        {is_cents.then_some(view! {
                            <div class="flex items-center">
                                <Icon attr:class="text-neutral-300" icon=if is_withdrawable { PadlockOpen } else { PadlockClose } />
                                <span class="text-neutral-400 text-xs mx-2">{withdraw_message}</span>
                                <Tooltip icon=Information title="Withdrawal Tokens" description="Only Cents earned above your airdrop amount can be withdrawn." />
                                <span class="ml-auto">{withdrawable_balance}</span>
                            </div>
                        })}
                        <button
                            class="rounded-lg px-5 py-2 text-sm text-center font-bold"
                            class=(["pointer-events-none", "text-primary-300", "bg-brand-gradient-disabled"], !is_withdrawable)
                            class=(["text-neutral-50", "bg-brand-gradient"], is_withdrawable)
                            on:click=withdraw_handle
                        >
                            {withdraw_cta}
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
    let WalletCardOptionsContext {
        is_utility_token,
        root,
        token_owner,
        user_principal,
        ..
    } = use_context()?;

    let share_link_coin = format!("/token/info/{root}/{user_principal}");
    let token_owner_c = token_owner.clone();
    let root_c = root.clone();

    let auth = auth_state();
    let base = unauth_canisters();
    let airdrop_action = Action::new_local(move |&()| {
        let token_owner_cans_id = token_owner_c.clone().unwrap().canister_id;
        airdrop_popup.set(true);
        let root = Principal::from_text(root_c.clone()).unwrap();
        let base = base.clone();

        async move {
            if claimed.get() && !buffer_signal.get() {
                return Ok(());
            }
            buffer_signal.set(true);
            let cans = auth.auth_cans(base).await?;
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
                CentsAdded.send_event(auth.event_ctx(), "airdrop".to_string(), 100);
            }

            buffer_signal.set(false);
            claimed.set(true);
            Ok::<_, ServerFnError>(())
        }
    });

    let airdrop_disabled =
        Signal::derive(move || token_owner.is_some() && claimed.get() || token_owner.is_none());

    Some(view! {
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
    })
}
