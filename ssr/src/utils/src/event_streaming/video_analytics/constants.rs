/// Maximum pause duration in seconds before logging an error
pub const VIDEO_PAUSE_ERROR_THRESHOLD_SECONDS: f64 = 3.0;

/// Minimum video watch time in seconds before tracking video_viewed event
pub const VIDEO_VIEWED_THRESHOLD_SECONDS: f64 = 3.0;

/// Video completion percentage threshold (95%)
pub const VIDEO_COMPLETION_PERCENTAGE: f64 = 0.95;

/// Progress check multiplier for stall detection
pub const PROGRESS_CHECK_MULTIPLIER: f64 = 0.6;

/// Minimum pause time in seconds to track pause event
pub const MIN_PAUSE_TIME_SECONDS: f64 = 0.1;

/// Progress check interval in milliseconds
pub const PROGRESS_CHECK_INTERVAL_MS: u64 = (VIDEO_PAUSE_ERROR_THRESHOLD_SECONDS * 1000.0) as u64;

/// Event names
pub const EVENT_VIDEO_VIEWED: &str = "video_viewed";
pub const EVENT_VIDEO_DURATION_WATCHED: &str = "video_duration_watched";
