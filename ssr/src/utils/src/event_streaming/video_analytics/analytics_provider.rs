use crate::event_streaming::events::EventCtx;
use crate::mixpanel::mixpanel_events::{
    MixPanelEvent, MixpanelGlobalProps, MixpanelPostGameType, MixpanelVideoClickedCTAType,
    MixpanelVideoClickedProps, MixpanelVideoStartedProps, MixpanelVideoViewedProps,
};
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
        let Some(global) = MixpanelGlobalProps::from_ev_ctx(ctx) else {
            return;
        };

        match event {
            VideoAnalyticsEvent::VideoStarted { post, is_logged_in } => {
                MixPanelEvent::track_video_started(MixpanelVideoStartedProps {
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
                    is_game_enabled: true,
                });
            }
            VideoAnalyticsEvent::VideoViewed { post, is_logged_in } => {
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
                    is_game_enabled: true,
                });
            }
            VideoAnalyticsEvent::VideoMuted { post, muted } => {
                MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                    user_id: global.user_id,
                    visitor_id: global.visitor_id,
                    is_logged_in: global.is_logged_in,
                    canister_id: global.canister_id,
                    is_nsfw_enabled: global.is_nsfw_enabled,
                    publisher_user_id: post.poster_principal.to_text(),
                    like_count: post.likes,
                    view_count: post.views,
                    is_game_enabled: true,
                    video_id: post.uid.clone(),
                    is_nsfw: post.is_nsfw,
                    game_type: MixpanelPostGameType::HotOrNot,
                    cta_type: if muted {
                        MixpanelVideoClickedCTAType::Mute
                    } else {
                        MixpanelVideoClickedCTAType::Unmute
                    },
                });
            }
        }
    }
}
