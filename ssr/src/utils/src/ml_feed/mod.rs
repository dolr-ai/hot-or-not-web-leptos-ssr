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
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: false,
        num_results,
        ip_address,
    };

    let cache_url = format!("{RECOMMENDATION_SERVICE_URL}/cache");

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .post(&cache_url)
            .json(&recommendation_request)
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

        let response = response.json::<FeedResponseV3>().await?;
        if !response.posts.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed coldstart clean succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            return Ok(response.posts);
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
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: true,
        num_results,
        ip_address,
    };

    let cache_url = format!("{RECOMMENDATION_SERVICE_URL}/cache");

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .post(&cache_url)
            .json(&recommendation_request)
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

        let response = response.json::<FeedResponseV3>().await?;
        if !response.posts.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed coldstart nsfw succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            return Ok(response.posts);
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
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: false,
        num_results,
        ip_address,
    };

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .post(RECOMMENDATION_SERVICE_URL)
            .json(&recommendation_request)
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

        let response = response.json::<FeedResponseV3>().await?;
        if !response.posts.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed clean succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            return Ok(response.posts);
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
    filter_results: Vec<String>,
    ip_address: Option<String>,
) -> Result<Vec<PostItemV3>, anyhow::Error> {
    const MAX_RETRIES: usize = 5;
    let client = reqwest::Client::new();
    let recommendation_request = FeedRequestV3 {
        user_id,
        exclude_items: filter_results,
        nsfw_label: true,
        num_results,
        ip_address,
    };

    for attempt in 1..=MAX_RETRIES {
        let response = client
            .post(RECOMMENDATION_SERVICE_URL)
            .json(&recommendation_request)
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

        let response = response.json::<FeedResponseV3>().await?;
        if !response.posts.is_empty() {
            leptos::logging::log!(
                "FEEDISSUE : ML feed nsfw succeeded on attempt {}/{}",
                attempt,
                MAX_RETRIES
            );
            return Ok(response.posts);
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
