use candid::Principal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct RankResponse {
    pub rank: u32,
    pub score: f64,
    pub percentile: f64,
    pub potential_reward: Option<f64>,
}

// Client-side function to fetch rank
pub async fn fetch_user_rank_from_api(
    principal: Principal,
) -> Result<Option<u32>, String> {
    #[cfg(feature = "hydrate")]
    {
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
            let data: RankResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            Ok(Some(data.rank))
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            // User not in current tournament
            Ok(None)
        } else {
            Ok(None)
        }
    }
    
    #[cfg(not(feature = "hydrate"))]
    {
        // SSR doesn't make client calls
        let _ = principal;
        Ok(None)
    }
}