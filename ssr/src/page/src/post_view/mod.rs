mod bet;
pub mod error;
pub mod overlay;
pub mod single_post;
pub mod video_iter;
pub mod video_loader;
use crate::scrolling_post_view::ScrollingPostView;
use component::buttons::HighlightedButton;
use component::overlay::ShadowOverlay;
use component::spinner::FullScreenSpinner;
use consts::{
    UserOnboardingStore, MAX_VIDEO_ELEMENTS_FOR_FEED, NSFW_TOGGLE_STORE, USER_ONBOARDING_STORE_KEY,
};
use indexmap::IndexSet;
use leptos_icons::*;
use priority_queue::DoublePriorityQueue;
use state::canisters::{auth_state, unauth_canisters};
use std::{cmp::Reverse, collections::HashMap};
use yral_types::post::PostItem;

use candid::Principal;
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use futures::StreamExt;
use leptos::prelude::*;
use leptos_router::{
    hooks::{use_navigate, use_params},
    params::Params,
};
use leptos_use::{storage::use_local_storage, use_debounce_fn};
use utils::{
    mixpanel::mixpanel_events::*,
    posts::{FeedPostCtx, FetchCursor},
    route::failure_redirect,
    send_wrap, try_or_redirect,
    types::PostId,
};

use video_iter::{new_video_fetch_stream, new_video_fetch_stream_auth, FeedResultType};
use yral_canisters_common::{utils::posts::PostDetails, Canisters};

#[derive(Params, PartialEq, Clone, Copy)]
struct PostParams {
    canister_id: Principal,
    post_id: u64,
}

#[derive(Clone, Default)]
pub struct PostViewCtx {
    fetch_cursor: RwSignal<FetchCursor>,
    // TODO: this is a dead simple with no GC
    // We're using virtual lists for DOM, so this doesn't consume much memory
    // as uids only occupy 32 bytes each
    // but ideally this should be cleaned up
    video_queue: RwSignal<IndexSet<PostDetails>>,
    video_queue_for_feed: RwSignal<Vec<FeedPostCtx>>,
    current_idx: RwSignal<usize>,
    queue_end: RwSignal<bool>,
    priority_q: RwSignal<DoublePriorityQueue<PostDetails, (usize, Reverse<usize>)>>, // we are using DoublePriorityQueue for GC in the future through pop_min
    batch_cnt: RwSignal<usize>,
}

impl PostViewCtx {
    pub fn new() -> Self {
        let mut video_queue_for_feed = Vec::new();
        for i in 0..MAX_VIDEO_ELEMENTS_FOR_FEED {
            video_queue_for_feed.push(FeedPostCtx {
                key: i,
                value: RwSignal::new(None),
            });
        }

        Self {
            video_queue_for_feed: RwSignal::new(video_queue_for_feed),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default)]
pub struct PostDetailsCacheCtx {
    pub post_details: RwSignal<HashMap<PostId, PostItem>>,
}

#[component]
pub fn CommonPostViewWithUpdates(
    initial_post: Option<PostDetails>,
    fetch_video_action: Action<(), ()>,
    threshold_trigger_fetch: usize,
) -> impl IntoView {
    let PostViewCtx {
        fetch_cursor,
        video_queue,
        current_idx,
        queue_end,
        video_queue_for_feed,
        ..
    } = expect_context();

    let recovering_state = RwSignal::new(false);
    if let Some(initial_post) = initial_post.clone() {
        fetch_cursor.update_untracked(|f| {
            // we've already fetched the first posts
            if f.start > 1 || queue_end.get_untracked() {
                recovering_state.set(true);
                return;
            }
            f.start = 1;
        });
        video_queue.update_untracked(|v| {
            if v.len() > 1 {
                // Safe to do a GC here
                let rem = 0..(current_idx.get_untracked().saturating_sub(6));
                current_idx.update(|c| *c -= rem.len());
                v.drain(rem.clone());
                video_queue_for_feed.update_untracked(|vqf| {
                    vqf.drain(rem);
                });
                return;
            }
            *v = IndexSet::new();
            v.insert(initial_post.clone());
            video_queue_for_feed.update(|vq| {
                vq[0].value.set(Some(initial_post.clone()));
            });
        })
    }

    let current_post_params: RwSignal<Option<utils::types::PostParams>> = expect_context();

    Effect::new(move || {
        if !recovering_state.get_untracked() {
            fetch_video_action.dispatch(());
        }
    });
    let next_videos = use_debounce_fn(
        move || {
            if !fetch_video_action.pending().get_untracked() && !queue_end.get_untracked() {
                fetch_video_action.dispatch(());
            }
        },
        200.0,
    );

    let current_post_base = Memo::new(move |_| {
        video_queue.with(|q| {
            let cur_idx = current_idx();
            let details = q.get_index(cur_idx)?;
            Some((details.canister_id, details.post_id))
        })
    });

    Effect::new(move || {
        let Some((canister_id, post_id)) = current_post_base() else {
            return;
        };
        current_post_params.set(Some(utils::types::PostParams {
            canister_id,
            post_id,
        }));
        use_navigate()(
            &format!("/hot-or-not/{canister_id}/{post_id}",),
            Default::default(),
        );
    });

    let hard_refresh_target = RwSignal::new("/".to_string());

    view! {
        <ScrollingPostView
            video_queue
            video_queue_for_feed
            current_idx
            recovering_state
            fetch_next_videos=next_videos
            queue_end
            threshold_trigger_fetch
            hard_refresh_target
        />
    }
    .into_any()
}

#[component]
pub fn PostViewWithUpdatesMLFeed(initial_post: Option<PostDetails>) -> impl IntoView {
    let PostViewCtx {
        fetch_cursor,
        video_queue,
        queue_end,
        priority_q,
        batch_cnt,
        current_idx,
        video_queue_for_feed,
        ..
    } = expect_context();

    let auth = auth_state();

    let fetch_video_action = Action::new(move |_| {
        let (nsfw_enabled, _, _) = use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
        #[cfg(not(feature = "hydrate"))]
        {
            return async {};
        }

        #[cfg(feature = "hydrate")]
        send_wrap(async move {
            {
                let mut prio_q = priority_q.write();
                let mut cnt = 0;
                while let Some((next, _)) = prio_q.pop_max() {
                    video_queue.update(|vq| {
                        if vq.insert(next.clone()) {
                            let len_vq = vq.len();
                            if len_vq > MAX_VIDEO_ELEMENTS_FOR_FEED {
                                return;
                            }

                            video_queue_for_feed.update(|vqf| {
                                vqf[len_vq - 1].value.set(Some(next.clone()));
                            });
                            cnt += 1;
                        }
                    });
                    if cnt >= 10 {
                        break;
                    }
                }
            }

            // backfill PQ from ML feed server
            // fetch to video_queue based on threshold
            if priority_q.with_untracked(|q| q.len()) < 100 {
                let Some(cursor) = fetch_cursor.try_get_untracked() else {
                    return;
                };
                let Some(nsfw_enabled) = nsfw_enabled.try_get_untracked() else {
                    return;
                };
                let Some(batch_cnt_val) = batch_cnt.try_get_untracked() else {
                    return;
                };
                leptos::logging::log!("fetching ml feed");
                let cans_false: Canisters<false> = unauth_canisters();
                let cans_true = auth.auth_cans_if_available(cans_false.clone());

                let video_queue_c = video_queue.get_untracked().iter().cloned().collect();
                let chunks = if let Some(cans_true) = cans_true.as_ref() {
                    let mut fetch_stream = new_video_fetch_stream_auth(cans_true, auth, cursor);
                    fetch_stream
                        .fetch_post_uids_hybrid(3, nsfw_enabled, video_queue_c)
                        .await
                } else {
                    let mut fetch_stream = new_video_fetch_stream(&cans_false, auth, cursor);
                    fetch_stream
                        .fetch_post_uids_hybrid(3, nsfw_enabled, video_queue_c)
                        .await
                };

                let res = try_or_redirect!(chunks);
                let mut chunks = res.posts_stream;
                let mut cnt = 0usize;
                while let Some(chunk) = chunks.next().await {
                    for uid in chunk {
                        let post_detail = try_or_redirect!(uid);
                        if video_queue
                            .with_untracked(|vq| vq.len())
                            .saturating_sub(current_idx.get_untracked())
                            <= 10
                        {
                            video_queue.update(|vq| {
                                if vq.insert(post_detail.clone()) {
                                    let len_vq = vq.len();
                                    if len_vq > MAX_VIDEO_ELEMENTS_FOR_FEED {
                                        return;
                                    }
                                    video_queue_for_feed.update(|vqf| {
                                        vqf[len_vq - 1].value.set(Some(post_detail.clone()));
                                    });
                                }
                            });
                        } else {
                            priority_q.update(|pq| {
                                pq.push(post_detail, (batch_cnt_val, Reverse(cnt)));
                            });
                        }
                        cnt += 1;
                    }
                }

                leptos::logging::log!("feed type: {:?} cnt {}", res.res_type, cnt); // For debugging purposes
                if res.res_type != FeedResultType::MLFeed {
                    fetch_cursor.try_update(|c| {
                        c.set_limit(50);
                        c.advance_and_set_limit(50)
                    });
                }

                if res.end {
                    queue_end.try_set(res.end);
                }

                batch_cnt.update(|x| *x += 1);
            }
        })
    });

    view! { <CommonPostViewWithUpdates initial_post fetch_video_action threshold_trigger_fetch=20 /> }.into_any()
}

#[component]
pub fn PostView() -> impl IntoView {
    let params = use_params::<PostParams>();
    let initial_canister_and_post = RwSignal::new(params.get_untracked().ok());
    let home_page_viewed_sent = RwSignal::new(false);
    let auth = auth_state();
    let (nsfw_enabled, _, _) = use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
    Effect::new(move |_| {
        if home_page_viewed_sent.get_untracked() {
            return;
        }
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(
            auth.event_ctx(),
            nsfw_enabled.get_untracked(),
        ) {
            MixPanelEvent::track_home_page_viewed(MixpanelBottomBarPageViewedProps {
                user_id: global.user_id,
                visitor_id: global.visitor_id,
                is_logged_in: global.is_logged_in,
                canister_id: global.canister_id,
                is_nsfw_enabled: global.is_nsfw_enabled,
            });
            home_page_viewed_sent.set(true);
        }
    });
    Effect::new_isomorphic(move |_| {
        if initial_canister_and_post.with_untracked(|p| p.is_some()) {
            return None;
        }
        let p = params.get().ok()?;
        initial_canister_and_post.set(Some(p));
        Some(())
    });

    let PostViewCtx {
        video_queue,
        current_idx,
        ..
    } = expect_context();
    let canisters = unauth_canisters();
    let post_details_cache: PostDetailsCacheCtx = expect_context();

    let fetch_first_video_uid = Resource::new(initial_canister_and_post, move |params| {
        let canisters = canisters.clone();
        async move {
            let Some(params) = params else {
                return Err(());
            };
            let cached_post = video_queue
                .with_untracked(|q| q.get_index(current_idx.get_untracked()).cloned())
                .filter(|post| {
                    post.canister_id == params.canister_id && post.post_id == params.post_id
                });
            if let Some(post) = cached_post {
                return Ok(Some(post));
            }
            let post_nsfw_prob = post_details_cache.post_details.with_untracked(|p| {
                let item = p.get(&(params.canister_id, params.post_id));
                if let Some(item) = item {
                    item.nsfw_probability
                } else {
                    1.0 // TODO: handle this for when we don't have details (when user shares video)
                }
            });

            match send_wrap(canisters.get_post_details_with_nsfw_info(
                params.canister_id,
                params.post_id,
                post_nsfw_prob,
            ))
            .await
            {
                Ok(post) => Ok(post),
                Err(e) => {
                    failure_redirect(e);
                    Err(())
                }
            }
        }
    });

    let (onboarding_store, set_onboarding_store, _) =
        use_local_storage::<UserOnboardingStore, JsonSerdeCodec>(USER_ONBOARDING_STORE_KEY);

    let show_onboarding_popup = RwSignal::new(false);

    let close_onboarding_action = Action::new(move |_: &()| {
        set_onboarding_store.update(|store| {
            store.has_seen_onboarding = true;
        });
        show_onboarding_popup.set(false);
        async move {}
    });

    Effect::new(move |_| {
        if !(onboarding_store.get_untracked().has_seen_onboarding)
            && !auth.is_logged_in_with_oauth().get_untracked()
        {
            show_onboarding_popup.set(true);
        }
    });

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let initial_post = fetch_first_video_uid.await.ok()?;
                { Some(view! { <PostViewWithUpdatesMLFeed initial_post /> }.into_any()) }
            })}
        </Suspense>
        <OnboardingWelcomePopup show=show_onboarding_popup close_action=close_onboarding_action />
    }
    .into_any()
}

#[component]
pub fn OnboardingWelcomePopup(show: RwSignal<bool>, close_action: Action<(), ()>) -> impl IntoView {
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    const CREDITED_AMOUNT: u64 = limits::NEW_USER_SIGNUP_REWARD_SATS;
    Effect::new(move || {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_onboarding_popup(MixpanelOnboardingPopupViewProps {
                user_id: global.user_id,
                visitor_id: global.visitor_id,
                is_logged_in: global.is_logged_in,
                canister_id: global.canister_id,
                is_nsfw_enabled: global.is_nsfw_enabled,
                credited_amount: CREDITED_AMOUNT,
                popup_type: MixpanelOnboardingPopupType::SatsCreditPopup,
            });
        }
    });
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
                        on:click=move |_| {
                            close_action.dispatch(());
                        }
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                    <div class="flex z-[2] relative flex-col items-center gap-4 text-white justify-center p-12">
                        <img src="/img/hotornot/onboarding-welcome.webp" class="h-60" />
                        <div class="text-center text-2xl font-semibold">Bitcoin credited to<br/> your wallet!</div>
                        <div class="text-center">
                            "You've got free "<span class="font-semibold">{format!("Bitcoin ({CREDITED_AMOUNT} SATS)")}</span>.
                            <br/>
                            "Here's how to make it grow"
                        </div>
                        <HighlightedButton
                            alt_style=false
                            disabled=false
                            on_click=move || { close_action.dispatch(()); }
                        >
                            "Start Playing"
                        </HighlightedButton>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
