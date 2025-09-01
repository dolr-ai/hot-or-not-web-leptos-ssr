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
pub struct LeaderboardRankResponse {
    pub user: UserRankInfo,
    // We don't need the other fields for now
    // pub tournament: ...,
    // pub surrounding_players: ...,
    // pub total_participants: u32,
}

// Client-side function to fetch rank
pub async fn fetch_user_rank_from_api(principal: Principal) -> Result<Option<u32>, String> {
    use consts::OFF_CHAIN_AGENT_URL;

    let url = OFF_CHAIN_AGENT_URL
        .join(&format!("api/v1/leaderboard/rank/{}", principal))
        .map_err(|e| format!("Failed to build URL: {}", e))?;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status().is_success() {
        let data: LeaderboardRankResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(Some(data.user.rank))
    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
        // User not in current tournament
        Ok(None)
    } else {
        Ok(None)
    }
}
