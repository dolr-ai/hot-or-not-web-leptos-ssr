use candid::{Nat, Principal};
use leptos::prelude::*;
use leptos_router::params::Params;
use serde::{Deserialize, Serialize};
use yral_pump_n_dump_common::rest::{BalanceInfoResponse, UserBetsResponse};

use consts::PUMP_AND_DUMP_WORKER_URL;

/// utility macro to quickly format cents
#[macro_export]
macro_rules! format_cents {
    ($num:expr) => {
        TokenBalance::new($num, 6).humanize_float_truncate_to_dp(2)
    };
}

/// Convert e8s to cents
/// Backend returns dolr in e8s, and 1dolr = 100cents
pub(super) fn convert_e8s_to_cents(num: Nat) -> u128 {
    (num * 100u64 / 10u64.pow(8))
        .0
        .try_into()
        .expect("cents, scoped at individual player, to be small enough to fit in a u128")
}

/// Estimates the player count based on the count returned by the server
///
/// Logarithimically inflates the count
fn estimate_player_count(num: u64) -> u64 {
    let x = num as f64;
    let res = x + 4.0 + 20.0 * ((x.sqrt() + 2.).log10());
    res.round() as u64
}

// TODO: use leptos::slice to achieve the same effect
/// The data that is required when game is being played by the user
///
/// This data is kept out of GameState so that mutating pumps and dumps doesn't
/// cause the whole game card to rerender
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub(super) struct GameRunningData {
    pub(super) pumps: u64,
    pub(super) dumps: u64,
    pub(super) winning_pot: Option<u64>,
    pub(super) player_count: u64,
}

/// The current state of the game
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) enum GameState {
    Playing,
    ResultDeclared(GameResult),
}

impl GameState {
    /// Get the winnings if the game result is declared to be a win
    pub(super) fn winnings(&self) -> Option<u128> {
        match self {
            GameState::ResultDeclared(GameResult::Win { amount }) => Some(*amount),
            _ => None,
        }
    }

    /// Get the amount the user lost if the game result is declared to be loss
    pub(super) fn lossings(&self) -> Option<u128> {
        match self {
            GameState::ResultDeclared(GameResult::Loss { amount }) => Some(*amount),
            _ => None,
        }
    }

    /// Has the player lost
    pub(super) fn has_lost(&self) -> bool {
        matches!(self, GameState::ResultDeclared(GameResult::Loss { .. }))
    }

    /// Has the player won
    pub(super) fn has_won(&self) -> bool {
        matches!(self, GameState::ResultDeclared(GameResult::Win { .. }))
    }

    /// Is the game running
    pub(super) fn is_running(&self) -> bool {
        matches!(self, GameState::Playing)
    }
}

/// The result of the game, can never be draw
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) enum GameResult {
    Win { amount: u128 },
    Loss { amount: u128 },
}

impl GameRunningData {
    pub(super) fn new(pumps: u64, dumps: u64, player_count: u64, winning_pot: Option<u64>) -> Self {
        Self {
            pumps,
            dumps,
            player_count: estimate_player_count(player_count),
            winning_pot,
        }
    }

    /// Load the game running data from the server
    pub(super) async fn load(
        owner: Principal,
        token_root: Principal,
        user_canister: Principal,
    ) -> Result<Self, String> {
        let bets_url = PUMP_AND_DUMP_WORKER_URL
            .join(&format!("/bets/{owner}/{token_root}/{user_canister}"))
            .expect("url to be valid");

        let player_count_url = PUMP_AND_DUMP_WORKER_URL
            .join(&format!("/player_count/{owner}/{token_root}"))
            .expect("url to be valid");

        let bets: UserBetsResponse = reqwest::get(bets_url)
            .await
            .map_err(|err| format!("Coulnd't load bets: {err}"))?
            .json()
            .await
            .map_err(|err| format!("Couldn't parse bets out of repsonse: {err}"))?;

        let player_count: u64 = reqwest::get(player_count_url)
            .await
            .map_err(|err| format!("Coulnd't load player count: {err}"))?
            .text()
            .await
            .map_err(|err| format!("Couldn't read response for player count: {err}"))?
            .parse()
            .map_err(|err| format!("Couldn't parse player count from response: {err}"))?;

        // Maybe we should also load winning pot as part of game running data
        Ok(Self::new(bets.pumps, bets.dumps, player_count, None))
    }
}

/// The player's overarching stats
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct PlayerData {
    pub(super) games_count: u64,
    pub(super) wallet_balance: u128,
}

impl PlayerData {
    pub(super) fn new(games_count: u64, wallet_balance: u128) -> Self {
        Self {
            games_count,
            wallet_balance,
        }
    }

    /// Load the user's stats from the server
    pub(super) async fn load(user_canister: Principal) -> Result<Self, ServerFnError> {
        let balance_url = PUMP_AND_DUMP_WORKER_URL
            .join(&format!("/balance/{user_canister}"))
            .expect("Url to be valid");
        let games_count_url = PUMP_AND_DUMP_WORKER_URL
            .join(&format!("/game_count/{user_canister}"))
            .expect("Url to be valid");

        let games_count: u64 = reqwest::get(games_count_url).await?.text().await?.parse()?;

        let wallet_balance: BalanceInfoResponse = reqwest::get(balance_url).await?.json().await?;
        let wallet_balance = wallet_balance.balance;

        let wallet_balance = convert_e8s_to_cents(wallet_balance);

        Ok(Self::new(games_count, wallet_balance))
    }
}

/// Query parameters for when user click on the card in profile section
#[derive(Debug, Params, PartialEq, Clone)]
pub(super) struct CardQuery {
    pub(super) root: Principal,
    pub(super) state: String,
    pub(super) amount: Option<u128>,
}

impl CardQuery {
    /// Check whether the query parameters are coherent
    pub(super) fn is_valid(&self) -> bool {
        let Self { state, amount, .. } = self;
        // only win and loss states are allowed currently
        matches!(
            (state.as_str(), amount),
            ("win", &Some(..)) | ("loss", &Some(..)) | ("pending", &None)
        )
    }

    /// Parses out the details necessary for showing game card
    ///
    /// PANICS: when the query is invalid
    pub(super) fn details(&self) -> (Principal, Option<GameResult>) {
        let Self {
            root,
            state,
            amount,
        } = self;
        match state.as_str() {
            "win" => (
                *root,
                Some(GameResult::Win {
                    amount: *amount.as_ref().expect("amount to exist for win state"),
                }),
            ),
            "loss" => (
                *root,
                Some(GameResult::Loss {
                    amount: *amount.as_ref().expect("amount to exist for loss state"),
                }),
            ),
            "pending" => (*root, None),
            _ => unreachable!("Unknown state key"),
        }
    }
}
