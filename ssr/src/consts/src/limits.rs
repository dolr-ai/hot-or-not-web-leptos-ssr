use std::ops::Range;

pub const SATS_AIRDROP_LIMIT_RANGE: Range<u64> = 50..100;

pub const NEW_USER_SIGNUP_REWARD: u64 = 1000;
pub const REFERRAL_REWARD: u64 = 10;
pub const MIN_WITHDRAWAL_PER_TXN: u64 = 200;
pub const MAX_WITHDRAWAL_PER_TXN: u64 = 500;

// Hot or not bet limits
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoinState {
    C10,
    C20,
    C50,
    C100,
    C200,
}

pub const MAX_BET_AMOUNT: usize = 20;
pub const BET_COIN_ENABLED_STATES: [CoinState; 2] = [CoinState::C10, CoinState::C20];
pub const DEFAULT_BET_COIN_STATE: CoinState = CoinState::C10;
