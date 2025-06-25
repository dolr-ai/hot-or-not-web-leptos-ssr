use std::cmp::Ordering;

use component::buttons::HighlightedButton;
use component::overlay::ShadowOverlay;
use indexmap::IndexSet;
use leptos::html::Audio;
use leptos::{ev, logging};
use leptos::{html::Video, prelude::*};
use leptos_icons::*;
use leptos_use::use_event_listener;
use state::canisters::{auth_state, unauth_canisters};
use utils::mixpanel::mixpanel_events::*;
use utils::send_wrap;
use yral_canisters_client::individual_user_template::PostViewDetailsFromFrontend;

use component::video_player::VideoPlayer;
use utils::event_streaming::events::VideoWatched;
use utils::{bg_url, mp4_url};

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
    let wallet_balance = RwSignal::new(0);
    let show_result_help_ping = RwSignal::new(false);

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
                Some(view! { <VideoDetailsOverlay post=post? prev_post win_audio_ref wallet_balance show_result_help_ping /> })
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
    muted: RwSignal<bool>,
) -> impl IntoView {
    let post_for_uid = post;
    let post_for_mixpanel = post;
    let uid = Memo::new(move |_| post_for_uid.with(|p| p.as_ref().map(|p| p.uid.clone())));
    let view_bg_url = move || uid().map(bg_url);
    let view_video_url = move || uid().map(mp4_url);
    let mixpanel_video_muted = RwSignal::new(muted.get_untracked());

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    let mixpanel_video_clicked_audio_state = Action::new(move |muted: &bool| {
        let ret = async {};
        if *muted == mixpanel_video_muted.get_untracked() {
            return ret;
        }
        mixpanel_video_muted.set(*muted);

        let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
            return ret;
        };

        let post = post_for_mixpanel.get_untracked().unwrap();
        let is_game_enabled = true;

        MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
            user_id: global.user_id,
            visitor_id: global.visitor_id,
            is_logged_in: global.is_logged_in,
            canister_id: global.canister_id,
            is_nsfw_enabled: global.is_nsfw_enabled,
            publisher_user_id: post.poster_principal.to_text(),
            like_count: post.likes,
            view_count: post.views,
            is_game_enabled,
            video_id: post.uid,
            is_nsfw: post.is_nsfw,

            game_type: MixpanelPostGameType::HotOrNot,
            cta_type: if *muted {
                MixpanelVideoClickedCTAType::Mute
            } else {
                MixpanelVideoClickedCTAType::Unmute
            },
        });
        ret
    });

    // Handles mute/unmute
    Effect::new(move |_| {
        let vid = _ref.get()?;
        vid.set_muted(muted());
        mixpanel_video_clicked_audio_state.dispatch(muted());
        Some(())
    });

    Effect::new(move |_| {
        let vid = _ref.get()?;
        // the attributes in DOM don't seem to be working
        vid.set_muted(muted.get_untracked());
        vid.set_loop(true);
        if autoplay_at_render {
            vid.set_autoplay(true);
            _ = vid.play();
        }
        Some(())
    });

    // Video views send to canister
    // 1. When video is paused -> partial video view
    // 2. When video is 95% done -> full view
    let post_for_view = post;
    let send_view_detail_action =
        Action::new(move |(percentage_watched, watch_count): &(u8, u8)| {
            let percentage_watched = *percentage_watched;
            let watch_count = *watch_count;
            let post_for_view = post_for_view;

            send_wrap(async move {
                let canisters = unauth_canisters();

                let payload = match percentage_watched.cmp(&95) {
                    Ordering::Less => {
                        PostViewDetailsFromFrontend::WatchedPartially { percentage_watched }
                    }
                    _ => PostViewDetailsFromFrontend::WatchedMultipleTimes {
                        percentage_watched,
                        watch_count,
                    },
                };

                let post = post_for_view.get_untracked();
                let post_id = post.as_ref().map(|p| p.post_id).unwrap();
                let canister_id = post.as_ref().map(|p| p.canister_id).unwrap();
                let send_view_res = canisters
                    .individual_user(canister_id)
                    .await
                    .update_post_add_view_details(post_id, payload)
                    .await;

                if let Err(err) = send_view_res {
                    log::warn!("failed to send view details: {err:?}");
                }
                Some(())
            })
        });

    let playing_started = RwSignal::new(false);

    let mixpanel_send_view_event = Action::new(move |_| {
        send_wrap(async move {
            let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
                return;
            };
            let post = post_for_view.get_untracked().unwrap();
            let is_logged_in = ev_ctx.is_connected();
            let is_game_enabled = true;

            MixPanelEvent::track_video_viewed(MixpanelVideoViewedProps {
                publisher_user_id: post.poster_principal.to_text(),
                user_id: global.user_id,
                visitor_id: global.visitor_id,
                is_logged_in,
                canister_id: global.canister_id,
                is_nsfw_enabled: global.is_nsfw_enabled,
                video_id: post.uid,
                view_count: post.views,
                like_count: post.likes,
                game_type: MixpanelPostGameType::HotOrNot,
                is_nsfw: post.is_nsfw,
                is_game_enabled,
            });
            playing_started.set(false);
        })
    });

    let mixpanel_video_started_event = Action::new(move |_: &()| {
        send_wrap(async move {
            let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
                return;
            };
            let post = post_for_view.get_untracked().unwrap();
            let is_logged_in = ev_ctx.is_connected();
            let is_game_enabled = true;

            MixPanelEvent::track_video_started(MixpanelVideoStartedProps {
                publisher_user_id: post.poster_principal.to_text(),
                user_id: global.user_id,
                visitor_id: global.visitor_id,
                is_logged_in,
                canister_id: global.canister_id,
                is_nsfw_enabled: global.is_nsfw_enabled,
                video_id: post.uid,
                view_count: post.views,
                like_count: post.likes,
                game_type: MixpanelPostGameType::HotOrNot,
                is_nsfw: post.is_nsfw,
                is_game_enabled,
            });
        })
    });

    let _ = use_event_listener(_ref, ev::playing, move |_evt| {
        let Some(_) = _ref.get() else {
            return;
        };
        playing_started.set(true);
        send_view_detail_action.dispatch((100, 0_u8));
        mixpanel_video_started_event.dispatch(());
    });

    let _ = use_event_listener(_ref, ev::timeupdate, move |_evt| {
        let Some(video) = _ref.get() else {
            return;
        };
        // let duration = video.duration();
        let current_time = video.current_time();

        if current_time >= 3.0 && playing_started() {
            mixpanel_send_view_event.dispatch(());
        }
    });

    VideoWatched.send_event(ev_ctx, post, _ref);

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
    video_queue: RwSignal<IndexSet<PostDetails>>,
    current_idx: RwSignal<usize>,
    idx: usize,
    muted: RwSignal<bool>,
) -> impl IntoView {
    let container_ref = NodeRef::<Video>::new();

    // Handles autoplay
    Effect::new(move |_| {
        let Some(vid) = container_ref.get() else {
            return;
        };
        if idx != current_idx() {
            _ = vid.pause();
            return;
        }
        vid.set_autoplay(true);
        let promise = vid.play();
        if let Ok(promise) = promise {
            wasm_bindgen_futures::spawn_local(async move {
                let rr = wasm_bindgen_futures::JsFuture::from(promise).await;
                if let Err(e) = rr {
                    logging::error!("promise failed: {e:?}");
                }
            });
        } else {
            logging::error!("Failed to play video");
        }
    });

    let post = Signal::derive(move || video_queue.with(|q| q.get_index(idx).cloned()));

    view! { <VideoView post _ref=container_ref muted /> }.into_any()
}

#[component]
pub fn OnboardingWelcomePopup(show: RwSignal<bool>) -> impl IntoView {
    view! {
        <ShadowOverlay show=show >
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div class="overflow-hidden h-fit max-w-md items-center pt-16 cursor-auto bg-neutral-950 rounded-md w-full relative">
                    <img src="/img/common/refer-bg.webp" class="absolute inset-0 z-0 w-full h-full object-cover opacity-40" />
                    <div
                        style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                        class="absolute z-[1] -left-1/2 bottom-1/3 size-[32rem]" >
                    </div>
                    <button
                        on:click=move |_| show.set(false)
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                    <div class="flex z-[2] flex-col items-center gap-16 text-white justify-center p-12">
                        <img src="/img/hotornot/onboarding-welcome.webp" class="h-60" />
                        <div class="text-center text-2xl font-semibold">Bitcoin credited to<br/> your wallet!</div>
                        <div class="text-center">
                            "You've got free "<span class="font-semibold">Bitcoin (100 SATS)</span>.
                            <br/>
                            "Here's how to make it grow"
                        </div>
                        <HighlightedButton
                            alt_style=false
                            disabled=false
                            on_click=move || { show.set(false) }
                        >
                            "Start Playing"
                        </HighlightedButton>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
