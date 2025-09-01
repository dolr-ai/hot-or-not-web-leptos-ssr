pub mod api;
pub mod rank_badge;

use serde::{Deserialize, Serialize};

pub use rank_badge::{GlobalRankBadge, RankBadge};

// Type for rank update counter to provide type safety
#[derive(Clone, Copy, Debug, Default)]
pub struct RankUpdateCounter(pub u32);

// Type for user rank value to provide type safety
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct UserRank(pub Option<u32>);