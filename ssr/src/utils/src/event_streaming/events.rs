use super::EventHistory;
use candid::Principal;
use ic_agent::Identity;
use leptos::html::Input;
use leptos::prelude::Signal;
use leptos::{ev, prelude::*};
use leptos_use::{use_event_listener, use_timeout_fn, UseTimeoutFnReturn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sns_validation::pbs::sns_pb::SnsInitPayload;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
    YralAuth,
}

use circular_buffer::CircularBuffer;

#[derive(Clone)]
pub struct HistoryCtx {
    pub history: RwSignal<CircularBuffer<3, String>>,
    pub utm: RwSignal<Vec<(String, String)>>,
}

impl Default for HistoryCtx {
    fn default() -> Self {
        Self {
            history: RwSignal::new(CircularBuffer::<3, String>::new()),
            utm: RwSignal::new(Vec::new()),
        }
    }
}

impl HistoryCtx {
    pub fn new() -> Self {
        Self {
            history: RwSignal::new(CircularBuffer::<3, String>::new()),
            utm: RwSignal::new(Vec::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.history.get_untracked().len() == 0
    }

    pub fn len(&self) -> usize {
        self.history.get_untracked().len()
    }

    pub fn push(&self, url: &str) {
        self.history.update(move |h| h.push_back(url.to_string()));
    }

    pub fn push_utm(&self, utm: Vec<(String, String)>) {
        let utm: Vec<(String, String)> = utm
            .iter()
            .filter(|(k, _)| k.contains("utm"))
            .cloned()
            .collect();
        if utm.is_empty() {
            return;
        }
        self.utm.set(utm);
    }

    pub fn back(&self, fallback: &str) -> String {
        self.history.update(move |h| {
            h.pop_back();
        });

        let url = self.history.with(|h| h.back().cloned());
        if let Some(url) = url {
            self.history.update(move |h| {
                h.pop_back();
            });
            url
        } else {
            fallback.to_string()
        }
    }

    pub fn prev_url(&self) -> Option<String> {
        self.history.with(|h| h.back().cloned())
    }

    pub fn prev_url_untracked(&self) -> Option<String> {
        self.history.with_untracked(|h| h.back().cloned())
    }

    pub fn log_history(&self) -> String {
        let history = self.history.get();
        let history_str = history
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");
        history_str
    }
}

#[cfg(feature = "ga4")]
use crate::event_streaming::{send_event_ssr_spawn, send_event_warehouse_ssr_spawn};
use crate::mixpanel::mixpanel_events::{
    MixPanelEvent, MixpanelGlobalProps, MixpanelPostGameType, MixpanelVideoClickedCTAType,
    MixpanelVideoClickedProps, MixpanelVideoViewedProps,
};
use leptos::html::Video;
use yral_canisters_common::{
    utils::{posts::PostDetails, profile::ProfileDetails},
    Canisters,
};

pub enum AnalyticsEvent {
    VideoWatched(VideoWatched),
    LikeVideo(LikeVideo),
    ShareVideo(ShareVideo),
    VideoUploadInitiated(VideoUploadInitiated),
    VideoUploadUploadButtonClicked(VideoUploadUploadButtonClicked),
    VideoUploadVideoSelected(VideoUploadVideoSelected),
    VideoUploadUnsuccessful(VideoUploadUnsuccessful),
    VideoUploadSuccessful(VideoUploadSuccessful),
    Refer(Refer),
    ReferShareLink(ReferShareLink),
    LoginSuccessful(LoginSuccessful),
    LoginMethodSelected(LoginMethodSelected),
    LoginJoinOverlayViewed(LoginJoinOverlayViewed),
    LoginCta(LoginCta),
    LogoutClicked(LogoutClicked),
    LogoutConfirmation(LogoutConfirmation),
    ErrorEvent(ErrorEvent),
    ProfileViewVideo(ProfileViewVideo),
    TokenCreationStarted(TokenCreationStarted),
    TokensTransferred(TokensTransferred),
    PageVisit(PageVisit),
    CentsAdded(CentsAdded),
    CentsWithdrawn(CentsWithdrawn),
}

#[derive(Clone)]
pub struct EventUserDetails {
    pub details: ProfileDetails,
    pub canister_id: Principal,
}

#[derive(Clone, Copy)]
pub struct EventCtx {
    pub is_connected: StoredValue<Box<dyn Fn() -> bool + Send + Sync>>,
    pub user_details: StoredValue<Box<dyn Fn() -> Option<EventUserDetails> + Send + Sync>>,
}

impl EventCtx {
    /// DO NOT USE THIS TO RENDER DOM
    pub fn user_details(&self) -> Option<EventUserDetails> {
        self.user_details.with_value(|c| c())
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.with_value(|c| c())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VideoEventData {
    pub publisher_user_id: Option<Principal>,
    pub user_id: Principal,
    pub is_logged_in: bool,
    pub display_name: Option<String>,
    pub canister_id: Principal,
    pub video_id: Option<String>,
    pub video_category: String,
    pub creator_category: String,
    pub hashtag_count: Option<usize>,
    pub is_nsfw: Option<bool>,
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

impl VideoEventData {
    pub fn from_details(
        user: &EventUserDetails,
        post: Option<&PostDetails>,
        ctx: &EventCtx,
    ) -> Self {
        let nsfw_probability = post.map(|p| p.nsfw_probability);
        let is_nsfw = nsfw_probability.map(|prob| prob > 0.5);

        Self {
            publisher_user_id: post.map(|p| p.poster_principal),
            user_id: user.details.principal,
            is_logged_in: ctx.is_connected(),
            display_name: user.details.display_name.clone(),
            canister_id: user.canister_id,
            video_id: post.map(|p| p.uid.clone()),
            video_category: "NA".to_string(),
            creator_category: "NA".to_string(),
            hashtag_count: post.map(|p| p.hastags.len()),
            is_nsfw,
            is_hotor_not: post.map(|p| p.is_hot_or_not()),
            feed_type: "NA".to_string(),
            view_count: post.map(|p| p.views),
            like_count: post.map(|p| p.likes),
            share_count: 0,
            post_id: post.map(|p| p.post_id),
            publisher_canister_id: post.map(|p| p.canister_id),
            nsfw_probability,
            percentage_watched: None,
            absolute_watched: None,
            video_duration: None,
        }
    }
}

#[derive(Default)]
pub struct VideoWatched;

impl VideoWatched {
    pub fn send_event(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
        muted: RwSignal<bool>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};

            // video_viewed - analytics
            let (video_watched, set_video_watched) = signal(false);
            let (full_video_watched, set_full_video_watched) = signal(false);
            let playing_started = RwSignal::new(false);
            // let stall_start_time = RwSignal::new(None::<f64>);

            // // Setup 5-second stall/buffer timeout
            // let UseTimeoutFnReturn {
            //     start: start_stall_timeout,
            //     stop: stop_stall_timeout,
            //     ..
            // } = use_timeout_fn(
            //     move |_| {
            //         leptos::logging::error!(
            //             "Video has been stalled/buffering for more than 5 seconds"
            //         );
            //     },
            //     5000.0, // 5 seconds
            // );

            // Clone timeout functions for use in multiple closures
            // let stop_stall_timeout_playing = stop_stall_timeout.clone();
            // let start_stall_timeout_waiting = start_stall_timeout.clone();
            // let start_stall_timeout_stalled = start_stall_timeout.clone();
            // let stop_stall_timeout_canplay = stop_stall_timeout.clone();
            // let stop_stall_timeout_ended = stop_stall_timeout.clone();

            // let _ = use_event_listener(container_ref, ev::playing, move |_evt| {
            //     let Some(_) = container_ref.get() else {
            //         return;
            //     };
            //     playing_started.set(true);
            //     // Video is playing, stop the stall timeout
            //     stop_stall_timeout_playing();
            //     stall_start_time.set(None);
            // });

            // // Track when video starts buffering/waiting for data
            // let _ = use_event_listener(container_ref, ev::waiting, move |evt| {
            //     let Some(target) = evt.target() else {
            //         return;
            //     };
            //     let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
            //     let current_time = video.current_time();

            //     leptos::logging::warn!("Video waiting/buffering at time: {}", current_time);
            //     stall_start_time.set(Some(current_time));
            //     start_stall_timeout_waiting(());
            // });

            // Track when video stalls
            // let _ = use_event_listener(container_ref, ev::stalled, move |evt| {
            //     let Some(target) = evt.target() else {
            //         return;
            //     };
            //     // let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
            //     // let current_time = video.current_time();

            //     // leptos::logging::warn!("Video stalled at time: {}", current_time);
            //     // if stall_start_time.get().is_none() {
            //     //     stall_start_time.set(Some(current_time));
            //     //     start_stall_timeout_stalled(());
            //     // }
            // });

            // Stop stall timeout when video can play through
            // let _ = use_event_listener(container_ref, ev::canplaythrough, move |_evt| {
            //     stop_stall_timeout_canplay();
            //     stall_start_time.set(None);
            // });

            let _ = use_event_listener(container_ref, ev::timeupdate, move |evt| {
                let Some(user) = ctx.user_details() else {
                    return;
                };
                let post_o = vid_details();
                let post = post_o.as_ref();

                let Some(target) = evt.target() else {
                    leptos::logging::error!("No target found for video timeupdate event");
                    return;
                };
                let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
                let duration = video.duration();
                let current_time = video.current_time();
                if current_time < 0.95 * duration {
                    set_full_video_watched.set(false);
                }

                // send bigquery event when video is watched > 95%
                if current_time >= 0.95 * duration && !full_video_watched.get() {
                    // Initialize base event data and add duration-specific fields
                    let mut event_data = VideoEventData::from_details(&user, post, &ctx);
                    event_data.percentage_watched = Some(100.0);
                    event_data.absolute_watched = Some(duration);
                    event_data.video_duration = Some(duration);

                    send_event_warehouse_ssr_spawn(
                        "video_duration_watched".to_string(),
                        serde_json::to_string(&event_data).unwrap_or_default(),
                    );

                    set_full_video_watched.set(true);
                }

                if video_watched.get() {
                    return;
                }

                if current_time >= 3.0 && playing_started() {
                    // Initialize event data for video_viewed event
                    let event_data = VideoEventData::from_details(&user, post, &ctx);

                    let _ = send_event_ssr_spawn(
                        "video_viewed".to_string(),
                        serde_json::to_string(&event_data).unwrap_or_default(),
                    );

                    // Mixpanel tracking
                    let Some(global) = MixpanelGlobalProps::from_ev_ctx(ctx) else {
                        return;
                    };
                    if let Some(post) = post {
                        let is_logged_in = ctx.is_connected();
                        let is_game_enabled = true;

                        MixPanelEvent::track_video_viewed(MixpanelVideoViewedProps {
                            publisher_user_id: post.poster_principal.to_text(),
                            user_id: global.user_id,
                            visitor_id: global.visitor_id,
                            is_logged_in,
                            canister_id: global.canister_id,
                            is_nsfw_enabled: global.is_nsfw_enabled,
                            video_id: post.uid.clone(),
                            view_count: post.views,
                            like_count: post.likes,
                            game_type: MixpanelPostGameType::HotOrNot,
                            is_nsfw: post.is_nsfw,
                            is_game_enabled,
                        });
                    }
                    playing_started.set(false);

                    set_video_watched.set(true);
                }
            });

            // video duration watched - warehousing
            let _ = use_event_listener(container_ref, ev::pause, move |evt| {
                let Some(user) = ctx.user_details() else {
                    return;
                };
                let post_o = vid_details();
                let post = post_o.as_ref();

                let Some(target) = evt.target() else {
                    leptos::logging::error!("No target found for video pause event");
                    return;
                };
                let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
                let duration = video.duration();
                let current_time = video.current_time();
                if current_time < 0.1 {
                    return;
                }

                let percentage_watched = (current_time / duration) * 100.0;

                // Initialize event data and add pause-specific fields
                let mut event_data = VideoEventData::from_details(&user, post, &ctx);
                event_data.percentage_watched = Some(percentage_watched);
                event_data.absolute_watched = Some(current_time);
                event_data.video_duration = Some(duration);

                send_event_warehouse_ssr_spawn(
                    "video_duration_watched".to_string(),
                    serde_json::to_string(&event_data).unwrap_or_default(),
                );
            });

            // Stop stall timeout when video ends
            // let _ = use_event_listener(container_ref, ev::ended, move |_evt| {
            //     stop_stall_timeout_ended();
            //     stall_start_time.set(None);
            // });

            // Track mute/unmute events
            let mixpanel_video_muted = RwSignal::new(muted.get_untracked());

            Effect::new(move |_| {
                let current_muted = muted.get();
                if current_muted == mixpanel_video_muted.get_untracked() {
                    return;
                }
                mixpanel_video_muted.set(current_muted);

                let Some(global) = MixpanelGlobalProps::from_ev_ctx(ctx) else {
                    return;
                };

                let post_o = vid_details();
                let post = post_o.as_ref();

                if let Some(post) = post {
                    let is_game_enabled = true;

                    MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                        user_id: global.user_id,
                        visitor_id: global.visitor_id,
                        is_logged_in: global.is_logged_in,
                        canister_id: global.canister_id,
                        is_nsfw_enabled: global.is_nsfw_enabled,
                        publisher_user_id: post.poster_principal.to_text(),
                        like_count: post.likes,
                        view_count: post.views,
                        is_game_enabled,
                        video_id: post.uid.clone(),
                        is_nsfw: post.is_nsfw,
                        game_type: MixpanelPostGameType::HotOrNot,
                        cta_type: if current_muted {
                            MixpanelVideoClickedCTAType::Mute
                        } else {
                            MixpanelVideoClickedCTAType::Unmute
                        },
                    });
                }
            });
        }
    }
}

#[derive(Default)]
pub struct LikeVideo;

impl LikeVideo {
    pub fn send_event(&self, ctx: EventCtx, post_details: PostDetails, likes: RwSignal<u64>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let publisher_user_id = post_details.poster_principal;
            let video_id = post_details.uid.clone();
            let hastag_count = post_details.hastags.len();
            let is_nsfw = post_details.is_nsfw;
            let is_hotornot = post_details.hot_or_not_feed_ranking_score.is_some();
            let view_count = post_details.views;
            let post_id = post_details.post_id;
            let publisher_canister_id = post_details.canister_id;
            let nsfw_probability = post_details.nsfw_probability;

            // like_video - analytics

            let Some(user) = ctx.user_details() else {
                return;
            };

            let _ = send_event_ssr_spawn(
                "like_video".to_string(),
                json!({
                    "publisher_user_id":publisher_user_id,
                    "user_id": user.details.principal,
                    "is_loggedIn": ctx.is_connected(),
                    "display_name": user.details.display_name,
                    "canister_id": user.canister_id,
                    "video_id": video_id,
                    "video_category": "NA",
                    "creator_category": "NA",
                    "hashtag_count": hastag_count,
                    "is_NSFW": is_nsfw,
                    "is_hotorNot": is_hotornot,
                    "feed_type": "NA",
                    "view_count": view_count,
                    "like_count": likes.get(),
                    "share_count": 0,
                    "post_id": post_id,
                    "publisher_canister_id": publisher_canister_id,
                    "nsfw_probability": nsfw_probability,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct ShareVideo;

impl ShareVideo {
    pub fn send_event(&self, ctx: EventCtx, post_details: PostDetails) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let publisher_user_id = post_details.poster_principal;
            let video_id = post_details.uid.clone();
            let hastag_count = post_details.hastags.len();
            let is_nsfw = post_details.is_nsfw;
            let is_hotornot = post_details.hot_or_not_feed_ranking_score.is_some();
            let view_count = post_details.views;
            let like_count = post_details.likes;
            let nsfw_probability = post_details.nsfw_probability;

            let Some(user) = ctx.user_details() else {
                return;
            };

            // share_video - analytics
            let _ = send_event_ssr_spawn(
                "share_video".to_string(),
                json!({
                    "publisher_user_id":publisher_user_id,
                    "user_id": user.details.principal,
                    "is_loggedIn": ctx.is_connected(),
                    "display_name": user.details.display_name,
                    "canister_id": user.canister_id,
                    "video_id": video_id,
                    "video_category": "NA",
                    "creator_category": "NA",
                    "hashtag_count": hastag_count,
                    "is_NSFW": is_nsfw,
                    "is_hotorNot": is_hotornot,
                    "feed_type": "NA",
                    "view_count": view_count,
                    "like_count": like_count,
                    "share_count": 0,
                    "nsfw_probability": nsfw_probability,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct VideoUploadInitiated;

impl VideoUploadInitiated {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_initiated - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };
            let _ = send_event_ssr_spawn(
                "video_upload_initiated".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "display_name": user.details.display_name,
                    "canister_id": user.canister_id,
                    "creator_category": "NA",
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct VideoUploadUploadButtonClicked;

impl VideoUploadUploadButtonClicked {
    pub fn send_event(
        &self,
        ctx: EventCtx,
        hashtag_inp: NodeRef<Input>,
        is_nsfw: NodeRef<Input>,
        enable_hot_or_not: NodeRef<Input>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_upload_button_clicked - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };

            let hashtag_count = hashtag_inp
                .get_untracked()
                .map_or_else(|| 0, |input| input.value().len());
            let is_nsfw_val = is_nsfw
                .get_untracked()
                .map(|v| v.checked())
                .unwrap_or_default();
            let is_hotornot_val = enable_hot_or_not
                .get_untracked()
                .map(|v| v.checked())
                .unwrap_or_default();

            Effect::new(move |_| {
                let _ = send_event_ssr_spawn(
                    "video_upload_upload_button_clicked".to_string(),
                    json!({
                        "user_id": user.details.principal,
                        "display_name": user.details.display_name.clone().unwrap_or_default(),
                        "canister_id": user.canister_id,
                        "creator_category": "NA",
                        "hashtag_count": hashtag_count,
                        "is_NSFW": is_nsfw_val,
                        "is_hotorNot": is_hotornot_val,
                    })
                    .to_string(),
                );
            });
        }
    }
}

#[derive(Default)]
pub struct VideoUploadVideoSelected;

impl VideoUploadVideoSelected {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_video_selected - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };

            let _ = send_event_ssr_spawn(
                "video_upload_video_selected".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "display_name": user.details.display_name.unwrap_or_default(),
                    "canister_id": user.canister_id,
                    "creator_category": "NA",
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct VideoUploadUnsuccessful;

impl VideoUploadUnsuccessful {
    #[allow(clippy::too_many_arguments)]
    pub fn send_event(
        &self,
        ctx: EventCtx,
        error: String,
        hashtags_len: usize,
        is_nsfw: bool,
        enable_hot_or_not: bool,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_unsuccessful - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };

            let _ = send_event_ssr_spawn(
                "video_upload_unsuccessful".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "display_name": user.details.display_name.unwrap_or_default(),
                    "canister_id": user.canister_id,
                    "creator_category": "NA",
                    "hashtag_count": hashtags_len,
                    "is_NSFW": is_nsfw,
                    "is_hotorNot": enable_hot_or_not,
                    "fail_reason": error,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct VideoUploadSuccessful;

impl VideoUploadSuccessful {
    pub fn send_event(
        &self,
        ctx: EventCtx,
        video_id: String,
        hashtags_len: usize,
        is_nsfw: bool,
        enable_hot_or_not: bool,
        post_id: u64,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_successful - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };
            let _ = send_event_ssr_spawn(
                "video_upload_successful".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "publisher_user_id": user.details.principal,
                    "display_name": user.details.display_name,
                    "canister_id": user.canister_id,
                    "creator_category": "NA",
                    "hashtag_count": hashtags_len,
                    "is_NSFW": is_nsfw,
                    "is_hotorNot": enable_hot_or_not,
                    "is_filter_used": false,
                    "video_id": video_id,
                    "post_id": post_id,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct Refer;

impl Refer {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // refer - analytics

            let Some(user) = ctx.user_details() else {
                return;
            };
            let details = user.details;
            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;

            let history_ctx: HistoryCtx = expect_context();
            let prev_site = history_ctx.prev_url_untracked();

            // refer - analytics
            let _ = send_event_ssr_spawn(
                "refer".to_string(),
                json!({
                    "user_id":user_id,
                    "is_loggedIn": ctx.is_connected(),
                    "display_name": display_name,
                    "canister_id": canister_id,
                    "refer_location": prev_site,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct ReferShareLink;

impl ReferShareLink {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // refer_share_link - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };
            let details = user.details;

            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;

            let history_ctx: HistoryCtx = expect_context();
            let prev_site = history_ctx.prev_url_untracked();

            // refer_share_link - analytics
            let _ = send_event_ssr_spawn(
                "refer_share_link".to_string(),
                json!({
                    "user_id":user_id,
                    "is_loggedIn": ctx.is_connected(),
                    "display_name": display_name,
                    "canister_id": canister_id,
                    "refer_location": prev_site,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct LoginSuccessful;

impl LoginSuccessful {
    pub fn send_event(&self, canisters: Canisters<true>) -> Result<(), anyhow::Error> {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_successful - analytics

            let user_id = canisters.identity().sender().map_err(|_| {
                leptos::logging::error!("No sender found for login successful event");
                anyhow::anyhow!("No sender found for login successful event")
            })?;
            let canister_id = canisters.user_canister();

            // login_successful - analytics
            let _ = send_event_ssr_spawn(
                "login_successful".to_string(),
                json!({
                    "login_method": "google", // TODO: change this when more providers are added
                    "user_id": user_id.to_string(),
                    "canister_id": canister_id.to_string(),
                    "is_new_user": false,                   // TODO: add this info
                })
                .to_string(),
            );
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct LoginMethodSelected;

impl LoginMethodSelected {
    pub fn send_event(&self, prov: ProviderKind) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_method_selected - analytics
            let _ = send_event_ssr_spawn(
                "login_method_selected".to_string(),
                json!({
                    "login_method": match prov {
                        #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
                        ProviderKind::YralAuth => "yral",
                    },
                    "attempt_count": 1,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct LoginJoinOverlayViewed;

impl LoginJoinOverlayViewed {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_join_overlay_viewed - analytics
            let Some(user) = ctx.user_details() else {
                return;
            };
            let event_history: EventHistory = expect_context();

            let user_id = user.details.principal;

            let _ = send_event_ssr_spawn(
                "login_join_overlay_viewed".to_string(),
                json!({
                    "user_id_viewer": user_id,
                    "previous_event": event_history.event_name.get_untracked(),
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct LoginCta;

impl LoginCta {
    pub fn send_event(&self, cta_location: String) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_cta - analytics

            let event_history: EventHistory = expect_context();

            let _ = send_event_ssr_spawn(
                "login_cta".to_string(),
                json!({
                    "previous_event": event_history.event_name.get_untracked(),
                    "cta_location": cta_location,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct LogoutClicked;

impl LogoutClicked {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let Some(user) = ctx.user_details() else {
                return;
            };
            let details = user.details;
            // logout_clicked - analytics

            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;

            let _ = send_event_ssr_spawn(
                "logout_clicked".to_string(),
                json!({
                    "user_id_viewer": user_id,
                    "display_name": display_name,
                    "canister_id": canister_id,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct LogoutConfirmation;

impl LogoutConfirmation {
    pub fn send_event(&self, ctx: EventCtx) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let Some(user) = ctx.user_details() else {
                return;
            };
            let details = user.details;

            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;
            // logout_confirmation - analytics

            let _ = send_event_ssr_spawn(
                "logout_confirmation".to_string(),
                json!({
                    "user_id_viewer": user_id,
                    "display_name": display_name,
                    "canister_id": canister_id,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct ErrorEvent;

impl ErrorEvent {
    pub fn send_event(&self, ctx: EventCtx, error_str: String) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let event_history: EventHistory = expect_context();
            let Some(user) = ctx.user_details() else {
                return;
            };
            let details = user.details;

            let user_id = details.principal;
            let canister_id = user.canister_id;

            // error_event - analytics
            let _ = send_event_ssr_spawn(
                "error_event".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "description": error_str,
                    "previous_event": event_history.event_name.get_untracked(),
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct ProfileViewVideo;

impl ProfileViewVideo {
    pub fn send_event(&self, ctx: EventCtx, post_details: PostDetails) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let publisher_user_id = post_details.poster_principal;
            let video_id = post_details.uid.clone();

            let Some(user) = ctx.user_details() else {
                return;
            };

            let _ = send_event_ssr_spawn(
                "profile_view_video".to_string(),
                json!({
                    "publisher_user_id":publisher_user_id,
                    "user_id": user.details.principal,
                    "is_loggedIn": ctx.is_connected(),
                    "display_name": user.details.display_name,
                    "canister_id": user.canister_id,
                    "video_id": video_id,
                    "profile_feed": "main",
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct TokenCreationStarted;

impl TokenCreationStarted {
    pub fn send_event(&self, ctx: EventCtx, sns_init_payload: SnsInitPayload) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let Some(user) = ctx.user_details() else {
                return;
            };
            let details = user.details;

            let user_id = details.principal;
            let canister_id = user.canister_id;

            // token_creation_started - analytics
            let _ = send_event_ssr_spawn(
                "token_creation_started".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "token_name": sns_init_payload.token_name,
                    "token_symbol": sns_init_payload.token_symbol,
                    "name": sns_init_payload.name
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct TokensTransferred;

impl TokensTransferred {
    pub fn send_event(&self, amount: String, to: Principal, cans_store: Canisters<true>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let details = cans_store.profile_details();

            let user_id = details.principal;
            let canister_id = cans_store.user_canister();

            // tokens_transferred - analytics
            let _ = send_event_ssr_spawn(
                "tokens_transferred".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "amount": amount,
                    "to": to
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct PageVisit;

impl PageVisit {
    pub fn send_event(&self, user_id: Principal, is_connected: bool, pathname: String) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
                move |_| {
                    let _ = send_event_ssr_spawn(
                        "yral_page_visit".to_string(),
                        json!({
                            "user_id": user_id,
                            "is_loggedIn": is_connected,
                            "pathname": pathname,
                        })
                        .to_string(),
                    );
                },
                10000.0,
            );

            start(());
        }
    }
}

#[derive(Default)]
pub struct CentsAdded;

impl CentsAdded {
    pub fn send_event(&self, ctx: EventCtx, payment_source: String, amount: u64) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let Some(user) = ctx.user_details() else {
                return;
            };

            let _ = send_event_ssr_spawn(
                "cents_added".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "canister_id": user.canister_id,
                    "is_loggedin": ctx.is_connected(),
                    "amount_added": amount,
                    "payment_source": payment_source,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct CentsWithdrawn;

impl CentsWithdrawn {
    pub fn send_event(&self, ctx: EventCtx, amount_withdrawn: f64) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let Some(user) = ctx.user_details() else {
                return;
            };
            let _ = send_event_ssr_spawn(
                "cents_withdrawn".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "canister_id": user.canister_id,
                    "is_loggedin": ctx.is_connected(),
                    "amount_withdrawn": amount_withdrawn,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct SatsWithdrawn;

impl SatsWithdrawn {
    pub fn send_event(&self, ctx: EventCtx, amount_withdrawn: f64) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let Some(user) = ctx.user_details() else {
                return;
            };
            let _ = send_event_ssr_spawn(
                "sats_withdrawn".to_string(),
                json!({
                    "user_id": user.details.principal,
                    "canister_id": user.canister_id,
                    "is_loggedin": ctx.is_connected(),
                    "amount_withdrawn": amount_withdrawn,
                })
                .to_string(),
            );
        }
    }
}
