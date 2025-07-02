use candid::Principal;
use consts::ML_FEED_URL;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::post::FeedRequest;
use yral_types::post::FeedRequestV2;
use yral_types::post::FeedResponse;
use yral_types::post::FeedResponseV2;
use yral_types::post::PostItem;
use yral_types::post::PostItemV2;

// New v2 REST APIs

pub async fn get_ml_feed_coldstart_clean(
    canister_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v1/feed/coldstart/clean").unwrap();

    let req = FeedRequest {
        canister_id,
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
    let response = response.json::<FeedResponse>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_coldstart_nsfw(
    canister_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v1/feed/coldstart/nsfw").unwrap();

    let req = FeedRequest {
        canister_id,
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
    let response = response.json::<FeedResponse>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_coldstart_mixed(
    canister_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v1/feed/coldstart/mixed").unwrap();

    let req = FeedRequest {
        canister_id,
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
    let response = response.json::<FeedResponse>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_clean(
    canister_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v1/feed/clean").unwrap();

    let req = FeedRequest {
        canister_id,
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
    let response = response.json::<FeedResponse>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_nsfw(
    canister_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v1/feed/nsfw").unwrap();

    let req = FeedRequest {
        canister_id,
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
    let response = response.json::<FeedResponse>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_mixed(
    canister_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v1/feed/mixed").unwrap();

    let req = FeedRequest {
        canister_id,
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
    let response = response.json::<FeedResponse>().await?;

    Ok(response.posts)
}

pub async fn get_ml_feed_mixed_v2(
    canister_id: Principal,
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItem>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v2/feed/mixed").unwrap();

    let req = FeedRequestV2 {
        user_id: user_id.to_string(),
        canister_id: canister_id.to_string(),
        filter_results: post_details_to_video_ids(filter_results),
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

    let posts = response
        .posts
        .into_iter()
        .map(|post| PostItem {
            post_id: post.post_id,
            canister_id: Principal::from_text(post.canister_id).unwrap(),
            video_id: post.video_id,
            nsfw_probability: post.nsfw_probability,
        })
        .collect();

    Ok(posts)
}

pub fn post_details_to_post_item(post_details: Vec<PostDetails>) -> Vec<PostItem> {
    post_details
        .into_iter()
        .map(|post_detail| PostItem {
            post_id: post_detail.post_id,
            canister_id: post_detail.canister_id,
            video_id: post_detail.uid,
            nsfw_probability: post_detail.nsfw_probability,
        })
        .collect()
}

pub fn post_details_to_video_ids(post_details: Vec<PostDetails>) -> Vec<String> {
    post_details
        .into_iter()
        .map(|post_detail| post_detail.uid)
        .collect()
}
