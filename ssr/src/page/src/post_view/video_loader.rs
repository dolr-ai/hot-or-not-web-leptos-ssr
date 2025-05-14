use std::cmp::Ordering;

use codee::string::FromToStringCodec;
use indexmap::IndexSet;
use leptos::ev;
use leptos::{html::Video, prelude::*};
use leptos_use::storage::use_local_storage;
use leptos_use::use_event_listener;
use state::canisters::unauth_canisters;
use utils::mixpanel::mixpanel_events::{
    IsHotOrNot, MixPanelEvent, MixpanelVideoViewedProps, UserCanisterAndPrincipal,
};
use utils::send_wrap;
use yral_canisters_client::individual_user_template::PostViewDetailsFromFrontend;

use crate::post_view::BetEligiblePostCtx;
use component::show_any::ShowAny;
use component::{
    feed_popup::FeedPopUp, onboarding_flow::OnboardingPopUp, video_player::VideoPlayer,
};
use consts::USER_ONBOARDING_STORE;
use state::local_storage::use_referrer_store;
use utils::event_streaming::events::{auth_canisters_store, VideoWatched};
use utils::{bg_url, event_streaming::events::account_connected_reader, mp4_url};

use super::{overlay::VideoDetailsOverlay, PostDetails};

#[component]
pub fn BgView(
    video_queue: RwSignal<IndexSet<PostDetails>>,
    idx: usize,
    children: Children,
) -> impl IntoView {
    let post = Memo::new(move |_| video_queue.with(|q| q.get_index(idx).cloned()));
    let uid = move || post().as_ref().map(|q| q.uid.clone()).unwrap_or_default();

    let (is_connected, _) = account_connected_reader();

    let (show_refer_login_popup, set_show_refer_login_popup) = signal(true);
    let (referrer_store, _, _) = use_referrer_store();

    let onboarding_eligible_post_context = BetEligiblePostCtx::default();
    provide_context(onboarding_eligible_post_context.clone());

    let (show_onboarding_popup, set_show_onboarding_popup) = signal(false);
    let (is_onboarded, set_onboarded, _) =
        use_local_storage::<bool, FromToStringCodec>(USER_ONBOARDING_STORE);

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
                referrer_store.get().is_some() && idx == 0 && !is_connected.get()
                    && show_refer_login_popup.get()
            }>
                <FeedPopUp
                    on_click=move |_| set_show_refer_login_popup.set(false)
                    header_text="Claim Your Referral
                    Rewards Now!"
                    body_text="SignUp from this link to get 500 Cents as referral rewards."
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
        Action::new_local(move |(percentage_watched, watch_count): &(u8, u8)| {
            let percentage_watched = *percentage_watched;
            let watch_count = *watch_count;
            let post_for_view = post_for_view;

            async move {
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
            }
        });

    let playing_started = RwSignal::new(false);

    let _ = use_event_listener(_ref, ev::playing, move |_evt| {
        let Some(_) = _ref.get() else {
            return;
        };
        playing_started.set(true);
        send_view_detail_action.dispatch((100, 0_u8));
    });

    let canisters = auth_canisters_store();

    let mixpanel_send_view_event = Action::new(move |_| {
        send_wrap(async move {
            if let Some(cans) = canisters.get_untracked() {
                let post = post_for_view.get_untracked().unwrap();
                let user = UserCanisterAndPrincipal::try_get(&cans);
                let is_hot_or_not = expect_context::<IsHotOrNot>();
                let is_hot_or_not = is_hot_or_not.get(post.canister_id, post.post_id);
                MixPanelEvent::track_video_viewed(MixpanelVideoViewedProps {
                    publisher_user_id: post.poster_principal.to_text(),
                    is_logged_in: user.is_some(),
                    user_id: user.clone().map(|f| f.user_id),
                    canister_id: user.map(|f| f.canister_id),
                    video_id: post.uid,
                    is_nsfw: post.is_nsfw,
                    is_hotor_not: is_hot_or_not,
                    view_count: post.views,
                    like_count: post.likes,
                });
                playing_started.set(false);
            }
        })
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
