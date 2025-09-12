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
//! This will ensure we keep the near instant loading time while also fetching items dynamically.

use std::time::Duration;

use crate::wallet::airdrop::dolr_airdrop::{claim_dolr_airdrop, is_user_eligible_for_dolr_airdrop};
use crate::wallet::airdrop::{
    sats_airdrop::claim_sats_airdrop, AirdropClaimState, AirdropStatus, SatsAirdropPopup,
    StatefulAirdropPopup,
};
use candid::Principal;
use component::action_btn::{ActionButton, ActionButtonLink};
use component::icons::information_icon::Information;
use component::icons::padlock_icon::{PadlockClose, PadlockOpen};
use component::icons::{
    airdrop_icon::AirdropIcon, arrow_left_right_icon::ArrowLeftRightIcon,
    chevron_right_icon::ChevronRightIcon, send_icon::SendIcon, share_icon::ShareIcon,
};
use component::overlay::PopupOverlay;
use component::share_popup::ShareContent;
use component::skeleton::Skeleton;
use component::tooltip::{Tooltip, TooltipBottomRight};
use consts::{
    CKBTC_LEDGER_CANISTER, DOLR_AI_LEDGER_CANISTER, DOLR_AI_ROOT_CANISTER, USDC_LEDGER_CANISTER,
};
use hon_worker_common::{sign_claim_request, ClaimRequest, WithdrawalState};
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_navigate;
use leptos_use::{use_interval, UseIntervalReturn};
use state::canisters::{auth_state, unauth_canisters};
use utils::host::get_host;
use utils::mixpanel::mixpanel_events::*;
use utils::send_wrap;
use utils::time::to_hh_mm_ss;
use yral_canisters_common::utils::token::balance::TokenBalance;
use yral_canisters_common::utils::token::{load_cents_balance, load_sats_balance};
use yral_canisters_common::{Canisters, CENT_TOKEN_NAME};
use yral_canisters_common::{SATS_TOKEN_NAME, SATS_TOKEN_SYMBOL};

use super::airdrop::sats_airdrop::is_user_eligible_for_sats_airdrop;
use super::ShowLoginSignal;

#[component]
pub fn TokenViewFallback() -> impl IntoView {
    view! {
        <div class="items-center w-full h-16 rounded-xl border-2 animate-pulse border-neutral-700 bg-white/15"></div>
    }
}

#[allow(unused)]
enum AirdropStatusFetcherType {
    Sats,
    Dolr,
    Yral,
    MockAvailable,
    MockWaiting,
    NonAirdropable,
}

impl AirdropStatusFetcherType {
    async fn fetch(
        &self,
        user_canister: Principal,
        user_principal: Principal,
    ) -> Result<Option<AirdropStatus>, ServerFnError> {
        let res = match self {
            Self::Sats | Self::Yral => {
                let eligible =
                    is_user_eligible_for_sats_airdrop(user_canister, user_principal).await?;
                Some(if eligible {
                    AirdropStatus::Available
                } else {
                    AirdropStatus::Claimed
                })
            }
            Self::Dolr => {
                Some(is_user_eligible_for_dolr_airdrop(user_canister, user_principal).await?)
            }
            Self::MockAvailable => {
                utils::time::sleep(Duration::from_millis(100)).await;
                Some(AirdropStatus::Available)
            }
            Self::MockWaiting => {
                utils::time::sleep(Duration::from_millis(100)).await;
                Some(AirdropStatus::WaitFor(Duration::from_secs(24 * 3600)))
            }
            Self::NonAirdropable => None,
        };

        Ok(res)
    }
}

/// Different strategies for loading balances of tokens as [`yral_canisters_common::utils::token::balance::TokenBalance`]
pub enum BalanceFetcherType {
    Icrc1 { ledger: Principal, decimals: u8 },
    Sats,
    Cents,
    Yral,
}

impl BalanceFetcherType {
    // Both `user_principal` and `user_canister` must be provided by the
    // caller, which allows for perfomance optimizations
    pub async fn fetch(
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
            BalanceFetcherType::Sats | BalanceFetcherType::Yral => {
                load_sats_balance(user_principal)
                    .await
                    .map(|info| TokenBalance::new(info.balance.into(), 0))?
            }
            BalanceFetcherType::Cents => load_cents_balance(user_canister)
                .await
                .map(|info| TokenBalance::new(info.balance, 6))?,
        };

        Ok(res)
    }
}

/// Different strategies for loading withdrawal state of tokens
enum WithdrawalStateFetcherType {
    Sats,
    Cents,
    /// Simply return `Ok(None)`, used for tokens which can't be withdrawn
    Noop,
}

impl WithdrawalStateFetcherType {
    // Both `user_principal` and `user_canister` must be provided by the
    // caller, which allows for performance optimizations
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

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Sats,
    Btc,
    Cents,
    Dolr,
    Usdc,
    Yral,
}

impl From<TokenType> for AirdropStatusFetcherType {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Sats => Self::Sats,
            TokenType::Dolr => Self::Dolr,
            TokenType::Yral => Self::Yral,
            _ => Self::NonAirdropable,
        }
    }
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

impl From<TokenType> for StakeType {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Sats => Self::Sats,
            TokenType::Cents => Self::Cents,
            TokenType::Btc => Self::Btc,
            TokenType::Usdc => Self::Usdc,
            TokenType::Dolr => Self::DolrAi,
            TokenType::Yral => Self::Yral,
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
                token_root_canister: None,
            },
            TokenType::Btc => Self {
                name: "Bitcoin".into(),
                symbol: "BTC".into(),
                logo: "/img/hotornot/bitcoin.svg".into(),
                token_root_canister: None,
            },
            TokenType::Cents => Self {
                name: CENT_TOKEN_NAME.into(),
                symbol: CENT_TOKEN_NAME.into(),
                logo: "/img/yral/cents.webp".into(),
                token_root_canister: None,
            },
            TokenType::Dolr => Self {
                name: "DOLR AI".into(),
                symbol: "DOLR".into(),
                logo: "/img/common/dolr.svg".into(),
                token_root_canister: Some(DOLR_AI_ROOT_CANISTER.parse().unwrap()),
            },
            TokenType::Usdc => Self {
                name: "USDC".into(),
                symbol: "USDC".into(),
                logo: "/img/common/usdc.svg".into(),
                token_root_canister: None,
            },
            TokenType::Yral => Self {
                name: "Yral".into(),
                symbol: "YRAL".into(),
                logo: "/img/yral/yral-token.webp".into(),
                token_root_canister: None,
            },
        }
    }
}

impl From<TokenType> for BalanceFetcherType {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Sats => Self::Sats,
            TokenType::Btc => Self::Icrc1 {
                ledger: CKBTC_LEDGER_CANISTER.parse().unwrap(),
                decimals: 8,
            },
            TokenType::Cents => Self::Cents,
            TokenType::Dolr => Self::Icrc1 {
                ledger: DOLR_AI_LEDGER_CANISTER.parse().unwrap(),
                decimals: 8,
            },
            TokenType::Usdc => Self::Icrc1 {
                ledger: USDC_LEDGER_CANISTER.parse().unwrap(),
                decimals: 6,
            },
            TokenType::Yral => Self::Yral,
        }
    }
}

impl TokenType {
    /// Whether the token is maintained artifically by our platform, unlike
    /// icrc1/2 tokens. For example, `Sats` and `Cents`
    fn is_utility_token(&self) -> bool {
        matches!(self, Self::Sats | Self::Cents | Self::Yral)
    }
}

#[component]
pub fn TokenList(user_principal: Principal, user_canister: Principal) -> impl IntoView {
    let balance = |token_type: TokenType| {
        Resource::new(
            || (),
            move |_| async move {
                let fetcher: BalanceFetcherType = token_type.into();
                send_wrap(fetcher.fetch(unauth_canisters(), user_canister, user_principal)).await
            },
        )
    };

    let withdrawal_state = |token_type: TokenType| {
        OnceResource::new(async move {
            let fetcher: WithdrawalStateFetcherType = token_type.into();
            send_wrap(fetcher.fetch(user_canister, user_principal)).await
        })
    };

    let airdrop_status = |token_type: TokenType| {
        Resource::new(
            || (),
            move |_| async move {
                let fetcher: AirdropStatusFetcherType = token_type.into();
                send_wrap(fetcher.fetch(user_canister, user_principal)).await
            },
        )
    };

    let tokens = [
        TokenType::Yral,
        // TokenType::Sats, // TODO: enable once SATS are added back
        TokenType::Btc,
        TokenType::Dolr,
        TokenType::Usdc,
    ];

    view! {
        <div class="flex flex-col gap-2 items-center pb-10 mb-2 w-full">
            {tokens
                .into_iter()
                .map(|token_type| {
                    let display_info: TokenDisplayInfo = token_type.into();
                    let display_info = display_info.clone();
                    let balance = balance(token_type);
                    let withdrawal_state = withdrawal_state(token_type);
                    let is_utility_token = token_type.is_utility_token();
                    let airdrop_status = airdrop_status(token_type);

                    view! {
                        <FastWalletCard
                            user_canister
                            user_principal
                            display_info
                            balance
                            withdrawal_state
                            is_utility_token
                            airdrop_status
                            token_type
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}

#[derive(Clone)]
struct WalletCardOptionsContext {
    is_utility_token: bool,
    root: String,
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

trait AirdroppableImpl {
    async fn claim_airdrop(&self, auth: Canisters<true>) -> Result<u64, ServerFnError>;

    fn show_info(&self, _status: AirdropStatus) -> bool {
        false
    }

    fn eligility_info(&self, _status: AirdropStatus) -> Option<String> {
        None
    }

    fn available_message(&self, _status: AirdropStatus) -> Option<String> {
        None
    }
}

#[derive(Clone)]
enum Airdropper {
    #[allow(unused)]
    MockAirdropDolr(MockAirdropDolr),
    AirdropDolr(AirdropDolr),
    AirdropSats(AirdropSats),
}

// enum_dispatch doesn't work with traits with `async fn` so we doing it by hand
// https://gitlab.com/antonok/enum_dispatch/-/issues/75
impl AirdroppableImpl for Airdropper {
    async fn claim_airdrop(&self, auth: Canisters<true>) -> Result<u64, ServerFnError> {
        match self {
            Airdropper::MockAirdropDolr(mock_airdrop_dolr) => {
                mock_airdrop_dolr.claim_airdrop(auth).await
            }
            Airdropper::AirdropSats(airdrop_sats) => airdrop_sats.claim_airdrop(auth).await,
            Airdropper::AirdropDolr(airdrop_dolr) => airdrop_dolr.claim_airdrop(auth).await,
        }
    }

    fn show_info(&self, status: AirdropStatus) -> bool {
        match self {
            Airdropper::MockAirdropDolr(mock_airdrop_dolr) => mock_airdrop_dolr.show_info(status),
            Airdropper::AirdropSats(airdrop_sats) => airdrop_sats.show_info(status),
            Airdropper::AirdropDolr(airdrop_dolr) => airdrop_dolr.show_info(status),
        }
    }

    fn eligility_info(&self, status: AirdropStatus) -> Option<String> {
        match self {
            Airdropper::MockAirdropDolr(mock_airdrop_dolr) => {
                mock_airdrop_dolr.eligility_info(status)
            }
            Airdropper::AirdropSats(airdrop_sats) => airdrop_sats.eligility_info(status),
            Airdropper::AirdropDolr(airdrop_dolr) => airdrop_dolr.eligility_info(status),
        }
    }

    fn available_message(&self, status: AirdropStatus) -> Option<String> {
        match self {
            Airdropper::MockAirdropDolr(mock_airdrop_dolr) => {
                mock_airdrop_dolr.available_message(status)
            }
            Airdropper::AirdropSats(airdrop_sats) => airdrop_sats.available_message(status),
            Airdropper::AirdropDolr(airdrop_dolr) => airdrop_dolr.available_message(status),
        }
    }
}

#[derive(Clone)]
struct MockAirdropDolr;

impl AirdroppableImpl for MockAirdropDolr {
    async fn claim_airdrop(&self, _auth: Canisters<true>) -> Result<u64, ServerFnError> {
        utils::time::sleep(Duration::from_secs(2)).await;

        Ok(100)
    }

    fn show_info(&self, _status: AirdropStatus) -> bool {
        true
    }

    fn eligility_info(&self, _status: AirdropStatus) -> Option<String> {
        Some("Claims are limited to once every 24 hours.".to_string())
    }

    fn available_message(&self, _status: AirdropStatus) -> Option<String> {
        Some("Tap on “airdrop” to claim free tokens.".to_string())
    }
}

#[derive(Clone)]
struct AirdropDolr;

impl AirdroppableImpl for AirdropDolr {
    async fn claim_airdrop(&self, auth: Canisters<true>) -> Result<u64, ServerFnError> {
        let user_canister = auth.user_canister();
        let user_principal = auth.user_principal();
        claim_dolr_airdrop(user_canister, user_principal).await
    }

    fn show_info(&self, _status: AirdropStatus) -> bool {
        true
    }

    fn eligility_info(&self, _status: AirdropStatus) -> Option<String> {
        Some("Claims are limited to once every 24 hours.".to_string())
    }

    fn available_message(&self, _status: AirdropStatus) -> Option<String> {
        Some("Tap on “airdrop” to claim free tokens.".to_string())
    }
}

#[derive(Clone)]
struct AirdropSats;

impl AirdroppableImpl for AirdropSats {
    async fn claim_airdrop(&self, cans: Canisters<true>) -> Result<u64, ServerFnError> {
        let request = ClaimRequest {
            user_principal: cans.user_principal(),
        };
        let signature = sign_claim_request(cans.identity(), request.clone()).unwrap();

        claim_sats_airdrop(cans.user_canister(), request, signature).await
    }
}

impl Airdropper {
    fn choose(name: &str) -> Option<Self> {
        match name {
            "DOLR AI" => Some(Airdropper::AirdropDolr(AirdropDolr)),
            "Yral" => Some(Airdropper::AirdropSats(AirdropSats)),
            s if s == SATS_TOKEN_NAME => Some(Airdropper::AirdropSats(AirdropSats)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenDisplayInfo {
    pub name: String,
    pub symbol: String,
    pub logo: String,
    pub token_root_canister: Option<Principal>,
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
    let auth_state = auth_state();
    let token_name_analytics = token_name.clone();
    let withdraw_handle = move |_| {
        if !is_connected() {
            show_login.set(true);
            return;
        }
        let global = MixpanelGlobalProps::from_ev_ctx(auth_state.event_ctx());
        if let Some(global) = global {
            let token_clicked = match token_name_analytics.as_str() {
                s if s == SATS_TOKEN_NAME => StakeType::Sats,
                s if s == CENT_TOKEN_NAME => StakeType::Cents,
                _ => unimplemented!("Withdrawing is not implemented for a token"),
            };
            MixPanelEvent::track_withdraw_tokens_clicked(global, token_clicked);
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
        <div class="flex flex-col gap-2 pt-4 border-t border-neutral-700">
            {is_cents
                .then_some(
                    view! {
                        <div class="flex items-center">
                            <Icon
                                attr:class="text-neutral-300"
                                icon=if is_withdrawable { PadlockOpen } else { PadlockClose }
                            />
                            <span class="mx-2 text-xs text-neutral-400">{withdraw_message}</span>
                            <Tooltip
                                icon=Information
                                title="Withdrawal Tokens"
                                description="Only Cents earned above your airdrop amount can be withdrawn."
                            />
                            <span class="ml-auto">{withdrawable_balance}</span>
                        </div>
                    },
                )}
            <button
                class="py-2 px-5 text-sm font-bold text-center rounded-lg"
                class=(
                    ["pointer-events-none", "text-primary-300", "bg-brand-gradient-disabled"],
                    !is_withdrawable,
                )
                class=(["text-neutral-50", "bg-brand-gradient"], is_withdrawable)
                on:click=withdraw_handle
            >
                {withdraw_cta}
            </button>
        </div>
    }
}

#[component]
fn AirdropInfoSection(
    airdrop_status: AirdropStatus,
    #[prop(into)] token_name: String,
) -> impl IntoView {
    let airdropper: Airdropper = Airdropper::choose(&token_name)
        .expect("airdrop status returned for a non airdroppable token");

    if !airdropper.show_info(airdrop_status) {
        return None;
    }

    if matches!(airdrop_status, AirdropStatus::Claimed) {
        return None;
    }

    let UseIntervalReturn { counter, .. } = use_interval(1000);
    let maybe_wait_for = match airdrop_status {
        AirdropStatus::WaitFor(duration) => Some(duration),
        _ => None,
    };

    let timer = move || {
        let counter = counter.get();
        let counter = web_time::Duration::from_secs(counter);

        maybe_wait_for
            .map(|wait_for| wait_for - counter)
            .map(to_hh_mm_ss)
    };

    let available_message = airdropper.available_message(airdrop_status)?;
    let tooltip_info = airdropper.eligility_info(airdrop_status);
    Some(view! {
        <div class="flex flex-col gap-2 pt-4 border-t border-neutral-700">
            <div class="flex justify-between items-center">
                {match airdrop_status {
                    AirdropStatus::Available => {
                        view! {
                            <div class="flex items-center">
                                <Icon attr:class="text-neutral-300" icon=PadlockOpen />
                                <span class="mx-2 text-xs text-neutral-400">
                                    {available_message}
                                </span>
                            </div>
                        }
                            .into_any()
                    }
                    AirdropStatus::WaitFor(..) => {
                        view! {
                            <div class="flex gap-1.5 items-center py-1.5 px-2 rounded-full bg-neutral-900">
                                <span class="text-xs text-400">Next Airdrop In:</span>
                                <span class="text-xs font-semibold text-center">{timer}</span>
                            </div>
                        }
                            .into_any()
                    }
                    _ => ().into_any(),
                }}
                {tooltip_info
                    .map(|tooltip_info| {
                        view! {
                            <TooltipBottomRight
                                icon=Information
                                title="Airdrop Eligibility"
                                description=tooltip_info
                            />
                        }
                    })}
            </div>
        </div>
    })
}

// avoid redirecting in case of error, because that will
// render the whole wallet useless even if only a single system
// is down
#[component]
pub fn FastWalletCard(
    user_principal: Principal,
    user_canister: Principal,
    display_info: TokenDisplayInfo,
    balance: Resource<Result<TokenBalance, ServerFnError>>,
    withdrawal_state: OnceResource<Result<Option<WithdrawalState>, ServerFnError>>,
    airdrop_status: Resource<Result<Option<AirdropStatus>, ServerFnError>>,
    token_type: TokenType,
    #[prop(optional)] is_utility_token: bool,
) -> impl IntoView {
    let _ = user_canister;

    let TokenDisplayInfo {
        name,
        symbol,
        logo,
        token_root_canister,
    } = display_info.clone();

    let root: String = token_root_canister
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
    let name_c = StoredValue::new(name.clone());

    provide_context(WalletCardOptionsContext {
        is_utility_token,
        root,
        user_principal,
    });

    let display_info = display_info.clone();
    let airdropper: Option<Airdropper> = Airdropper::choose(&display_info.name);

    // airdrop popup state
    let show_airdrop_popup = RwSignal::new(false);
    let airdrop_amount_claimed: RwSignal<u64> = RwSignal::new(0);
    let error_claiming_airdrop = RwSignal::new(false);

    // fetch airdrop claim info
    let is_airdrop_claimed = RwSignal::new(true);
    let airdropper_c = airdropper.clone();
    let airdropper_c2 = airdropper_c.clone();

    Effect::new(move || {
        let is_airdrop_available =
            airdrop_status.map(|value| matches!(value, Ok(Some(AirdropStatus::Available))));

        if let Some(true) = is_airdrop_available {
            is_airdrop_claimed.set(false);
        }
    });

    // any variant works as default
    let claim_state = RwSignal::new(AirdropClaimState::Claiming);
    let airdrop_popup = RwSignal::new(false);

    let auth = auth_state();
    let show_login = use_context()
        .map(|ShowLoginSignal(show_login)| show_login)
        .unwrap_or_else(|| RwSignal::new(false));
    // action to claim airdrop
    let claim_airdrop = Action::new_unsync(move |&is_connected: &bool| {
        let airdrop_amount_claimed = airdrop_amount_claimed;
        let error_claiming_airdrop = error_claiming_airdrop;
        let airdropper = airdropper_c2.clone();
        let token_type: StakeType = token_type.into();
        async move {
            if !is_connected {
                show_login.set(true);
                return Err(ServerFnError::new("login required"));
            }

            let cans = auth.auth_cans().await?;
            let global = MixpanelGlobalProps::try_get(&cans.clone(), is_connected);
            let global_dispatched = MixpanelGlobalProps::try_get(&cans.clone(), is_connected);
            MixPanelEvent::track_claim_airdrop_clicked(
                global,
                token_type.clone(),
                "wallet".to_string(),
            );
            error_claiming_airdrop.set(false);
            show_airdrop_popup.set(true);
            match airdropper.as_ref().unwrap().claim_airdrop(cans).await {
                Ok(amount) => {
                    airdrop_amount_claimed.set(amount);
                    MixPanelEvent::track_airdrop_claimed(
                        global_dispatched,
                        token_type,
                        true,
                        amount,
                        "wallet".to_string(),
                    );
                    is_airdrop_claimed.set(true);
                    error_claiming_airdrop.set(false);
                    balance.refetch();
                    airdrop_status.refetch();
                    Ok(amount)
                }
                Err(err) => {
                    log::error!("error claiming airdrop");
                    MixPanelEvent::track_airdrop_claimed(
                        global_dispatched,
                        token_type,
                        false,
                        0,
                        "wallet".to_string(),
                    );
                    error_claiming_airdrop.set(true);
                    Err(err)
                }
            }
        }
    });

    let pending = claim_airdrop.pending();
    let value = claim_airdrop.value();
    Effect::watch(
        move || (pending.get(), value.get()),
        move |(pending, value), _, _| {
            log::info!("pending: {pending} and value: {value:?}");
            if name_c.get_value() == SATS_TOKEN_NAME {
                log::info!("ignoring");
                return;
            }

            if *pending {
                airdrop_popup.set(true);
                claim_state.set(AirdropClaimState::Claiming);
            }

            if let Some(res) = value {
                let new_state = match res {
                    Ok(amount) => AirdropClaimState::Claimed(*amount),
                    Err(_) => AirdropClaimState::Failed,
                };

                claim_state.set(new_state);
            }
        },
        false,
    );

    view! {
        <div class="flex flex-col gap-4 p-4 w-full text-white rounded-lg bg-neutral-900/90 font-kumbh">
            <div class="flex flex-col gap-4 p-3 rounded-sm bg-neutral-800/70">
                <div class="flex justify-between items-center w-full">
                    <div class="flex gap-2 items-center">
                        <img
                            src=logo.clone()
                            alt=name.clone()
                            class="object-cover w-8 h-8 rounded-full"
                        />
                        <div class="text-sm font-medium uppercase truncate">{name.clone()}</div>
                    </div>
                    <div class="flex flex-col items-end">
                        <Suspense fallback=move || {
                            view! {
                                <Skeleton class="w-10 h-3 rounded-sm text-neutral-600 [--shimmer:#27272A]" />
                            }
                        }>
                            {move || Suspend::new(async move {
                                let bal = balance
                                    .await
                                    .inspect_err(|err| {
                                        log::error!("balance loading error: {err:?}");
                                    })
                                    .ok();
                                let bal = bal.map(|b| b.humanize_float_truncate_to_dp(8));
                                let err = bal.is_none();
                                let text = bal.unwrap_or_else(|| "err".into());
                                view! {
                                    // show error text if balance fails to load for whatever reason
                                    // error logs are captured by sentry
                                    <div class="text-lg font-medium" class=("text-red-500", err)>
                                        {text}
                                    </div>
                                }
                            })}
                        </Suspense>
                        <div class="text-xs">{symbol}</div>
                    </div>
                </div>
                <Suspense>
                    {move || Suspend::new(async move {
                        let withdrawal_state = withdrawal_state
                            .await
                            .inspect_err(|err| {
                                log::error!("withdrawal state loading error: {err:?}")
                            })
                            .ok()
                            .flatten();
                        let withdrawal_state = withdrawal_state?;
                        Some(
                            view! {
                                // withdraw section wont show in case of error
                                // error logs are captured by sentry

                                <WithdrawSection withdrawal_state token_name=name_c.get_value() />
                            },
                        )
                    })}
                </Suspense>
                <Suspense>
                    {move || Suspend::new(async move {
                        let airdrop_status = airdrop_status
                            .await
                            .inspect_err(|err| {
                                log::error!("airdrop status loading failed: {err:?}");
                            })
                            .ok()
                            .flatten();
                        let airdrop_status = airdrop_status?;
                        Some(
                            view! {
                                <AirdropInfoSection airdrop_status token_name=name_c.get_value() />
                            },
                        )
                    })}
                </Suspense>
            </div>

            <WalletCardOptions
                pop_up=pop_up.write_only()
                share_link=share_link.write_only()
                airdrop_claimed=is_airdrop_claimed
                claim_airdrop
            />

            <PopupOverlay show=pop_up>
                <ShareContent
                    share_link=format!("{base_url}{}", share_link())
                    message=share_message()
                    show_popup=pop_up
                />
            </PopupOverlay>

            {(name_c.get_value() == SATS_TOKEN_NAME)
                .then_some(
                    view! {
                        <SatsAirdropPopup
                            show=show_airdrop_popup
                            amount_claimed=airdrop_amount_claimed.read_only()
                            claimed=is_airdrop_claimed.read_only()
                            error=error_claiming_airdrop.read_only()
                            try_again=claim_airdrop
                        />
                    },
                )}

            <StatefulAirdropPopup
                name=display_info.symbol.clone()
                logo=logo
                claim_state=claim_state.read_only()
                airdrop_popup
            />
        </div>
    }
}

#[component]
fn WalletCardOptions(
    pop_up: WriteSignal<bool>,
    share_link: WriteSignal<String>,
    airdrop_claimed: RwSignal<bool>,
    claim_airdrop: Action<bool, Result<u64, ServerFnError>>,
) -> impl IntoView {
    let WalletCardOptionsContext {
        is_utility_token,
        root,
        user_principal,
        ..
    } = use_context()?;
    let is_connected = auth_state().is_logged_in_with_oauth();

    let share_link_coin = format!("/token/info/{root}/{user_principal}");

    Some(view! {
        <div class="flex justify-around items-center">
            <ActionButtonLink
                disabled=is_utility_token
                href=format!("/token/transfer/{root}")
                label="Send".to_string()
            >
                <SendIcon class="w-full h-full" />
            </ActionButtonLink>
            <ActionButtonLink disabled=true href="#".to_string() label="Buy/Sell".to_string()>
                <Icon attr:class="h-6 w-6" icon=ArrowLeftRightIcon />
            </ActionButtonLink>
            <ActionButton
                disabled=airdrop_claimed
                on:click=move |_| {
                    claim_airdrop.dispatch(is_connected.get());
                }
                label="Airdrop".to_string()
            >
                <Icon attr:class="h-6 w-6" icon=AirdropIcon />
            </ActionButton>
            <ActionButtonLink
                disabled=is_utility_token
                href="#".to_string()
                label="Share".to_string()
            >
                <Icon
                    attr:class="h-6 w-6"
                    icon=ShareIcon
                    on:click=move |_| {
                        pop_up.set(true);
                        share_link.set(share_link_coin.clone())
                    }
                />
            </ActionButtonLink>
            <ActionButtonLink
                disabled=is_utility_token
                href=format!("/token/info/{root}/{user_principal}")
                label="Details".to_string()
            >
                <Icon attr:class="h-6 w-6" icon=ChevronRightIcon />
            </ActionButtonLink>
        </div>
    })
}
