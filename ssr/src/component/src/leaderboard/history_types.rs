use super::types::CursorInfo;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TournamentWinner {
    pub principal_id: String,
    pub username: String,
    pub score: f64,
    pub reward: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TournamentHistoryEntry {
    pub id: String,
    pub prize_pool: f64,
    pub prize_token: String,
    pub start_time: i64,
    pub end_time: i64,
    pub status: String,
    pub total_participants: u32,
    pub winner: TournamentWinner,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TournamentHistoryResponse {
    pub tournaments: Vec<TournamentHistoryEntry>,
    pub cursor_info: CursorInfo,
}
