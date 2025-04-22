use futures::Future;
use serde::{Deserialize, Serialize};

pub mod ab_testing;
pub mod event_streaming;
pub mod host;
pub mod icon;
pub mod ml_feed;
pub mod notifications;
pub mod posts;
pub mod profile;
#[cfg(feature = "qstash")]
pub mod qstash;
pub mod report;
pub mod route;
pub mod time;
pub mod token;
pub mod types;
pub mod user;
pub mod web;
/// Wrapper for PartialEq that always returns false
/// this is currently only used for resources
/// this does not provide a sane implementation of PartialEq
#[derive(Clone, Serialize, Deserialize)]
pub struct MockPartialEq<T>(pub T);

impl<T> PartialEq for MockPartialEq<T> {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

use std::fmt::Display;

use consts::CF_STREAM_BASE;

pub fn bg_url(uid: impl Display) -> String {
    format!("{CF_STREAM_BASE}/{uid}/thumbnails/thumbnail.jpg")
}

pub fn stream_url(uid: impl Display) -> String {
    format!("{CF_STREAM_BASE}/{uid}/manifest/video.m3u8")
}

pub fn mp4_url(uid: impl Display) -> String {
    format!("{CF_STREAM_BASE}/{uid}/downloads/default.mp4")
}

#[cfg(all(feature = "ga4", feature = "ssr"))]
pub mod off_chain {
    tonic::include_proto!("off_chain");
}

#[cfg(not(feature = "hydrate"))]
pub fn send_wrap<Fut: Future + Send>(
    t: Fut,
) -> impl Future<Output = <Fut as Future>::Output> + Send {
    t
}

/// Wraps a specific future that is not `Send` when `hydrate` feature is enabled
/// the future must be `Send` when `ssr` is enabled
/// use only when necessary (usually inside resources)
/// if you get a Send related error inside an Action, it probably makes more
/// sense to use `Action::new_local` or `Action::new_unsync`
#[cfg(feature = "hydrate")]
pub fn send_wrap<Fut: Future>(t: Fut) -> impl Future<Output = <Fut as Future>::Output> + Send {
    send_wrapper::SendWrapper::new(t)
}
