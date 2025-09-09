use crate::event_streaming::events::EventCtx;
use crate::mixpanel::mixpanel_events::MixpanelGlobalProps;
use crate::mixpanel::mixpanel_events::MixpanelVideoClickedCTAType;
use crate::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelPostGameType};
use crate::ml_feed::QuickPostDetails;

#[derive(Clone)]
pub enum VideoAnalyticsEvent {
    VideoStarted {
        post: QuickPostDetails,
        is_logged_in: bool,
    },
    VideoViewed {
        post: QuickPostDetails,
        is_logged_in: bool,
    },
    VideoMuted {
        post: QuickPostDetails,
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
                    post.video_uid.clone(),
                    post.publisher_user_id.to_text(),
                    MixpanelPostGameType::HotOrNot,
                    true,
                );
            }
            VideoAnalyticsEvent::VideoViewed { post, is_logged_in } => {
                global.is_logged_in = is_logged_in;
                MixPanelEvent::track_video_viewed(
                    global,
                    post.video_uid.clone(),
                    post.publisher_user_id.to_text(),
                    MixpanelPostGameType::HotOrNot,
                    true,
                );
            }
            VideoAnalyticsEvent::VideoMuted { post, muted } => {
                MixPanelEvent::track_video_clicked(
                    global,
                    post.publisher_user_id.to_text(),
                    true,
                    post.video_uid.clone(),
                    MixpanelPostGameType::HotOrNot,
                    if muted {
                        MixpanelVideoClickedCTAType::Mute
                    } else {
                        MixpanelVideoClickedCTAType::Unmute
                    },
                );
            }
        }
    }
}
