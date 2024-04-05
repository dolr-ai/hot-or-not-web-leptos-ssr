use super::{overlay::VideoDetailsOverlay, PostViewCtx};
use crate::{
    canister::{
        individual_user_template::PostViewDetailsFromFrontend,
        utils::{bg_url, mp4_url},
    },
    component::feed_popup::FeedPopUp,
    state::{
        auth::account_connected_reader,
        canisters::{unauth_canisters, AuthProfileCanisterResource},
        local_storage::use_referrer_store,
    },
    utils::event_streaming::send_event_warehouse,
};
use leptos::{html::Video, *};
use leptos_use::{use_event_listener, watch_debounced};

use crate::utils::event_streaming::send_event;
use serde_json::json;
use wasm_bindgen::JsCast;

#[component]
pub fn BgView(idx: usize, children: Children) -> impl IntoView {
    let PostViewCtx {
        video_queue,
        current_idx,
        ..
    } = expect_context();
    let post = move || video_queue.with(|q| q.get(idx).cloned());
    let uid = move || post().as_ref().map(|q| q.uid.clone()).unwrap_or_default();

    let (is_connected, _) = account_connected_reader();
    let (show_login_popup, set_show_login_popup) = create_signal(true);

    let (show_refer_login_popup, set_show_refer_login_popup) = create_signal(true);
    let (referrer_store, _, _) = use_referrer_store();

    create_effect(move |_| {
        if current_idx.get() == 5 {
            set_show_login_popup.update(|n| *n = false);
        }
        Some(())
    });

    view! {
        <div class="bg-transparent w-full h-full relative overflow-hidden">
            <div
                class="absolute top-0 left-0 bg-cover bg-center w-full h-full z-[1] blur-lg"
                style:background-color="rgb(0, 0, 0)"
                style:background-image=move || format!("url({})", bg_url(uid()))
            ></div>
            <Show when=move || { idx == 4 && !is_connected.get() && show_login_popup.get() }>
                <FeedPopUp
                    on_click=move |_| set_show_login_popup.set(false)
                    header_text="Your Rewards are
                    Waiting!"
                    body_text="SignUp/Login to save your progress and claim your rewards."
                    login_text="Login"
                />
            </Show>
            <Show when=move || {
                referrer_store.get().is_some() && idx == 0 && !is_connected.get()
                    && show_refer_login_popup.get()
            }>
                <FeedPopUp
                    on_click=move |_| set_show_refer_login_popup.set(false)
                    header_text="Claim Your Referral
                    Rewards Now!"
                    body_text="SignUp from this link to get 500 COYNs as referral rewards."
                    login_text="Sign Up"
                />
            </Show>
            {move || post().map(|post| view! { <VideoDetailsOverlay post/> })}
            {children()}
        </div>
    }
}

#[component]
pub fn VideoView(idx: usize, muted: RwSignal<bool>) -> impl IntoView {
    let container_ref = create_node_ref::<Video>();
    let PostViewCtx {
        video_queue,
        current_idx,
        ..
    } = expect_context();

    let vid_details = create_memo(move |_| with!(|video_queue| video_queue.get(idx).cloned()));

    let uid =
        create_memo(move |_| with!(|video_queue| video_queue.get(idx).map(|q| q.uid.clone())));
    let view_bg_url = move || uid().map(bg_url);
    let view_video_url = move || uid().map(mp4_url);

    // Handles autoplay
    create_effect(move |_| {
        let Some(vid) = container_ref() else {
            return;
        };
        if idx != current_idx() {
            _ = vid.pause();
            return;
        }
        vid.set_autoplay(true);
        _ = vid.play();
    });

    // Handles mute/unmute
    create_effect(move |_| {
        let vid = container_ref()?;
        vid.set_muted(muted());
        Some(())
    });

    create_effect(move |_| {
        let vid = container_ref()?;
        // the attributes in DOM don't seem to be working
        vid.set_muted(muted.get_untracked());
        vid.set_loop(true);
        Some(())
    });

    let watched_percentage = create_rw_signal(0_u8);
    let watched_count = create_rw_signal(0_u8);

    let video_previous_current_time = store_value(0.0);
    let _ = use_event_listener(container_ref, ev::timeupdate, move |_event| {
        let Some(video) = container_ref() else {
            return;
        };

        let duration = video.duration();
        let current_time = video.current_time();

        if current_time < video_previous_current_time() {
            watched_count.update(|wc| *wc += 1);
        }
        watched_percentage.update(|watched_percentage| {
            *watched_percentage = (100.0 * (current_time / duration)) as u8;
        });

        video_previous_current_time.update_value(|v| *v = current_time);
    });

    let post = Signal::derive(move || video_queue.with(|q| q.get(idx).cloned()));

    let send_view_detail_action = create_action(move |()| async move {
        let canisters = unauth_canisters();
        let current_watched_percentage = watched_percentage.get_untracked();
        let current_watched_count = watched_count.get_untracked();
        let payload = match current_watched_count {
            0 => PostViewDetailsFromFrontend::WatchedPartially {
                percentage_watched: current_watched_percentage,
            },
            _ => PostViewDetailsFromFrontend::WatchedMultipleTimes {
                percentage_watched: current_watched_percentage,
                watch_count: current_watched_count,
            },
        };
        watched_count.update(|wc| *wc = 0);
        watched_percentage.update(|watched_percentage| *watched_percentage = 0);
        let post_id = post.get_untracked().as_ref().map(|p| p.post_id).unwrap();
        let canister_id = post
            .get_untracked()
            .as_ref()
            .map(|p| p.canister_id)
            .unwrap();
        let send_view_res = canisters
            .individual_user(canister_id)
            .update_post_add_view_details(post_id, payload)
            .await;

        if let Err(err) = send_view_res {
            log::warn!("failed to send view details: {:?}", err);
        }
    });

    let send_view_details_guard = create_memo(move |_| {
        current_idx() != idx && (watched_percentage() != 0 || watched_count() != 0)
    });

    let _ = watch_debounced(
        watched_percentage,
        move |_, _, _| {
            if send_view_details_guard() {
                send_view_detail_action.dispatch(());
            }
        },
        2000.0,
    );
    #[cfg(all(feature = "hydrate", feature = "ga4"))]
    {
        let profile_and_canister_details: AuthProfileCanisterResource = expect_context();
        let (is_connected, _) = account_connected_reader();

        let publisher_user_id = move || vid_details().as_ref().map(|q| q.poster_principal);
        let user_id = move || {
            profile_and_canister_details()
                .flatten()
                .map(|(q, _)| q.principal)
        };
        let display_name = move || {
            profile_and_canister_details()
                .flatten()
                .map(|(q, _)| q.display_name)
        };
        let canister_id = move || profile_and_canister_details().flatten().map(|(_, q)| q);
        let video_id = move || vid_details().as_ref().map(|q| q.uid.clone());
        let hastag_count = move || vid_details().as_ref().map(|q| q.hastags.len());
        let is_nsfw = move || vid_details().as_ref().map(|q| q.is_nsfw);
        let is_hotornot = move || {
            vid_details()
                .as_ref()
                .map(|q| q.hot_or_not_feed_ranking_score.is_some())
        };
        let view_count = move || vid_details().as_ref().map(|q| q.views);
        let like_count = move || vid_details().as_ref().map(|q| q.likes);

        // video_viewed - analytics
        let (video_watched, set_video_watched) = create_signal(false);
        let (full_video_watched, set_full_video_watched) = create_signal(false);

        let _ = use_event_listener(container_ref, ev::timeupdate, move |evt| {
            let target = evt.target().unwrap();
            let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
            let duration = video.duration();
            let current_time = video.current_time();

            if current_time < 0.95 * duration {
                set_full_video_watched.set(false);
            }

            // send bigquery event when video is watched > 95%
            if current_time >= 0.95 * duration && !full_video_watched.get() {
                send_event_warehouse(
                    "video_duration_watched",
                    &json!({
                        "publisher_user_id":publisher_user_id(),
                        "user_id":user_id(),
                        "is_loggedIn": is_connected(),
                        "display_name": display_name(),
                        "canister_id": canister_id(),
                        "video_id": video_id(),
                        "video_category": "NA",
                        "creator_category": "NA",
                        "hashtag_count": hastag_count(),
                        "is_NSFW": is_nsfw(),
                        "is_hotorNot": is_hotornot(),
                        "feed_type": "NA",
                        "view_count": view_count(),
                        "like_count": like_count(),
                        "share_count": 0,
                        "percentage_watched": 100.0,
                        "absolute_watched": duration,
                        "video_duration": duration,
                    }),
                );

                set_full_video_watched.set(true);
            }

            if video_watched.get() {
                return;
            }

            if current_time >= 3.0 {
                // Video is halfway done, take action here

                send_event(
                    "video_viewed",
                    &json!({
                        "publisher_user_id":publisher_user_id(),
                        "user_id":user_id(),
                        "is_loggedIn": is_connected(),
                        "display_name": display_name(),
                        "canister_id": canister_id(),
                        "video_id": video_id(),
                        "video_category": "NA",
                        "creator_category": "NA",
                        "hashtag_count": hastag_count(),
                        "is_NSFW": is_nsfw(),
                        "is_hotorNot": is_hotornot(),
                        "feed_type": "NA",
                        "view_count": view_count(),
                        "like_count": like_count(),
                        "share_count": 0,
                    }),
                );
                set_video_watched.set(true);
            }
        });

        // video duration watched - warehousing

        let _ = use_event_listener(container_ref, ev::pause, move |evt| {
            let target = evt.target().unwrap();
            let video = target.unchecked_into::<web_sys::HtmlVideoElement>();
            let duration = video.duration();
            let current_time = video.current_time();
            if current_time < 1.0 {
                return;
            }

            let percentage_watched = (current_time / duration) * 100.0;

            send_event_warehouse(
                "video_duration_watched",
                &json!({
                    "publisher_user_id":publisher_user_id(),
                    "user_id":user_id(),
                    "is_loggedIn": is_connected(),
                    "display_name": display_name(),
                    "canister_id": canister_id(),
                    "video_id": video_id(),
                    "video_category": "NA",
                    "creator_category": "NA",
                    "hashtag_count": hastag_count(),
                    "is_NSFW": is_nsfw(),
                    "is_hotorNot": is_hotornot(),
                    "feed_type": "NA",
                    "view_count": view_count(),
                    "like_count": like_count(),
                    "share_count": 0,
                    "percentage_watched": percentage_watched,
                    "absolute_watched": current_time,
                    "video_duration": duration,
                }),
            );
        });
    }

    view! {
        <Suspense>
            <label class="w-full h-full absolute top-0 left-0 grid grid-cols-1 justify-items-center items-center cursor-pointer z-[3]">
                <input
                    on:change=move |_| muted.update(|m| *m = !*m)
                    type="checkbox"
                    value=""
                    class="sr-only"
                />
                <video
                    _ref=container_ref
                    class="object-contain h-dvh max-h-dvh cursor-pointer"
                    poster=view_bg_url
                    src=view_video_url
                    loop
                    muted
                    playsinline
                    disablepictureinpicture
                    disableremoteplayback
                    preload="auto"
                ></video>
            </label>
        </Suspense>
    }
}
