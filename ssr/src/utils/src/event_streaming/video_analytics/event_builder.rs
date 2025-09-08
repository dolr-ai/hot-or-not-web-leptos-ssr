use crate::{
    event_streaming::events::{EventCtx, EventUserDetails},
    ml_feed::QuickPostDetails,
};
use candid::Principal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoEventData {
    pub publisher_user_id: Option<Principal>,
    pub user_id: Principal,
    #[serde(rename = "is_loggedIn")]
    pub is_logged_in: bool,
    pub display_name: Option<String>,
    pub canister_id: Principal,
    pub video_id: Option<String>,
    pub video_category: String,
    pub creator_category: String,
    pub hashtag_count: Option<usize>,
    #[serde(rename = "is_NSFW", skip_serializing_if = "Option::is_none")]
    pub is_nsfw: Option<bool>,
    #[serde(rename = "is_hotorNot", skip_serializing_if = "Option::is_none")]
    pub is_hotor_not: Option<bool>,
    pub feed_type: String,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub share_count: u64,
    pub post_id: Option<u64>,
    pub publisher_canister_id: Option<Principal>,
    pub nsfw_probability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage_watched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_watched: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_duration: Option<f64>,
}

pub struct VideoEventDataBuilder {
    data: VideoEventData,
}

impl Default for VideoEventDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEventDataBuilder {
    pub fn new() -> Self {
        Self {
            data: VideoEventData {
                publisher_user_id: None,
                user_id: Principal::anonymous(),
                is_logged_in: false,
                display_name: None,
                canister_id: Principal::anonymous(),
                video_id: None,
                video_category: "NA".to_string(),
                creator_category: "NA".to_string(),
                hashtag_count: None,
                is_nsfw: None,
                is_hotor_not: None,
                feed_type: "NA".to_string(),
                view_count: None,
                like_count: None,
                share_count: 0,
                post_id: None,
                publisher_canister_id: None,
                nsfw_probability: None,
                percentage_watched: None,
                absolute_watched: None,
                video_duration: None,
            },
        }
    }

    pub fn from_context(
        user: &EventUserDetails,
        post: Option<&QuickPostDetails>,
        ctx: &EventCtx,
    ) -> Self {
        let nsfw_probability = post.map(|p| p.nsfw_probability);
        let is_nsfw = nsfw_probability.map(|prob| prob > 0.5);

        Self {
            data: VideoEventData {
                publisher_user_id: post.map(|p| p.publisher_user_id),
                user_id: user.details.principal,
                is_logged_in: ctx.is_connected(),
                display_name: user.details.display_name.clone(),
                canister_id: user.canister_id,
                video_id: post.map(|p| p.video_uid.clone()),
                video_category: "NA".to_string(),
                creator_category: "NA".to_string(),
                hashtag_count: None,
                is_nsfw,
                is_hotor_not: None,
                feed_type: "NA".to_string(),
                view_count: None,
                like_count: None,
                share_count: 0,
                post_id: post.map(|p| p.post_id),
                publisher_canister_id: post.map(|p| p.canister_id),
                nsfw_probability,
                percentage_watched: None,
                absolute_watched: None,
                video_duration: None,
            },
        }
    }

    pub fn with_video_progress(
        mut self,
        percentage: f64,
        absolute_time: f64,
        duration: f64,
    ) -> Self {
        self.data.percentage_watched = Some(percentage);
        self.data.absolute_watched = Some(absolute_time);
        self.data.video_duration = Some(duration);
        self
    }

    pub fn with_completion(mut self, duration: f64) -> Self {
        self.data.percentage_watched = Some(100.0);
        self.data.absolute_watched = Some(duration);
        self.data.video_duration = Some(duration);
        self
    }

    pub fn with_pause_progress(self, current_time: f64, duration: f64) -> Self {
        let percentage = (current_time / duration) * 100.0;
        self.with_video_progress(percentage, current_time, duration)
    }

    pub fn with_likes(mut self, likes: u64) -> Self {
        self.data.like_count = Some(likes);
        self
    }

    pub fn with_shares(mut self, shares: u64) -> Self {
        self.data.share_count = shares;
        self
    }

    pub fn build(self) -> VideoEventData {
        self.data
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.data)
    }
}
