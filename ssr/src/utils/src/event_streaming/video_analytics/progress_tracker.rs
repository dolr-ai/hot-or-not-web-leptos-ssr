use super::constants::*;
use leptos::html::Video;
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct VideoProgressTracker {
    last_video_time: RwSignal<f64>,
    progress_stalled: RwSignal<bool>,
    check_interval: RwSignal<Option<leptos::prelude::IntervalHandle>>,
}

impl VideoProgressTracker {
    pub fn new() -> Self {
        Self {
            last_video_time: RwSignal::new(0.0),
            progress_stalled: RwSignal::new(false),
            check_interval: RwSignal::new(None),
        }
    }

    pub fn start_tracking(&self, video_ref: NodeRef<Video>, log_info: ProgressLogInfo) {
        if self.check_interval.get_untracked().is_some() {
            return;
        }

        let last_time = self.last_video_time;
        let stalled = self.progress_stalled;

        let interval_handle = set_interval_with_handle(
            move || {
                Self::check_progress(video_ref, last_time, stalled, &log_info);
            },
            std::time::Duration::from_millis(PROGRESS_CHECK_INTERVAL_MS),
        );

        if let Ok(handle) = interval_handle {
            self.check_interval.set(Some(handle));
        }
    }

    pub fn stop_tracking(&self) {
        if let Some(handle) = self.check_interval.get_untracked() {
            handle.clear();
            self.check_interval.set(None);
        }
    }

    pub fn reset_stall_state(&self) {
        self.progress_stalled.set(false);
    }

    pub fn is_stalled(&self) -> bool {
        self.progress_stalled.get_untracked()
    }

    fn check_progress(
        video_ref: NodeRef<Video>,
        last_time: RwSignal<f64>,
        stalled: RwSignal<bool>,
        log_info: &ProgressLogInfo,
    ) {
        let Some(video_el) = video_ref.get() else {
            return;
        };

        let current_time = video_el.current_time();
        let duration = video_el.duration();
        let prev_time = last_time.get_untracked();

        let has_looped = current_time < prev_time;
        let time_diff = if has_looped {
            (duration - prev_time) + current_time
        } else {
            current_time - prev_time
        };

        let expected_threshold = VIDEO_PAUSE_ERROR_THRESHOLD_SECONDS * PROGRESS_CHECK_MULTIPLIER;

        if time_diff < expected_threshold && !has_looped {
            if !stalled.get_untracked() {
                stalled.set(true);
                leptos::logging::error!(
                    "video_log: Video stalled for more than {} seconds at position={:.2}s, video_id={}, publisher_canister_id={}, post_id={} ; expected progress: {:.2}s, actual progress: {:.2}s at position={:.2}s",
                    expected_threshold,
                    current_time,
                    log_info.video_id,
                    log_info.publisher_canister_id,
                    log_info.post_id,
                    VIDEO_PAUSE_ERROR_THRESHOLD_SECONDS,
                    time_diff,
                    current_time,
                );
            }
        } else if stalled.get_untracked()
            && (time_diff >= VIDEO_PAUSE_ERROR_THRESHOLD_SECONDS || has_looped)
        {
            leptos::logging::log!(
                "Video progress resumed, video_id={}, publisher_canister_id={}, post_id={}",
                log_info.video_id,
                log_info.publisher_canister_id,
                log_info.post_id
            );
            stalled.set(false);
        }

        last_time.set(current_time);
    }
}

#[derive(Clone)]
pub struct ProgressLogInfo {
    pub video_id: String,
    pub publisher_canister_id: String,
    pub post_id: String,
}
