use crate::event_streaming::events::EventCtx;
use crate::mixpanel::mixpanel_events::MixpanelGlobalProps;
use crate::mixpanel::mixpanel_events::MixpanelVideoClickedCTAType;
use crate::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelPostGameType};
use yral_canisters_common::utils::posts::PostDetails;

#[derive(Clone)]
pub enum VideoAnalyticsEvent {
    VideoStarted {
        post: PostDetails,
        is_logged_in: bool,
    },
    VideoViewed {
        post: PostDetails,
        is_logged_in: bool,
    },
    VideoMuted {
        post: PostDetails,
        muted: bool,
    },
}

pub trait VideoAnalyticsProvider: Send + Sync {
    fn track_event(&self, event: VideoAnalyticsEvent, ctx: EventCtx);
}

#[cfg(feature = "ga4")]
pub struct MixpanelProvider;

#[cfg(feature = "ga4")]
impl VideoAnalyticsProvider for MixpanelProvider {
    fn track_event(&self, event: VideoAnalyticsEvent, ctx: EventCtx) {
        let Some(mut global) = MixpanelGlobalProps::from_ev_ctx(ctx) else {
            return;
        };

        match event {
            VideoAnalyticsEvent::VideoStarted { post, is_logged_in } => {
                global.is_logged_in = is_logged_in;
                MixPanelEvent::track_video_started(
                    global,
                    post.uid.clone(),
                    post.poster_principal.to_text(),
                    MixpanelPostGameType::HotOrNot,
                    post.likes,
                    post.views,
                    post.is_nsfw,
                    true,
                );
            }
            VideoAnalyticsEvent::VideoViewed { post, is_logged_in } => {
                global.is_logged_in = is_logged_in;
                MixPanelEvent::track_video_viewed(
                    global,
                    post.uid.clone(),
                    post.poster_principal.to_text(),
                    MixpanelPostGameType::HotOrNot,
                    post.likes,
                    post.views,
                    post.is_nsfw,
                    true,
                );
            }
            VideoAnalyticsEvent::VideoMuted { post, muted } => {
                MixPanelEvent::track_video_clicked(
                    global,
                    post.poster_principal.to_text(),
                    post.likes,
                    post.views,
                    true,
                    post.uid.clone(),
                    MixpanelPostGameType::HotOrNot,
                    if muted {
                        MixpanelVideoClickedCTAType::Mute
                    } else {
                        MixpanelVideoClickedCTAType::Unmute
                    },
                    post.is_nsfw,
                );
            }
        }
    }
}
