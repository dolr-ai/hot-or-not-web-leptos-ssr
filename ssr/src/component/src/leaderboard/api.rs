use super::types::{LeaderboardResponse, SearchResponse};
use candid::Principal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct UserRankInfo {
    pub rank: u32,
    pub score: f64,
    pub percentile: f64,
    pub reward: Option<u32>,
    pub principal_id: String,
    pub username: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TournamentStatus {
    pub id: String,
    pub metric_type: String,
    pub metric_display_name: String,
    pub status: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LeaderboardRankResponse {
    pub user: UserRankInfo,
    pub tournament: TournamentStatus,
    // We don't need the other fields for now
    // pub surrounding_players: ...,
    // pub total_participants: u32,
}

// Client-side function to fetch rank with tournament status
pub async fn fetch_user_rank_from_api(
    principal: Principal,
) -> Result<Option<(u32, String)>, String> {
    use consts::OFF_CHAIN_AGENT_URL;

    let url = OFF_CHAIN_AGENT_URL
        .join(&format!("api/v1/leaderboard/rank/{principal}"))
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if response.status().is_success() {
        let data: LeaderboardRankResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;
        Ok(Some((data.user.rank, data.tournament.status)))
    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
        // User not in current tournament
        Ok(None)
    } else {
        Ok(None)
    }
}

// Fetch paginated leaderboard data
pub async fn fetch_leaderboard_page(
    start: u32,
    limit: u32,
    user_id: Option<String>,
    sort_order: Option<&str>,
    tournament_id: Option<String>,
) -> Result<LeaderboardResponse, String> {
    use consts::OFF_CHAIN_AGENT_URL;

    let mut url = OFF_CHAIN_AGENT_URL
        .join("api/v1/leaderboard/current")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    // Add query parameters
    url.query_pairs_mut()
        .append_pair("start", &start.to_string())
        .append_pair("limit", &limit.to_string());

    if let Some(id) = user_id {
        url.query_pairs_mut().append_pair("user_id", &id);
    }

    if let Some(order) = sort_order {
        url.query_pairs_mut().append_pair("sort_order", order);
    }

    if let Some(t_id) = tournament_id {
        url.query_pairs_mut().append_pair("tournament_id", &t_id);
    }

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if response.status().is_success() {
        let data: LeaderboardResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;
        Ok(data)
    } else {
        let status = response.status();
        Err(format!("API error: {status}"))
    }
}

// Search users in leaderboard
pub async fn search_users(
    query: String,
    start: u32,
    limit: u32,
    sort_order: Option<&str>,
) -> Result<SearchResponse, String> {
    use consts::OFF_CHAIN_AGENT_URL;

    let mut url = OFF_CHAIN_AGENT_URL
        .join("api/v1/leaderboard/search")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    url.query_pairs_mut()
        .append_pair("q", &query)
        .append_pair("start", &start.to_string())
        .append_pair("limit", &limit.to_string());

    if let Some(order) = sort_order {
        url.query_pairs_mut().append_pair("sort_order", order);
    }

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if response.status().is_success() {
        let data: SearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;
        Ok(data)
    } else {
        let status = response.status();
        Err(format!("Search API error: {status}"))
    }
}
