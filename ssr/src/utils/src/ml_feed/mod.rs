use candid::Principal;
use consts::ML_FEED_URL;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::post::FeedRequestV2;
use yral_types::post::FeedResponseV2;
use yral_types::post::PostItemV2;

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
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItemV2>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v3/feed/clean").unwrap();

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

pub async fn get_ml_feed_nsfw(
    user_id: Principal,
    num_results: u32,
    filter_results: Vec<PostDetails>,
) -> Result<Vec<PostItemV2>, anyhow::Error> {
    let client = reqwest::Client::new();
    let ml_feed_url = ML_FEED_URL.join("api/v3/feed/nsfw").unwrap();

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
