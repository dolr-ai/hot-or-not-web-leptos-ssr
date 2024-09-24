pub mod icpump;
#[cfg(feature = "ssr")]
mod server_impl;

use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

use candid::{Encode, Nat, Principal};
use ic_agent::AgentError;
use leptos::ServerFnError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    canister::{
        sns_governance::{DissolveState, GetMetadataArg, ListNeurons, Neuron, SnsGovernance},
        sns_ledger::Account as LedgerAccount,
        sns_root::ListSnsCanistersArg,
    },
    state::canisters::{Canisters, CanistersAuthWire},
};
use leptos::{server, server_fn::codec::Cbor};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenBalance {
    pub e8s: Nat,
    decimals: u8,
}

impl TokenBalance {
    pub fn new(e8s: Nat, decimals: u8) -> Self {
        Self { e8s, decimals }
    }

    /// Token Balance but with 8 decimals (default for Cdao)
    pub fn new_cdao(e8s: Nat) -> Self {
        Self::new(e8s, 8u8)
    }

    /// Parse a numeric value
    /// multiplied by 8 decimals (1e8)
    pub fn parse_cdao(token_str: &str) -> Result<Self, rust_decimal::Error> {
        let tokens = (Decimal::from_str(token_str)? * Decimal::new(1e8 as i64, 0)).floor();
        let e8s = Nat::from_str(&tokens.to_string()).unwrap();
        Ok(Self::new_cdao(e8s))
    }

    // Human friendly token amount
    pub fn humanize(&self) -> String {
        (self.e8s.clone() / 10u64.pow(self.decimals as u32))
            .to_string()
            .replace("_", ",")
    }

    // Humanize the amount, but as a float
    pub fn humanize_float(&self) -> String {
        let tokens = Decimal::from_str(&self.e8s.0.to_str_radix(10)).unwrap()
            / Decimal::new(10i64.pow(self.decimals as u32), 0);
        tokens.to_string()
    }

    // Returns number of tokens(not e8s)
    pub fn to_tokens(&self) -> String {
        let tokens = self.e8s.clone() / Nat::from(10u64.pow(self.decimals as u32));
        tokens.0.to_str_radix(10)
    }
}

impl From<TokenBalance> for Nat {
    fn from(value: TokenBalance) -> Nat {
        value.e8s
    }
}

impl Add<Nat> for TokenBalance {
    type Output = Self;

    fn add(self, other: Nat) -> Self {
        Self {
            e8s: self.e8s + other,
            decimals: self.decimals,
        }
    }
}

impl AddAssign<Nat> for TokenBalance {
    fn add_assign(&mut self, rhs: Nat) {
        self.e8s += rhs;
    }
}

impl PartialEq<Nat> for TokenBalance {
    fn eq(&self, other: &Nat) -> bool {
        self.e8s.eq(other)
    }
}

impl PartialOrd<Nat> for TokenBalance {
    fn partial_cmp(&self, other: &Nat) -> Option<Ordering> {
        self.e8s.partial_cmp(other)
    }
}

impl Sub<Nat> for TokenBalance {
    type Output = Self;

    fn sub(self, rhs: Nat) -> Self {
        Self {
            e8s: self.e8s - rhs,
            decimals: self.decimals,
        }
    }
}

impl SubAssign<Nat> for TokenBalance {
    fn sub_assign(&mut self, rhs: Nat) {
        self.e8s -= rhs;
    }
}

impl Sub for TokenBalance {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            e8s: self.e8s - rhs.e8s,
            decimals: self.decimals,
        }
    }
}

impl SubAssign<TokenBalance> for TokenBalance {
    fn sub_assign(&mut self, rhs: TokenBalance) {
        self.e8s -= rhs.e8s;
    }
}

impl PartialEq for TokenBalance {
    fn eq(&self, other: &Self) -> bool {
        self.e8s.eq(&other.e8s)
    }
}

impl PartialOrd for TokenBalance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.e8s.partial_cmp(&other.e8s)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeployedCdaoCanisters {
    pub root: Principal,
    pub swap: Principal,
    pub ledger: Principal,
    pub index: Principal,
    pub governance: Principal,
}

impl From<crate::canister::individual_user_template::DeployedCdaoCanisters>
    for DeployedCdaoCanisters
{
    fn from(value: crate::canister::individual_user_template::DeployedCdaoCanisters) -> Self {
        Self {
            root: value.root,
            swap: value.swap,
            ledger: value.ledger,
            index: value.index,
            governance: value.governance,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenMetadata {
    pub logo_b64: String,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub balance: TokenBalance,
    pub fees: TokenBalance,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenCans {
    pub governance: Principal,
    pub ledger: Principal,
    pub root: Principal,
}

pub async fn token_metadata_by_root<const A: bool>(
    cans: &Canisters<A>,
    user_principal: Principal,
    token_root: Principal,
) -> Result<Option<TokenMetadata>, ServerFnError> {
    // let user_principal = cans
    let root = cans.sns_root(token_root).await;
    let sns_cans = root.list_sns_canisters(ListSnsCanistersArg {}).await?;
    let Some(governance) = sns_cans.governance else {
        return Ok(None);
    };
    let Some(ledger) = sns_cans.ledger else {
        return Ok(None);
    };
    let metadata = get_token_metadata(cans, user_principal, governance, ledger).await?;

    Ok(Some(metadata))
}

pub async fn get_token_metadata<const A: bool>(
    cans: &Canisters<A>,
    user_principal: Principal,
    governance: Principal,
    ledger: Principal,
) -> Result<TokenMetadata, AgentError> {
    let governance = cans.sns_governance(governance).await;
    let metadata = governance.get_metadata(GetMetadataArg {}).await?;

    let ledger = cans.sns_ledger(ledger).await;
    let symbol = ledger.icrc_1_symbol().await?;

    let acc = LedgerAccount {
        owner: user_principal,
        subaccount: None,
    };
    let balance_e8s = ledger.icrc_1_balance_of(acc).await?;
    let fees = ledger.icrc_1_fee().await?;

    Ok(TokenMetadata {
        logo_b64: metadata.logo.unwrap_or_default(),
        name: metadata.name.unwrap_or_default(),
        description: metadata.description.unwrap_or_default(),
        symbol,
        fees: TokenBalance::new_cdao(fees),
        balance: TokenBalance::new_cdao(balance_e8s),
    })
}

#[server(input = Cbor)]
pub async fn claim_tokens_from_first_neuron(
    cans_wire: CanistersAuthWire,
    governance_principal: Principal,
    ledger_principal: Principal,
    raw_neuron: Vec<u8>,
) -> Result<(), ServerFnError> {
    server_impl::claim_tokens_from_first_neuron(
        cans_wire,
        governance_principal,
        ledger_principal,
        raw_neuron,
    )
    .await
}

async fn get_neurons(
    governance: &SnsGovernance<'_>,
    user_principal: Principal,
) -> Result<Vec<Neuron>, ServerFnError> {
    let neurons = governance
        .list_neurons(ListNeurons {
            of_principal: Some(user_principal),
            limit: 10,
            start_page_at: None,
        })
        .await?;

    Ok(neurons.neurons)
}

pub async fn claim_tokens_from_first_neuron_if_required(
    cans_wire: CanistersAuthWire,
    token_root: Principal,
) -> Result<(), ServerFnError> {
    let cans = cans_wire.clone().canisters()?;
    let root_canister = cans.sns_root(token_root).await;
    let token_cans = root_canister
        .list_sns_canisters(ListSnsCanistersArg {})
        .await?;
    let Some(governance) = token_cans.governance else {
        log::warn!("No governance canister found for token. Ignoring...");
        return Ok(());
    };
    let Some(ledger) = token_cans.ledger else {
        log::warn!("No ledger canister found for token. Ignoring...");
        return Ok(());
    };

    let governance_can = cans.sns_governance(governance).await;

    let neurons = get_neurons(&governance_can, cans.user_principal()).await?;
    if neurons.len() < 2 || neurons[1].cached_neuron_stake_e8s == 0 {
        return Ok(());
    }
    let ix = if matches!(
        neurons[1].dissolve_state.as_ref(),
        Some(DissolveState::DissolveDelaySeconds(0))
    ) {
        1
    } else {
        0
    };
    if neurons[ix].cached_neuron_stake_e8s == 0 {
        return Ok(());
    }

    let raw_neurons = Encode!(&neurons[ix]).unwrap();
    claim_tokens_from_first_neuron(cans_wire, governance, ledger, raw_neurons).await
}
