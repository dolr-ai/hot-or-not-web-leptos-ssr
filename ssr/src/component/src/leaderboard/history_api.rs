use super::history_types::TournamentHistoryResponse;

// Fetch tournament history
pub async fn fetch_tournament_history(limit: u32) -> Result<TournamentHistoryResponse, String> {
    use consts::OFF_CHAIN_AGENT_URL;

    let mut url = OFF_CHAIN_AGENT_URL
        .join("api/v1/leaderboard/history")
        .map_err(|e| format!("Failed to build URL: {e}"))?;

    // Add query parameters
    url.query_pairs_mut()
        .append_pair("limit", &limit.to_string());

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if response.status().is_success() {
        let data: TournamentHistoryResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;
        Ok(data)
    } else {
        let status = response.status();
        Err(format!("API error: {status}"))
    }
}
