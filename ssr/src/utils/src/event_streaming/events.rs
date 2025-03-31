use super::EventHistory;
use candid::Principal;
use codee::string::FromToStringCodec;
use codee::string::JsonSerdeCodec;
use consts::ACCOUNT_CONNECTED_STORE;
use consts::USER_CANISTER_ID_STORE;
use consts::USER_PRINCIPAL_STORE;
use ic_agent::Identity;
use leptos::html::Input;
use leptos::prelude::Signal;
use leptos::{ev, prelude::*};
use leptos_use::storage::use_local_storage;
use leptos_use::use_cookie;
use leptos_use::{use_event_listener, use_timeout_fn, UseTimeoutFnReturn};
use serde_json::json;
use sns_validation::pbs::sns_pb::SnsInitPayload;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    #[cfg(feature = "local-auth")]
    LocalStorage,
    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
    Google,
}
/// The store for Authenticated canisters
/// Do not use this for anything other than analytics
pub fn auth_canisters_store() -> RwSignal<Option<Canisters<true>>> {
    expect_context()
}

use circular_buffer::CircularBuffer;
/// Prevents hydration bugs if the value in store is used to conditionally show views
/// this is because the server will always get a `false` value and do rendering based on that
pub fn account_connected_reader() -> (ReadSignal<bool>, Effect<LocalStorage>) {
    let (read_account_connected, _, _) =
        use_local_storage::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
    let (is_connected, set_is_connected) = signal(false);

    (
        is_connected,
        Effect::new(move |_| {
            set_is_connected(read_account_connected());
        }),
    )
}

#[derive(Clone)]
pub struct HistoryCtx {
    pub history: RwSignal<CircularBuffer<3, String>>,
}

impl Default for HistoryCtx {
    fn default() -> Self {
        Self {
            history: RwSignal::new(CircularBuffer::<3, String>::new()),
        }
    }
}

impl HistoryCtx {
    pub fn new() -> Self {
        Self {
            history: RwSignal::new(CircularBuffer::<3, String>::new()),
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
use crate::event_streaming::{
    send_event_ssr, send_event_ssr_spawn, send_event_warehouse_ssr_spawn, send_user_id,
};
use crate::token::nsfw::NSFWInfo;
use crate::user::{user_details_can_store_or_ret, user_details_or_ret};
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
    TokenCreationCompleted(TokenCreationCompleted),
    TokenCreationFailed(TokenCreationFailed),
    TokensClaimedFromNeuron(TokensClaimedFromNeuron),
    TokensTransferred(TokensTransferred),
    PageVisit(PageVisit),
    CentsAdded(CentsAdded),
    CentsWithdrawn(CentsWithdrawn),
    TokenPumpedDumped(TokenPumpedDumped),
}

#[derive(Default)]
pub struct VideoWatched;

impl VideoWatched {
    pub fn send_event(
        &self,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let (is_connected, _) = account_connected_reader();

            // video_viewed - analytics
            let (video_watched, set_video_watched) = signal(false);
            let (full_video_watched, set_full_video_watched) = signal(false);

            let cans_store: RwSignal<Option<Canisters<true>>> = auth_canisters_store();

            let post_for_time = vid_details;
            let _ = use_event_listener(container_ref, ev::timeupdate, move |evt| {
                let user = user_details_can_store_or_ret!(cans_store);
                let post_o = post_for_time();
                let post = post_o.as_ref();

                let target = evt.target().unwrap();
                let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
                let duration = video.duration();
                let current_time = video.current_time();
                let nsfw_probability = post.map(|p| p.nsfw_probability);
                let is_nsfw = nsfw_probability.map(|prob| prob > 0.5);
                if current_time < 0.95 * duration {
                    set_full_video_watched.set(false);
                }

                // send bigquery event when video is watched > 95%
                if current_time >= 0.95 * duration && !full_video_watched.get() {
                    send_event_warehouse_ssr_spawn(
                        "video_duration_watched".to_string(),
                        json!({
                            "publisher_user_id": post.map(|p| p.poster_principal),
                            "user_id": user.details.principal,
                            "is_loggedIn": is_connected(),
                            "display_name": user.details.display_name.clone(),
                            "canister_id": user.canister_id,
                            "video_id": post.map(|p| p.uid.clone()),
                            "video_category": "NA",
                            "creator_category": "NApublisher_canister_id(",
                            "hashtag_count": post.map(|p| p.hastags.len()),
                            "is_NSFW": is_nsfw,
                            "is_hotorNot": post.map(|p| p.is_hot_or_not()),
                            "feed_type": "NA",
                            "view_count": post.map(|p| p.views),
                            "like_count": post.map(|p| p.likes),
                            "share_count": 0,
                            "percentage_watched": 100.0,
                            "absolute_watched": duration,
                            "video_duration": duration,
                            "post_id": post.map(|p| p.post_id),
                            "publisher_canister_id": post.map(|p| p.canister_id),
                            "nsfw_probability": nsfw_probability,
                        })
                        .to_string(),
                    );

                    set_full_video_watched.set(true);
                }

                if video_watched.get() {
                    return;
                }

                if current_time >= 3.0 {
                    send_event_ssr_spawn(
                        "video_viewed".to_string(),
                        json!({
                            "publisher_user_id": post.map(|p| p.poster_principal),
                            "user_id": user.details.principal,
                            "is_loggedIn": is_connected(),
                            "display_name": user.details.display_name,
                            "canister_id": user.canister_id,
                            "video_id": post.map(|p| p.uid.clone()),
                            "video_category": "NA",
                            "creator_category": "NA",
                            "hashtag_count": post.map(|p| p.hastags.len()),
                            "is_NSFW": is_nsfw,
                            "is_hotorNot": post.map(|p| p.is_hot_or_not()),
                            "feed_type": "NA",
                            "view_count": post.map(|p| p.views),
                            "like_count": post.map(|p| p.likes),
                            "share_count": 0,
                            "post_id": post.map(|p| p.post_id),
                            "publisher_canister_id": post.map(|p| p.canister_id),
                            "nsfw_probability": nsfw_probability,
                        })
                        .to_string(),
                    );
                    set_video_watched.set(true);
                }
            });

            // video duration watched - warehousing
            let post_for_warehouse = vid_details;
            let _ = use_event_listener(container_ref, ev::pause, move |evt| {
                let user = user_details_can_store_or_ret!(cans_store);
                let post_o = post_for_warehouse();
                let post = post_o.as_ref();
                let nsfw_probability = post.map(|p| p.nsfw_probability);
                let is_nsfw = nsfw_probability.map(|prob| prob > 0.5);
                let target = evt.target().unwrap();
                let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
                let duration = video.duration();
                let current_time = video.current_time();
                if current_time < 0.1 {
                    return;
                }

                let percentage_watched = (current_time / duration) * 100.0;

                send_event_warehouse_ssr_spawn(
                    "video_duration_watched".to_string(),
                    json!({
                        "publisher_user_id": post.map(|p| p.poster_principal),
                        "user_id": user.details.principal,
                        "is_loggedIn": is_connected(),
                        "display_name": user.details.display_name.clone(),
                        "canister_id": user.canister_id,
                        "video_id": post.map(|p| p.uid.clone()),
                        "video_category": "NA",
                        "creator_category": "NA",
                        "hashtag_count": post.map(|p| p.hastags.len()),
                        "is_NSFW": is_nsfw,
                        "is_hotorNot": post.map(|p| p.is_hot_or_not()),
                        "feed_type": "NA",
                        "view_count": post.map(|p| p.views),
                        "like_count": post.map(|p| p.likes),
                        "share_count": 0,
                        "percentage_watched": percentage_watched,
                        "absolute_watched": current_time,
                        "video_duration": duration,
                        "post_id": post.map(|p| p.post_id),
                        "publisher_canister_id": post.map(|p| p.canister_id),
                        "nsfw_probability": nsfw_probability,
                    })
                    .to_string(),
                );
            });
        }
    }
}

#[derive(Default)]
pub struct LikeVideo;

impl LikeVideo {
    pub fn send_event(
        &self,
        post_details: PostDetails,
        likes: RwSignal<u64>,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
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

            let (is_connected, _) = account_connected_reader();
            // like_video - analytics

            let user = user_details_can_store_or_ret!(cans_store);

            send_event_ssr_spawn(
                "like_video".to_string(),
                json!({
                    "publisher_user_id":publisher_user_id,
                    "user_id": user.details.principal,
                    "is_loggedIn": is_connected(),
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
    pub fn send_event(
        &self,
        post_details: PostDetails,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
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

            let (is_connected, _) = account_connected_reader();

            let user = user_details_can_store_or_ret!(cans_store);

            // share_video - analytics
            send_event_ssr_spawn(
                "share_video".to_string(),
                json!({
                    "publisher_user_id":publisher_user_id,
                    "user_id": user.details.principal,
                    "is_loggedIn": is_connected.get(),
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
    pub fn send_event(&self) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_initiated - analytics
            let user = user_details_or_ret!();
            send_event_ssr_spawn(
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
        hashtag_inp: NodeRef<Input>,
        is_nsfw: NodeRef<Input>,
        enable_hot_or_not: NodeRef<Input>,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_upload_button_clicked - analytics
            let user = user_details_can_store_or_ret!(cans_store);

            let hashtag_count = hashtag_inp.get_untracked().unwrap().value().len();
            let is_nsfw_val = is_nsfw
                .get_untracked()
                .map(|v| v.checked())
                .unwrap_or_default();
            let is_hotornot_val = enable_hot_or_not
                .get_untracked()
                .map(|v| v.checked())
                .unwrap_or_default();

            Effect::new(move |_| {
                send_event_ssr_spawn(
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
    pub fn send_event(&self, cans_store: RwSignal<Option<Canisters<true>>>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_video_selected - analytics
            let user = user_details_can_store_or_ret!(cans_store);

            send_event_ssr_spawn(
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
        error: String,
        hashtags_len: usize,
        is_nsfw: bool,
        enable_hot_or_not: bool,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_unsuccessful - analytics
            let user = user_details_can_store_or_ret!(cans_store);

            send_event_ssr_spawn(
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
        video_id: String,
        hashtags_len: usize,
        is_nsfw: bool,
        enable_hot_or_not: bool,
        post_id: u64,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // video_upload_successful - analytics
            let user = user_details_can_store_or_ret!(cans_store);
            send_event_ssr_spawn(
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
    pub fn send_event(&self, logged_in: ReadSignal<bool>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // refer - analytics

            let user = user_details_or_ret!();
            let details = user.details;
            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;

            let history_ctx: HistoryCtx = expect_context();
            let prev_site = history_ctx.prev_url_untracked();

            // refer - analytics
            send_event_ssr_spawn(
                "refer".to_string(),
                json!({
                    "user_id":user_id,
                    "is_loggedIn": logged_in.get_untracked(),
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
    pub fn send_event(
        &self,
        logged_in: ReadSignal<bool>,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // refer_share_link - analytics
            let user = user_details_can_store_or_ret!(cans_store);
            let details = user.details;

            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;

            let history_ctx: HistoryCtx = expect_context();
            let prev_site = history_ctx.prev_url_untracked();

            // refer_share_link - analytics
            send_event_ssr_spawn(
                "refer_share_link".to_string(),
                json!({
                    "user_id":user_id,
                    "is_loggedIn": logged_in.get_untracked(),
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
    pub fn send_event(&self, canisters: Canisters<true>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_successful - analytics

            let user_id = canisters.identity().sender().unwrap();
            let canister_id = canisters.user_canister();

            send_user_id(user_id.to_string());

            // login_successful - analytics
            send_event_ssr_spawn(
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
    }
}

#[derive(Default)]
pub struct LoginMethodSelected;

impl LoginMethodSelected {
    pub fn send_event(&self, prov: ProviderKind) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_method_selected - analytics
            send_event_ssr_spawn(
                "login_method_selected".to_string(),
                json!({
                    "login_method": match prov {
                        #[cfg(feature = "local-auth")]
                        ProviderKind::LocalStorage => "local_storage",
                        #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
                        ProviderKind::Google => "google",
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
    pub fn send_event(&self) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            // login_join_overlay_viewed - analytics
            let user = user_details_or_ret!();
            let event_history: EventHistory = expect_context();

            let user_id = user.details.principal;

            send_event_ssr_spawn(
                "login_join_overlay_viewed".to_string(),
                json!({
                    "user_id_viewer": user_id,
                    "previous_event": event_history.event_name.get_untracked(),
                })
                .to_string(),
            );

            send_user_id(user_id.to_string());
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

            send_event_ssr_spawn(
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
    pub fn send_event(&self, cans_store: RwSignal<Option<Canisters<true>>>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let user = user_details_can_store_or_ret!(cans_store);
            let details = user.details;
            // logout_clicked - analytics

            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;

            send_event_ssr_spawn(
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
    pub fn send_event(&self, cans_store: RwSignal<Option<Canisters<true>>>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let user = user_details_can_store_or_ret!(cans_store);
            let details = user.details;

            let user_id = details.principal;
            let display_name = details.display_name;
            let canister_id = user.canister_id;
            // logout_confirmation - analytics

            send_event_ssr_spawn(
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
    pub fn send_event(&self, error_str: String, cans_store: RwSignal<Option<Canisters<true>>>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let event_history: EventHistory = expect_context();
            let user = user_details_can_store_or_ret!(cans_store);
            let details = user.details;

            let user_id = details.principal;
            let canister_id = user.canister_id;

            // error_event - analytics
            send_event_ssr_spawn(
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
    pub fn send_event(
        &self,
        post_details: PostDetails,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let publisher_user_id = post_details.poster_principal;
            let video_id = post_details.uid.clone();

            let (is_connected, _) = account_connected_reader();

            let user = user_details_can_store_or_ret!(cans_store);

            send_event_ssr_spawn(
                "profile_view_video".to_string(),
                json!({
                    "publisher_user_id":publisher_user_id,
                    "user_id": user.details.principal,
                    "is_loggedIn": is_connected(),
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
    pub fn send_event(
        &self,
        sns_init_payload: SnsInitPayload,
        cans_store: RwSignal<Option<Canisters<true>>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let user = user_details_can_store_or_ret!(cans_store);
            let details = user.details;

            let user_id = details.principal;
            let canister_id = user.canister_id;

            // token_creation_started - analytics
            send_event_ssr_spawn(
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
pub struct TokenCreationCompleted;

impl TokenCreationCompleted {
    pub async fn send_event(
        &self,
        sns_init_payload: SnsInitPayload,
        token_root: Principal,
        profile_details: ProfileDetails,
        canister_id: Principal,
        nsfw_info: NSFWInfo,
    ) {
        #[cfg(feature = "ga4")]
        {
            let user_id = profile_details.principal;

            let link = format!("/token/info/{token_root}");

            // token_creation_completed - analytics
            let _ = send_event_ssr(
                "token_creation_completed".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "token_name": sns_init_payload.token_name,
                    "token_symbol": sns_init_payload.token_symbol,
                    "name": sns_init_payload.name,
                    "description": sns_init_payload.description,
                    "logo": sns_init_payload.logo,
                    "link": link,
                    "is_nsfw": nsfw_info.is_nsfw,
                    "nsfw_ec": nsfw_info.nsfw_ec,
                    "nsfw_gore": nsfw_info.nsfw_gore,
                    "csam_detected": nsfw_info.csam_detected,
                })
                .to_string(),
            )
            .await;
        }
    }
}

#[derive(Default)]
pub struct TokenCreationFailed;

impl TokenCreationFailed {
    pub async fn send_event(
        &self,
        error_str: String,
        sns_init_payload: SnsInitPayload,
        profile_details: ProfileDetails,
        canister_id: Principal,
    ) {
        #[cfg(feature = "ga4")]
        {
            let user_id = profile_details.principal;

            // token_creation_failed - analytics
            let _ = send_event_ssr(
                "token_creation_failed".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "token_name": sns_init_payload.token_name,
                    "token_symbol": sns_init_payload.token_symbol,
                    "name": sns_init_payload.name,
                    "description": sns_init_payload.description,
                    "error": error_str
                })
                .to_string(),
            )
            .await;
        }
    }
}

#[derive(Default)]
pub struct TokensClaimedFromNeuron;

impl TokensClaimedFromNeuron {
    pub fn send_event(&self, amount: u64, cans_store: Canisters<true>) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let details = cans_store.profile_details();

            let user_id = details.principal;
            let canister_id = cans_store.user_canister();

            // tokens_claimed_from_neuron - analytics
            send_event_ssr_spawn(
                "tokens_claimed_from_neuron".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "amount": amount
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
            send_event_ssr_spawn(
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
    pub fn send_event(&self, canisters: Canisters<true>, pathname: String) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let user_id = canisters.profile_details().principal;
            let (is_connected, _) = account_connected_reader();
            let is_connected = is_connected.get_untracked();

            let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
                move |_| {
                    send_event_ssr_spawn(
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
    pub fn send_event(&self, payment_source: String, amount: u64) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let (canister_id, _, _) =
                use_local_storage::<Option<Principal>, JsonSerdeCodec>(USER_CANISTER_ID_STORE);
            let (user_id, _) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);
            let (is_connected, _) = account_connected_reader();
            let is_connected = is_connected.get_untracked();

            send_event_ssr_spawn(
                "cents_added".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "is_loggedin": is_connected,
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
    pub fn send_event(&self, amount_withdrawn: f64) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let (canister_id, _, _) =
                use_local_storage::<Option<Principal>, JsonSerdeCodec>(USER_CANISTER_ID_STORE);
            let (user_id, _) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);
            let (is_connected, _) = account_connected_reader();
            let is_connected = is_connected.get_untracked();

            send_event_ssr_spawn(
                "cents_withdrawn".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "is_loggedin": is_connected,
                    "amount_withdrawn": amount_withdrawn,
                })
                .to_string(),
            );
        }
    }
}

#[derive(Default)]
pub struct TokenPumpedDumped;

impl TokenPumpedDumped {
    pub fn send_event(
        &self,
        token_name: String,
        token_root: Principal,
        direction: String,
        count: u32,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let (canister_id, _, _) =
                use_local_storage::<Option<Principal>, JsonSerdeCodec>(USER_CANISTER_ID_STORE);
            let (user_id, _) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);

            let is_loggedin = account_connected_reader().0.get_untracked();

            send_event_ssr_spawn(
                "token_pumped_dumped".to_string(),
                json!({
                    "user_id": user_id,
                    "canister_id": canister_id,
                    "token_name": token_name,
                    "token_root": token_root.to_string(),
                    "direction": direction,
                    "count": count,
                    "is_loggedin": is_loggedin,
                })
                .to_string(),
            );
        }
    }
}
