use candid::Principal;
use consts::ML_FEED_URL;
use serde::Deserialize;
use serde::Serialize;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::post::FeedRequestV2;
use yral_types::post::FeedResponseV2;
use yral_types::post::PostItemV2;

const RECOMMENDATION_SERVICE_URL: &str =
    "https://recommendation-service-749244211103.us-central1.run.app/recommendations";

#[derive(Debug, Serialize, Deserialize)]
pub struct WatchHistoryItem {
    pub video_id: String,
    pub last_watched_timestamp: String,
    pub mean_percentage_watched: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub user_id: String,
    pub exclude_items: Vec<String>, // IDs of videos to exclude from recommendations
    pub nsfw_label: bool,           // Whether to include NSFW content in recommendations
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub publisher_user_id: String,
    pub canister_id: String,
    pub post_id: u64,
    pub video_id: String,
    pub nsfw_probability: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub recommendations: Vec<Recommendation>,
}

impl From<RecommendationResponse> for FeedResponseV2 {
    fn from(response: RecommendationResponse) -> Self {
        FeedResponseV2 {
            posts: response
                .recommendations
                .into_iter()
                .map(|rec| PostItemV2 {
                    publisher_user_id: rec.publisher_user_id,
                    post_id: rec.post_id,
                    canister_id: rec.canister_id,
                    video_id: rec.video_id,
                    is_nsfw: rec.nsfw_probability > 0.4,
                })
                .collect(),
        }
    }
}

// New v2 REST APIs

pub async fn get_ml_feed_coldstart_clean(
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItemV2>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v3/feed/coldstart/clean").unwrap();

    let req = FeedRequestV2 {
        user_id,
        filter_results: post_details_to_post_item(filter_results),
        num_results,
    };

    let response = client.post(ml_feed_url).json(&req).send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(format!(
            "Error fetching ML feed: {:?}",
            response.text().await?
        )));
    }
    let response = response.json::<FeedResponseV2>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_coldstart_nsfw(
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItemV2>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v3/feed/coldstart/nsfw").unwrap();

    let req = FeedRequestV2 {
        user_id,
        filter_results: post_details_to_post_item(filter_results),
        num_results,
    };

    let response = client.post(ml_feed_url).json(&req).send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(format!(
            "Error fetching ML feed: {:?}",
            response.text().await?
        )));
    }
    let response = response.json::<FeedResponseV2>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_clean(
    user_id: Principal,
    _num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItemV2>, anyhow::Error> {
    let client = reqwest::Client::new();
    let recommendation_request = RecommendationRequest {
        user_id: user_id.to_string(),
        exclude_items: post_details_to_video_ids(filter_results),
        nsfw_label: false,
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
    let response = response.json::<RecommendationResponse>().await?;
    let response: FeedResponseV2 = response.into();
    Ok(response.posts)
}

pub async fn get_ml_feed_nsfw(
    user_id: Principal,
    _num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItemV2>, anyhow::Error> {
    let client = reqwest::Client::new();
    let recommendation_request = RecommendationRequest {
        user_id: user_id.to_string(),
        exclude_items: post_details_to_video_ids(filter_results),
        nsfw_label: true,
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
    let response = response.json::<RecommendationResponse>().await?;
    let response: FeedResponseV2 = response.into();
    Ok(response.posts)
}

pub fn post_details_to_post_item(post_details: Vec<PostDetails>) -> Vec<PostItemV2> {
    post_details
        .into_iter()
        .map(|post_detail| PostItemV2 {
            publisher_user_id: post_detail.poster_principal.to_text(),
            post_id: post_detail.post_id,
            canister_id: post_detail.canister_id.to_text(),
            video_id: post_detail.uid,
            is_nsfw: post_detail.is_nsfw,
        })
        .collect()
}

pub fn post_details_to_video_ids(post_details: Vec<PostDetails>) -> Vec<String> {
    post_details
        .into_iter()
        .map(|post_detail| post_detail.uid)
        .collect()
}
