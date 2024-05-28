use std::fmt::Display;

use crate::consts::CF_STREAM_BASE;

pub fn bg_url(uid: impl Display) -> String {
    format!("{CF_STREAM_BASE}/{uid}/thumbnails/thumbnail.jpg")
}

pub fn hls_stream_url(uid: impl Display) -> String {
    format!("{CF_STREAM_BASE}/{uid}/manifest/video.m3u8")
}

pub fn mp4_stream_url(uid: impl Display) -> String {
    format!("{CF_STREAM_BASE}/{uid}/downloads/default.mp4")
}
