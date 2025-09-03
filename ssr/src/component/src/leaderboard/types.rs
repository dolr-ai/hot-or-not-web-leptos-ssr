use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TournamentInfo {
    pub id: String,
    pub start_time: i64,
    pub end_time: i64,
    pub prize_pool: f64,
    pub prize_token: String,
    pub status: String,
    pub metric_type: String,
    pub metric_display_name: String,
    pub client_timezone: Option<String>,
    pub client_start_time: Option<String>,
    pub client_end_time: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LeaderboardEntry {
    pub principal_id: String,
    pub username: String,
    pub rank: u32,
    pub score: f64,
    pub reward: Option<u32>, // Changed from u64 to u32 to match API
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CursorInfo {
    pub start: u32,
    pub limit: u32,
    pub total_count: u32,
    pub has_more: bool,
    pub next_cursor: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LeaderboardResponse {
    pub data: Vec<LeaderboardEntry>,
    pub cursor_info: CursorInfo,
    pub tournament_info: TournamentInfo,
    pub user_info: Option<serde_json::Value>,
    pub upcoming_tournament_info: Option<TournamentInfo>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchResponse {
    pub data: Vec<LeaderboardEntry>,
    pub cursor_info: CursorInfo,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UserInfo {
    pub principal_id: String,
    pub username: String,
    pub rank: u32,
    pub score: f64,
    pub percentile: f32,
    pub reward: Option<u32>, // Changed from u64 to u32 to match API
}
