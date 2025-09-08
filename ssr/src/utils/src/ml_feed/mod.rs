use candid::Principal;
use serde::Deserialize;
use serde::Serialize;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::post::FeedRequestV3;
use yral_types::post::FeedResponseV3;
use yral_types::post::PostItemV3;

const RECOMMENDATION_SERVICE_URL: &str =
    "https://recommendation-service-82502260393.us-central1.run.app/v2/recommendations";

/// Piece of post details that should be available as quickly as possible to ensure fast loading of the infinite scroller
#[derive(Clone)]
pub struct QuickPostDetails {
    pub video_uid: String,
    pub canister_id: Principal,
    pub publisher_user_id: Principal,
    pub nsfw_probability: f32,
    pub post_id: u64,
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
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: false,
        num_results,
        ip_address,
    };

    let cache_url = format!("{RECOMMENDATION_SERVICE_URL}/cache");
    let response = client
        .post(&cache_url)
        .json(&recommendation_request)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(format!(
            "Error fetching ML feed: {:?}",
            response.text().await?
        )));
    }
    let response = response.json::<FeedResponseV3>().await?;
    Ok(response.posts)
}

pub async fn get_ml_feed_coldstart_nsfw(
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: true,
        num_results,
        ip_address,
    };

    let cache_url = format!("{RECOMMENDATION_SERVICE_URL}/cache");
    let response = client
        .post(&cache_url)
        .json(&recommendation_request)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(format!(
            "Error fetching ML feed: {:?}",
            response.text().await?
        )));
    }
    let response = response.json::<FeedResponseV3>().await?;
    Ok(response.posts)
}

pub async fn get_ml_feed_clean(
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: false,
        num_results,
        ip_address,
    };

    let response = client
        .post(RECOMMENDATION_SERVICE_URL)
        .json(&recommendation_request)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(format!(
            "Error fetching ML feed: {:?}",
            response.text().await?
        )));
    }
    let response = response.json::<FeedResponseV3>().await?;
    Ok(response.posts)
}

pub async fn get_ml_feed_nsfw(
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: true,
        num_results,
        ip_address,
    };

    let response = client
        .post(RECOMMENDATION_SERVICE_URL)
        .json(&recommendation_request)
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(format!(
            "Error fetching ML feed: {:?}",
            response.text().await?
        )));
    }
    let response = response.json::<FeedResponseV3>().await?;
    Ok(response.posts)
}
