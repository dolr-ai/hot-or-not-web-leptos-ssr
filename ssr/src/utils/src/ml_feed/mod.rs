use candid::Principal;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use yral_canisters_common::utils::posts::PostDetails;

const RECOMMENDATION_SERVICE_URL: &str = "https://recsys-on-premise.fly.dev";

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

// v2 recommend-with-metadata API types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoItemV2 {
    pub video_id: String,
    pub canister_id: String,
    pub post_id: String,
    pub publisher_user_id: String,
    pub num_views_loggedin: u64,
    pub num_views_all: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedResponseV2 {
    pub user_id: String,
    pub videos: Vec<VideoItemV2>,
    pub count: u32,
    pub sources: HashMap<String, u32>,
    pub timestamp: u64,
}

/// Recommendation type for the feed
#[derive(Debug, Clone, Copy, Default)]
pub enum RecType {
    #[default]
    Mixed,
    Popularity,
    Freshness,
}

impl RecType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecType::Mixed => "mixed",
            RecType::Popularity => "popularity",
            RecType::Freshness => "freshness",
        }
    }
}

/// PostItem compatible with existing code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostItem {
    pub video_id: String,
    pub canister_id: Principal,
    pub post_id: String,
    pub publisher_user_id: Principal,
    pub views: u64,
}

impl TryFrom<VideoItemV2> for PostItem {
    type Error = anyhow::Error;

    fn try_from(item: VideoItemV2) -> Result<Self, Self::Error> {
        Ok(Self {
            video_id: item.video_id,
            canister_id: Principal::from_str(&item.canister_id)
                .map_err(|e| anyhow::anyhow!("Invalid canister_id: {}", e))?,
            post_id: item.post_id,
            publisher_user_id: Principal::from_str(&item.publisher_user_id)
                .map_err(|e| anyhow::anyhow!("Invalid publisher_user_id: {}", e))?,
            views: item.num_views_all,
        })
    }
}

pub async fn get_ml_feed_coldstart_clean(
    user_id: Principal,
    num_results: u32,
) -> Result<Vec<PostItem>, anyhow::Error> {
    get_recommendations(user_id, num_results, RecType::Mixed).await
}

pub async fn get_ml_feed_coldstart_nsfw(
    user_id: Principal,
    num_results: u32,
) -> Result<Vec<PostItem>, anyhow::Error> {
    get_recommendations(user_id, num_results, RecType::Mixed).await
}

pub async fn get_ml_feed_clean(
    user_id: Principal,
    num_results: u32,
) -> Result<Vec<PostItem>, anyhow::Error> {
    get_recommendations(user_id, num_results, RecType::Mixed).await
}

pub async fn get_ml_feed_nsfw(
    user_id: Principal,
    num_results: u32,
) -> Result<Vec<PostItem>, anyhow::Error> {
    get_recommendations(user_id, num_results, RecType::Mixed).await
}

/// Core function to fetch recommendations from the new API
pub async fn get_recommendations(
    user_id: Principal,
    count: u32,
    rec_type: RecType,
) -> Result<Vec<PostItem>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();

    let url = format!(
        "{}/v2/recommend-with-metadata/{}",
        RECOMMENDATION_SERVICE_URL,
        user_id.to_text()
    );

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .get(&url)
            .query(&[
                ("count", count.to_string()),
                ("rec_type", rec_type.as_str().to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            leptos::logging::warn!(
                "FEEDISSUE : ML feed attempt {}/{} failed with status: {}",
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

        let response = response.json::<FeedResponseV2>().await?;
        if !response.videos.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed succeeded on attempt {}/{}, got {} videos",
                attempt,
                MAX_RETRIES,
                response.videos.len()
            );

            let posts: Vec<PostItem> = response
                .videos
                .into_iter()
                .filter_map(|v| PostItem::try_from(v).ok())
                .collect();

            return Ok(posts);
        }

        leptos::logging::warn!(
            "FEEDISSUE : ML feed attempt {}/{} returned empty results",
            attempt,
            MAX_RETRIES
        );
    }

    leptos::logging::error!(
        "FEEDISSUE : All {} ML feed attempts returned empty results",
        MAX_RETRIES
    );
    Ok(vec![])
}
