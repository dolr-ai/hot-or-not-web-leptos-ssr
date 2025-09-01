pub mod api;
pub mod rank_badge;

pub use rank_badge::RankBadge;

// Type for rank update counter to provide type safety
#[derive(Clone, Copy, Debug, Default)]
pub struct RankUpdateCounter(pub u32);