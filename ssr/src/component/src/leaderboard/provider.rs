use yral_canisters_common::cursored_data::{CursoredDataProvider, KeyedData, PageEntry};
use super::types::LeaderboardEntry;
use super::api::{fetch_leaderboard_page, search_users};

// Make LeaderboardEntry implement KeyedData
impl KeyedData for LeaderboardEntry {
    type Key = String;
    
    fn key(&self) -> Self::Key {
        self.principal_id.clone()
    }
}

#[derive(Clone, PartialEq)]
pub struct LeaderboardProvider {
    pub user_id: Option<String>,
    pub sort_order: String,
    pub search_query: Option<String>,
}

impl LeaderboardProvider {
    pub fn new(user_id: Option<String>, sort_order: String) -> Self {
        Self {
            user_id,
            sort_order,
            search_query: None,
        }
    }
    
    pub fn with_search(mut self, query: String) -> Self {
        self.search_query = if query.is_empty() { 
            None 
        } else { 
            Some(query) 
        };
        self
    }
}

#[derive(Debug)]
pub struct LeaderboardError(pub String);

impl std::fmt::Display for LeaderboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LeaderboardError {}

impl CursoredDataProvider for LeaderboardProvider {
    type Data = LeaderboardEntry;
    type Error = LeaderboardError;
    
    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        let limit = (end - start).min(50); // Max 50 per request
        
        let response = if let Some(query) = &self.search_query {
            // Search mode
            search_users(
                query.clone(), 
                start as u32, 
                limit as u32, 
                Some(&self.sort_order)
            )
            .await
            .map_err(|e| LeaderboardError(e))?
            .into()
        } else {
            // Normal leaderboard mode
            fetch_leaderboard_page(
                start as u32, 
                limit as u32, 
                self.user_id.clone(), 
                Some(&self.sort_order),
                None // No tournament_id for current leaderboard
            )
            .await
            .map_err(|e| LeaderboardError(e))?
        };
        
        Ok(PageEntry {
            data: response.data,
            end: !response.cursor_info.has_more,
        })
    }
}

// Helper to convert SearchResponse to LeaderboardResponse-like structure
impl From<super::types::SearchResponse> for super::types::LeaderboardResponse {
    fn from(search: super::types::SearchResponse) -> Self {
        // Create a dummy tournament info for search results
        let dummy_tournament = super::types::TournamentInfo {
            id: String::new(),
            start_time: 0,
            end_time: 0,
            prize_pool: 0.0,
            prize_token: String::new(),
            status: String::new(),
            metric_type: String::new(),
            metric_display_name: String::new(),
            client_timezone: None,
            client_start_time: None,
            client_end_time: None,
        };
        
        super::types::LeaderboardResponse {
            data: search.data,
            cursor_info: search.cursor_info,
            tournament_info: dummy_tournament,
            user_info: None,
        }
    }
}