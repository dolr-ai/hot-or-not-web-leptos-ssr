pub mod api;
pub mod history_api;
pub mod history_card;
pub mod history_types;
pub mod pagination;
pub mod podium;
pub mod provider;
pub mod rank_badge;
pub mod search_bar;
pub mod tournament_completion_popup;
pub mod tournament_header;
pub mod tournament_provider;
pub mod types;

use serde::{Deserialize, Serialize};

pub use rank_badge::GlobalRankBadge;

// Type for rank update counter to provide type safety
#[derive(Clone, Copy, Debug, Default)]
pub struct RankUpdateCounter(pub u32);

// Type for user rank value with tournament status to provide type safety
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserRank {
    pub rank: Option<u32>,
    pub tournament_status: Option<String>,
}
