use super::api::fetch_leaderboard_page;
use super::provider::LeaderboardError;
use super::types::LeaderboardEntry;
use yral_canisters_common::cursored_data::{CursoredDataProvider, PageEntry};

// Tournament provider that fetches specific tournament data
#[derive(Clone, PartialEq)]
pub struct TournamentLeaderboardProvider {
    pub tournament_id: String,
    pub user_id: Option<String>,
    pub sort_order: String,
    pub start_offset: usize,
}

impl TournamentLeaderboardProvider {
    pub fn new(tournament_id: String, user_id: Option<String>, sort_order: String) -> Self {
        Self {
            tournament_id,
            user_id,
            sort_order,
            start_offset: 0,
        }
    }

    pub fn with_start_offset(mut self, offset: usize) -> Self {
        self.start_offset = offset;
        self
    }
}

impl CursoredDataProvider for TournamentLeaderboardProvider {
    type Data = LeaderboardEntry;
    type Error = LeaderboardError;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        let limit = (end - start).min(50); // Max 50 per request

        // Apply start offset
        let adjusted_start = start + self.start_offset;

        let response = fetch_leaderboard_page(
            adjusted_start as u32,
            limit as u32,
            self.user_id.clone(),
            Some(&self.sort_order),
            Some(self.tournament_id.clone()), // Pass tournament ID
        )
        .await
        .map_err(LeaderboardError)?;

        Ok(PageEntry {
            data: response.data,
            end: !response.cursor_info.has_more,
        })
    }
}
