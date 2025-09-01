use leptos::html::Video;
use leptos::logging;
use leptos::prelude::*;

use crate::event_streaming::events::EventCtx;
#[cfg(all(feature = "hydrate", feature = "ga4"))]
use crate::event_streaming::video_analytics::progress_tracker::ProgressLogInfo;
use yral_canisters_common::utils::posts::PostDetails;

use crate::event_streaming::video_analytics::VideoEventDataBuilder;
#[cfg(feature = "ga4")]
use crate::event_streaming::video_analytics::{VideoAnalyticsEvent, VideoAnalyticsProvider};
use crate::event_streaming::video_analytics::{
    EVENT_VIDEO_DURATION_WATCHED, EVENT_VIDEO_VIEWED, MIN_PAUSE_TIME_SECONDS,
    VIDEO_COMPLETION_PERCENTAGE, VIDEO_VIEWED_THRESHOLD_SECONDS,
};
use leptos::ev;
use leptos_use::use_event_listener;

use super::VideoProgressTracker;

#[cfg(feature = "ga4")]
use crate::event_streaming::{send_event_ssr_spawn, send_event_warehouse_ssr_spawn};

#[cfg(feature = "ga4")]
use super::analytics_provider::MixpanelProvider;

pub struct VideoWatchedHandler {
    progress_tracker: VideoProgressTracker,
}

#[cfg(all(feature = "hydrate", feature = "ga4"))]
struct TimeUpdateListenerParams {
    video_watched: Signal<bool>,
    set_video_watched: WriteSignal<bool>,
    full_video_watched: Signal<bool>,
    set_full_video_watched: WriteSignal<bool>,
    playing_started: RwSignal<bool>,
}

impl Default for VideoWatchedHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoWatchedHandler {
    pub fn new() -> Self {
        Self {
            progress_tracker: VideoProgressTracker::new(),
        }
    }

    pub fn setup_event_tracking(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
        muted: RwSignal<bool>,
    ) {
        self.setup_event_tracking_with_current(ctx, vid_details, container_ref, muted, None);
    }

    pub fn setup_event_tracking_with_current(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
        muted: RwSignal<bool>,
        is_current: Option<Signal<bool>>,
    ) {
        #[cfg(all(feature = "hydrate", feature = "ga4"))]
        {
            let (video_watched, set_video_watched) = signal(false);
            let (full_video_watched, set_full_video_watched) = signal(false);
            let playing_started = RwSignal::new(false);

            self.setup_playing_listener(
                ctx,
                vid_details,
                container_ref,
                playing_started,
                self.progress_tracker,
            );

            let params = TimeUpdateListenerParams {
                video_watched: video_watched.into(),
                set_video_watched,
                full_video_watched: full_video_watched.into(),
                set_full_video_watched,
                playing_started,
            };
            self.setup_timeupdate_listener(ctx, vid_details, container_ref, params);

            self.setup_pause_listener(ctx, vid_details, container_ref, self.progress_tracker);

            self.setup_mute_listener(ctx, vid_details, muted, is_current);
        }
    }

    #[cfg(all(feature = "hydrate", feature = "ga4"))]
    fn setup_playing_listener(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
        playing_started: RwSignal<bool>,
        progress_tracker: VideoProgressTracker,
    ) {
        use leptos::ev;
        use leptos_use::use_event_listener;

        let _ = use_event_listener(container_ref, ev::playing, move |_evt| {
            let Some(_) = container_ref.get() else {
                return;
            };
            playing_started.set(true);

            if progress_tracker.is_stalled() {
                let log_info = Self::create_log_info(vid_details);
                logging::log!(
                    "Video resumed playing after stall, video_id={}, publisher_canister_id={}, post_id={}",
                    log_info.video_id,
                    log_info.publisher_canister_id,
                    log_info.post_id
                );
                progress_tracker.reset_stall_state();
            }

            let log_info = Self::create_log_info(vid_details);
            progress_tracker.start_tracking(container_ref, log_info);

            if let Some(post) = vid_details() {
                #[cfg(feature = "ga4")]
                MixpanelProvider.track_event(
                    VideoAnalyticsEvent::VideoStarted {
                        post,
                        is_logged_in: ctx.is_connected(),
                    },
                    ctx,
                );
            }
        });
    }

    #[cfg(all(feature = "hydrate", feature = "ga4"))]
    fn setup_timeupdate_listener(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
        params: TimeUpdateListenerParams,
    ) {
        let _ = use_event_listener(container_ref, ev::timeupdate, move |evt| {
            use wasm_bindgen::JsCast;

            let Some(user) = ctx.user_details() else {
                return;
            };
            let post_o = vid_details();
            let post = post_o.as_ref();

            let Some(target) = evt.target() else {
                logging::error!("video_log: No target found for video timeupdate event");
                return;
            };
            let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
            let duration = video.duration();
            let current_time = video.current_time();

            if current_time < VIDEO_COMPLETION_PERCENTAGE * duration {
                params.set_full_video_watched.set(false);
            }

            // Track 95% completion
            if current_time >= VIDEO_COMPLETION_PERCENTAGE * duration
                && !params.full_video_watched.get()
            {
                let event_data = VideoEventDataBuilder::from_context(&user, post, &ctx)
                    .with_completion(duration)
                    .build();

                send_event_warehouse_ssr_spawn(
                    EVENT_VIDEO_DURATION_WATCHED.to_string(),
                    serde_json::to_string(&event_data).unwrap_or_default(),
                );

                params.set_full_video_watched.set(true);
            }

            if params.video_watched.get() {
                return;
            }

            // Track 3 second view
            if current_time >= VIDEO_VIEWED_THRESHOLD_SECONDS && params.playing_started.get() {
                let event_data = VideoEventDataBuilder::from_context(&user, post, &ctx).build();

                let _ = send_event_ssr_spawn(
                    EVENT_VIDEO_VIEWED.to_string(),
                    serde_json::to_string(&event_data).unwrap_or_default(),
                );

                if let Some(post) = post {
                    #[cfg(feature = "ga4")]
                    MixpanelProvider.track_event(
                        VideoAnalyticsEvent::VideoViewed {
                            post: post.clone(),
                            is_logged_in: ctx.is_connected(),
                        },
                        ctx,
                    );
                }

                params.playing_started.set(false);
                params.set_video_watched.set(true);
            }
        });
    }

    #[cfg(all(feature = "hydrate", feature = "ga4"))]
    fn setup_pause_listener(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        container_ref: NodeRef<Video>,
        progress_tracker: VideoProgressTracker,
    ) {
        let _ = use_event_listener(container_ref, ev::pause, move |evt| {
            use wasm_bindgen::JsCast;

            progress_tracker.stop_tracking();

            let Some(user) = ctx.user_details() else {
                return;
            };
            let post_o = vid_details();
            let post = post_o.as_ref();

            let Some(target) = evt.target() else {
                logging::error!("No target found for video pause event");
                return;
            };
            let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
            let duration = video.duration();
            let current_time = video.current_time();

            if current_time < MIN_PAUSE_TIME_SECONDS {
                return;
            }

            let event_data = VideoEventDataBuilder::from_context(&user, post, &ctx)
                .with_pause_progress(current_time, duration)
                .build();

            send_event_warehouse_ssr_spawn(
                EVENT_VIDEO_DURATION_WATCHED.to_string(),
                serde_json::to_string(&event_data).unwrap_or_default(),
            );
        });
    }

    #[cfg(all(feature = "hydrate", feature = "ga4"))]
    fn setup_mute_listener(
        &self,
        ctx: EventCtx,
        vid_details: Signal<Option<PostDetails>>,
        muted: RwSignal<bool>,
        is_current: Option<Signal<bool>>,
    ) {
        let mixpanel_video_muted = RwSignal::new(muted.get_untracked());

        Effect::new(move |_| {
            // Check if this is the current video (if is_current is provided)
            if let Some(is_current_signal) = is_current {
                if !is_current_signal.get() {
                    return;
                }
            }

            let current_muted = muted.get();
            if current_muted == mixpanel_video_muted.get_untracked() {
                return;
            }
            mixpanel_video_muted.set(current_muted);

            let post_o = vid_details();
            if let Some(post) = post_o {
                #[cfg(feature = "ga4")]
                MixpanelProvider.track_event(
                    VideoAnalyticsEvent::VideoMuted {
                        post,
                        muted: current_muted,
                    },
                    ctx,
                );
            }
        });
    }

    #[cfg(all(feature = "hydrate", feature = "ga4"))]
    fn create_log_info(vid_details: Signal<Option<PostDetails>>) -> ProgressLogInfo {
        let video_id = vid_details.with(|post| {
            post.as_ref()
                .map(|p| p.uid.clone())
                .unwrap_or_else(|| "unknown".to_string())
        });

        let publisher_canister_id = vid_details.with(|post| {
            post.as_ref()
                .map(|p| p.canister_id.to_text())
                .unwrap_or_else(|| "unknown".to_string())
        });

        let post_id = vid_details.with(|post| {
            post.as_ref()
                .map(|p| p.post_id.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

        ProgressLogInfo {
            video_id,
            publisher_canister_id,
            post_id,
        }
    }
}
