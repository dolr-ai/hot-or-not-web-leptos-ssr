use indexmap::IndexSet;
use leptos::html::Audio;
use leptos::logging;
use leptos::{html::Video, prelude::*};
use state::canisters::auth_state;
use utils::event_streaming::events::VideoWatched;

use component::video_player::VideoPlayer;
use futures::FutureExt;
use gloo::timers::future::TimeoutFuture;
use utils::{bg_url, mp4_url};

/// Maximum time in milliseconds to wait for video play promise to resolve
const VIDEO_PLAY_TIMEOUT_MS: u64 = 5000;

use super::{overlay::VideoDetailsOverlay, PostDetails};

#[component]
pub fn BgView(
    video_queue: RwSignal<IndexSet<PostDetails>>,
    idx: usize,
    children: Children,
) -> impl IntoView {
    let post_with_prev = Memo::new(move |_| {
        video_queue.with(|q| {
            let cur_post = q.get_index(idx).cloned();
            let prev_post = if idx > 0 {
                q.get_index(idx - 1).cloned()
            } else {
                None
            };
            (cur_post, prev_post)
        })
    });

    let uid = move || {
        post_with_prev()
            .0
            .as_ref()
            .map(|q| q.uid.clone())
            .unwrap_or_default()
    };

    let win_audio_ref = NodeRef::<Audio>::new();

    view! {
        <div class="overflow-hidden relative w-full h-full bg-transparent">
            <div
                class="absolute top-0 left-0 w-full h-full bg-center bg-cover z-1 blur-lg"
                style:background-color="rgb(0, 0, 0)"
                style:background-image=move || format!("url({})", bg_url(uid()))
            ></div>
            <audio
                class="sr-only"
                node_ref=win_audio_ref
                preload="auto"
                src="/img/hotornot/chaching.m4a"
            />
            {move || {
                let (post, prev_post) = post_with_prev.get();
                Some(view! { <VideoDetailsOverlay post=post? prev_post win_audio_ref /> })
            }}
            {children()}
        </div>
    }
    .into_any()
}

#[component]
pub fn VideoView(
    #[prop(into)] post: Signal<Option<PostDetails>>,
    #[prop(optional)] _ref: NodeRef<Video>,
    #[prop(optional)] autoplay_at_render: bool,
    to_load: Memo<bool>,
    muted: RwSignal<bool>,
) -> impl IntoView {
    let post_for_uid = post;
    let uid = Memo::new(move |_| {
        if !to_load() {
            return None;
        }
        post_for_uid.with(|p| p.as_ref().map(|p| p.uid.clone()))
    });
    let view_bg_url = move || uid().map(bg_url);
    let view_video_url = move || uid().map(mp4_url);

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    // Handles mute/unmute
    Effect::new(move |_| {
        let vid = _ref.get()?;
        vid.set_muted(muted());
        Some(())
    });

    Effect::new(move |_| {
        let vid = _ref.get()?;
        // the attributes in DOM don't seem to be working
        // vid.set_muted(muted.get_untracked());
        // vid.set_loop(true);
        if autoplay_at_render {
            vid.set_autoplay(true);
            _ = vid.play();
        }
        Some(())
    });

    VideoWatched.send_event(ev_ctx, post, _ref, muted);

    view! {
        <VideoPlayer
            node_ref=_ref
            view_bg_url=Signal::derive(view_bg_url)
            view_video_url=Signal::derive(view_video_url)
        />
    }
    .into_any()
}

#[component]
pub fn VideoViewForQueue(
    post: RwSignal<Option<PostDetails>>,
    current_idx: RwSignal<usize>,
    idx: usize,
    muted: RwSignal<bool>,
    to_load: Memo<bool>,
) -> impl IntoView {
    let container_ref = NodeRef::<Video>::new();

    // Track if video is already playing to prevent multiple play attempts
    let is_playing = RwSignal::new(false);

    // Handles autoplay
    Effect::new(move |_| {
        let Some(vid) = container_ref.get() else {
            return;
        };

        let is_current = idx == current_idx();
        if !is_current {
            if is_playing.get_untracked() {
                is_playing.set(false);
                _ = vid.pause();
            }
            return;
        }

        // Only attempt to play if not already playing
        if is_current && !is_playing.get_untracked() {
            is_playing.set(true);
            vid.set_autoplay(true);

            if let Some(vid) = container_ref.get() {
                let promise = vid.play();
                if let Ok(promise) = promise {
                    wasm_bindgen_futures::spawn_local(async move {
                        // Create futures
                        let mut play_future = wasm_bindgen_futures::JsFuture::from(promise).fuse();
                        let mut timeout_future =
                            TimeoutFuture::new(VIDEO_PLAY_TIMEOUT_MS as u32).fuse();

                        // Race between play and timeout
                        futures::select! {
                            play_result = play_future => {
                                if let Err(e) = play_result {
                                    logging::error!("video_log: Video play() promise failed: {e:?}");
                                }
                            }
                            _ = timeout_future => {
                                logging::error!("video_log: Video play() did not resolve within 5 seconds");
                            }
                        }
                    });
                } else {
                    logging::error!("video_log: Failed to play video");
                }
            }
        }
    });

    view! {
        <VideoView
            post
            _ref=container_ref
            to_load
            muted
        />
    }
    .into_any()
}
