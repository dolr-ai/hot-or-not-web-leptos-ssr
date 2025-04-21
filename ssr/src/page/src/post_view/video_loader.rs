use std::cmp::Ordering;

use codee::string::FromToStringCodec;
use indexmap::IndexSet;
use leptos::ev;
use leptos::{html::Video, prelude::*};
use leptos_use::storage::use_local_storage;
use leptos_use::use_event_listener;
use state::canisters::unauth_canisters;
use utils::send_wrap;
use yral_canisters_client::individual_user_template::PostViewDetailsFromFrontend;

use crate::post_view::BetEligiblePostCtx;
use component::show_any::ShowAny;
use component::{
    feed_popup::FeedPopUp, onboarding_flow::OnboardingPopUp, video_player::VideoPlayer,
};
use consts::USER_ONBOARDING_STORE;
use state::local_storage::use_referrer_store;
use utils::event_streaming::events::VideoWatched;
use utils::{bg_url, event_streaming::events::account_connected_reader, mp4_url};

use super::{overlay::VideoDetailsOverlay, PostDetails};

#[component]
pub fn BgView(
    video_queue: RwSignal<IndexSet<PostDetails>>,
    current_idx: RwSignal<usize>,
    idx: usize,
    children: Children,
) -> impl IntoView {
    let post = Memo::new(move |_| video_queue.with(|q| q.get_index(idx).cloned()));
    let uid = move || post().as_ref().map(|q| q.uid.clone()).unwrap_or_default();

    let (is_connected, _) = account_connected_reader();
    let (show_login_popup, set_show_login_popup) = signal(true);

    let (show_refer_login_popup, set_show_refer_login_popup) = signal(true);
    let (referrer_store, _, _) = use_referrer_store();

    let onboarding_eligible_post_context = BetEligiblePostCtx::default();
    provide_context(onboarding_eligible_post_context.clone());

    let (show_onboarding_popup, set_show_onboarding_popup) = signal(false);
    let (is_onboarded, set_onboarded, _) =
        use_local_storage::<bool, FromToStringCodec>(USER_ONBOARDING_STORE);

    Effect::new(move |_| {
        if current_idx.get() % 5 != 0 {
            set_show_login_popup.update(|n| *n = false);
        } else {
            set_show_login_popup.update(|n| *n = true);
        }
        Some(())
    });

    Effect::new(move |_| {
        if onboarding_eligible_post_context.can_place_bet.get() && (!is_onboarded.get()) {
            set_show_onboarding_popup.update(|show| *show = true);
        } else {
            set_show_onboarding_popup.update(|show| *show = false);
        }
    });

    view! {
        <div class="bg-transparent w-full h-full relative overflow-hidden">
            <div
                class="absolute top-0 left-0 bg-cover bg-center w-full h-full z-[1] blur-lg"
                style:background-color="rgb(0, 0, 0)"
                style:background-image=move || format!("url({})", bg_url(uid()))
            ></div>
            <ShowAny when=move || {
                current_idx.get() != 0 && current_idx.get() % 5 == 0 && !is_connected.get()
                    && show_login_popup.get()
            }>
                <FeedPopUp
                    on_click=move |_| set_show_login_popup.set(false)
                    header_text="Your 1000 COYNs
                    Await You!"
                    body_text="SignUp/Login to save your progress and claim your rewards."
                    login_text="Login"
                />
            </ShowAny>
            <ShowAny when=move || {
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
            </ShowAny>
            <ShowAny when=move || { show_onboarding_popup.get() }>
                <OnboardingPopUp onboard_on_click=set_onboarded />
            </ShowAny>
            {move || post().map(|post| view! { <VideoDetailsOverlay post /> })}
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
    let uid = Memo::new(move |_| post_for_uid.with(|p| p.as_ref().map(|p| p.uid.clone())));
    let view_bg_url = move || uid().map(bg_url);
    let view_video_url = move || uid().map(mp4_url);

    // Handles mute/unmute
    Effect::new(move |_| {
        let vid = _ref.get()?;
        vid.set_muted(muted());
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
                    log::warn!("failed to send view details: {:?}", err);
                }
                Some(())
            })
        });

    let video_views_watch_multiple = RwSignal::new(false);

    let _ = use_event_listener(_ref, ev::pause, move |_evt| {
        let Some(video) = _ref.get() else {
            return;
        };

        let duration = video.duration();
        let current_time = video.current_time();
        if current_time < 0.5 {
            return;
        }

        let percentage_watched = ((current_time / duration) * 100.0) as u8;
        send_view_detail_action.dispatch((percentage_watched, 0_u8));
    });

    let _ = use_event_listener(_ref, ev::timeupdate, move |_evt| {
        let Some(video) = _ref.get() else {
            return;
        };

        let duration = video.duration();
        let current_time = video.current_time();
        let percentage_watched = ((current_time / duration) * 100.0) as u8;

        if current_time < 0.95 * duration {
            video_views_watch_multiple.set(false);
        }

        if percentage_watched >= 95 && !video_views_watch_multiple.get() {
            send_view_detail_action.dispatch((percentage_watched, 0_u8));

            video_views_watch_multiple.set(true);
        }
    });

    VideoWatched.send_event(post, _ref);

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
        _ = vid.play();
    });

    let post = Signal::derive(move || video_queue.with(|q| q.get_index(idx).cloned()));

    view! { <VideoView post _ref=container_ref muted /> }.into_any()
}
