use candid::Principal;
use serde::Deserialize;
use serde::Serialize;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::post::PostItemV3;

const RECOMMENDATION_SERVICE_URL: &str =
    "https://recsys-on-premise.fly.dev/recommend-with-metadata";

#[derive(Debug, Serialize, Deserialize)]
struct NewVideoItem {
    video_id: String,
    canister_id: String,
    post_id: String,
    publisher_user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NewFeedResponse {
    user_id: String,
    videos: Vec<NewVideoItem>,
    count: u32,
    sources: serde_json::Value,
    timestamp: u64,
}

/// Piece of post details that should be available as quickly as possible to ensure fast loading of the infinite scroller
#[derive(Clone)]
pub struct QuickPostDetails {
    pub video_uid: String,
    pub canister_id: Principal,
    pub publisher_user_id: Principal,
    pub nsfw_probability: f32,
    pub post_id: String,
}

impl From<PostDetails> for QuickPostDetails {
    fn from(value: PostDetails) -> Self {
        Self {
            video_uid: value.uid,
            canister_id: value.canister_id,
            post_id: value.post_id,
            publisher_user_id: value.poster_principal,
            nsfw_probability: value.nsfw_probability,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchHistoryItem {
    pub video_id: String,
    pub last_watched_timestamp: String,
    pub mean_percentage_watched: String,
}

// New v2 REST APIs

pub async fn get_ml_feed_coldstart_clean(
    user_id: Principal,
    num_results: u32,
    _filter_results: Vec<String>,
    _ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();

    let url = format!(
        "{}/{}?count={}&rec_type=mixed",
        RECOMMENDATION_SERVICE_URL,
        user_id.to_text(),
        num_results
    );

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            leptos::logging::warn!(
                "FEEDISSUE : ML feed coldstart clean attempt {}/{} failed with status: {}",
                attempt,
                MAX_RETRIES,
                response.status()
            );
            if attempt == MAX_RETRIES {
                return Err(anyhow::anyhow!(format!(
                    "FEEDISSUE : Error fetching ML feed after {} attempts: {:?}",
                    MAX_RETRIES,
                    response.text().await?
                )));
            }
            continue;
        }

        let response = response.json::<NewFeedResponse>().await?;
        if !response.videos.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed coldstart clean succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            let posts: Vec<PostItemV3> = response
                .videos
                .into_iter()
                .map(|video| PostItemV3 {
                    video_id: video.video_id,
                    canister_id: video.canister_id,
                    post_id: video.post_id,
                    publisher_user_id: video.publisher_user_id,
                    nsfw_probability: 0.0,
                })
                .collect();
            return Ok(posts);
        }

        leptos::logging::warn!(
            "FEEDISSUE : ML feed coldstart clean attempt {}/{} returned empty results",
            attempt,
            MAX_RETRIES
        );
    }

    leptos::logging::error!(
        "FEEDISSUE : All {} ML feed coldstart clean attempts returned empty results",
        MAX_RETRIES
    );
    Ok(vec![])
}

pub async fn get_ml_feed_coldstart_nsfw(
    user_id: Principal,
    num_results: u32,
    _filter_results: Vec<String>,
    _ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();

    let url = format!(
        "{}/{}?count={}&rec_type=mixed",
        RECOMMENDATION_SERVICE_URL,
        user_id.to_text(),
        num_results
    );

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            leptos::logging::warn!(
                "FEEDISSUE : ML feed coldstart nsfw attempt {}/{} failed with status: {}",
                attempt,
                MAX_RETRIES,
                response.status()
            );
            if attempt == MAX_RETRIES {
                return Err(anyhow::anyhow!(format!(
                    "FEEDISSUE : Error fetching ML feed after {} attempts: {:?}",
                    MAX_RETRIES,
                    response.text().await?
                )));
            }
            continue;
        }

        let response = response.json::<NewFeedResponse>().await?;
        if !response.videos.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed coldstart nsfw succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            let posts: Vec<PostItemV3> = response
                .videos
                .into_iter()
                .map(|video| PostItemV3 {
                    video_id: video.video_id,
                    canister_id: video.canister_id,
                    post_id: video.post_id,
                    publisher_user_id: video.publisher_user_id,
                    nsfw_probability: 0.0,
                })
                .collect();
            return Ok(posts);
        }

        leptos::logging::warn!(
            "FEEDISSUE : ML feed coldstart nsfw attempt {}/{} returned empty results",
            attempt,
            MAX_RETRIES
        );
    }

    leptos::logging::error!(
        "FEEDISSUE : All {} ML feed coldstart nsfw attempts returned empty results",
        MAX_RETRIES
    );
    Ok(vec![])
}

pub async fn get_ml_feed_clean(
    user_id: Principal,
    num_results: u32,
    _filter_results: Vec<String>,
    _ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();

    let url = format!(
        "{}/{}?count={}&rec_type=mixed",
        RECOMMENDATION_SERVICE_URL,
        user_id.to_text(),
        num_results
    );

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            leptos::logging::warn!(
                "FEEDISSUE : ML feed clean attempt {}/{} failed with status: {}",
                attempt,
                MAX_RETRIES,
                response.status()
            );
            if attempt == MAX_RETRIES {
                return Err(anyhow::anyhow!(format!(
                    "FEEDISSUE : Error fetching ML feed after {} attempts: {:?}",
                    MAX_RETRIES,
                    response.text().await?
                )));
            }
            continue;
        }

        let response = response.json::<NewFeedResponse>().await?;
        if !response.videos.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed clean succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            let posts: Vec<PostItemV3> = response
                .videos
                .into_iter()
                .map(|video| PostItemV3 {
                    video_id: video.video_id,
                    canister_id: video.canister_id,
                    post_id: video.post_id,
                    publisher_user_id: video.publisher_user_id,
                    nsfw_probability: 0.0,
                })
                .collect();
            return Ok(posts);
        }

        leptos::logging::warn!(
            "FEEDISSUE : ML feed clean attempt {}/{} returned empty results",
            attempt,
            MAX_RETRIES
        );
    }

    leptos::logging::error!(
        "FEEDISSUE : All {} ML feed clean attempts returned empty results",
        MAX_RETRIES
    );
    Ok(vec![])
}

pub async fn get_ml_feed_nsfw(
    user_id: Principal,
    num_results: u32,
    _filter_results: Vec<String>,
    _ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();

    let url = format!(
        "{}/{}?count={}&rec_type=mixed",
        RECOMMENDATION_SERVICE_URL,
        user_id.to_text(),
        num_results
    );

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            leptos::logging::warn!(
                "FEEDISSUE : ML feed nsfw attempt {}/{} failed with status: {}",
                attempt,
                MAX_RETRIES,
                response.status()
            );
            if attempt == MAX_RETRIES {
                return Err(anyhow::anyhow!(format!(
                    "FEEDISSUE : Error fetching ML feed after {} attempts: {:?}",
                    MAX_RETRIES,
                    response.text().await?
                )));
            }
            continue;
        }

        let response = response.json::<NewFeedResponse>().await?;
        if !response.videos.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed nsfw succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            let posts: Vec<PostItemV3> = response
                .videos
                .into_iter()
                .map(|video| PostItemV3 {
                    video_id: video.video_id,
                    canister_id: video.canister_id,
                    post_id: video.post_id,
                    publisher_user_id: video.publisher_user_id,
                    nsfw_probability: 0.0,
                })
                .collect();
            return Ok(posts);
        }

        leptos::logging::warn!(
            "FEEDISSUE : ML feed nsfw attempt {}/{} returned empty results",
            attempt,
            MAX_RETRIES
        );
    }

    leptos::logging::error!(
        "FEEDISSUE : All {} ML feed nsfw attempts returned empty results",
        MAX_RETRIES
    );
    Ok(vec![])
}
