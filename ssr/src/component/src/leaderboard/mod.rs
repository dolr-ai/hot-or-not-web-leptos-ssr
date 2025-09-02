pub mod api;
pub mod history_api;
pub mod history_card;
pub mod history_types;
pub mod pagination;
pub mod provider;
pub mod rank_badge;
pub mod search_bar;
pub mod table;
pub mod tournament_header;
pub mod types;

use serde::{Deserialize, Serialize};

pub use rank_badge::{GlobalRankBadge, RankBadge};

// Type for rank update counter to provide type safety
#[derive(Clone, Copy, Debug, Default)]
pub struct RankUpdateCounter(pub u32);

// Type for user rank value to provide type safety
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct UserRank(pub Option<u32>);