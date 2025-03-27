mod bet;
pub mod error;
pub mod overlay;
pub mod single_post;
pub mod video_iter;
pub mod video_loader;
use crate::scrolling_post_view::ScrollingPostView;
use component::spinner::FullScreenSpinner;
use consts::NSFW_TOGGLE_STORE;
use priority_queue::DoublePriorityQueue;
use state::canisters::{authenticated_canisters, unauth_canisters};
use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
};
use yral_types::post::PostItem;
use indexmap::IndexSet;

use candid::Principal;
use codee::string::FromToStringCodec;
use futures::StreamExt;
use leptos::prelude::*;
use leptos_router::{
    hooks::{use_navigate, use_params},
    params::Params,
};
use leptos_use::{storage::use_local_storage, use_debounce_fn};
use utils::{
    posts::FetchCursor, route::failure_redirect, send_wrap, try_or_redirect, types::PostId,
};

use video_iter::{FeedResultType, VideoFetchStream};
use yral_canisters_common::{utils::posts::PostDetails, Canisters};

#[derive(Params, PartialEq, Clone, Copy)]
struct PostParams {
    canister_id: Principal,
    post_id: u64,
}

#[derive(Clone, Default)]
pub struct BetEligiblePostCtx {
    // This is true if betting is enabled for the current post and no bet has been placed
    pub can_place_bet: RwSignal<bool>,
}

#[derive(Clone, Default)]
pub struct PostViewCtx {
    fetch_cursor: RwSignal<FetchCursor>,
    // TODO: this is a dead simple with no GC
    // We're using virtual lists for DOM, so this doesn't consume much memory
    // as uids only occupy 32 bytes each
    // but ideally this should be cleaned up
    video_queue: RwSignal<IndexSet<PostDetails>>,
    current_idx: RwSignal<usize>,
    queue_end: RwSignal<bool>,
    priority_q: RwSignal<DoublePriorityQueue<PostDetails, (usize, Reverse<usize>)>>, // we are using DoublePriorityQueue for GC in the future through pop_min
    batch_cnt: RwSignal<usize>,
}

#[derive(Clone, Default)]
pub struct PostDetailsCacheCtx {
    pub post_details: RwSignal<HashMap<PostId, PostItem>>,
}

#[component]
pub fn CommonPostViewWithUpdates<S: Storage<ArcAction<(), ()>>>(
    initial_post: Option<PostDetails>,
    fetch_video_action: Action<(), (), S>,
    threshold_trigger_fetch: usize,
) -> impl IntoView {
    let PostViewCtx {
        fetch_cursor,
        video_queue,
        current_idx,
        queue_end,
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
                v.drain(rem);
                return;
            }
            *v = IndexSet::new();
            v.insert(initial_post);
        })
    }

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
        use_navigate()(
            &format!("/hot-or-not/{canister_id}/{post_id}",),
            Default::default(),
        );
    });

    view! {
        <ScrollingPostView
            video_queue
            current_idx
            recovering_state
            fetch_next_videos=next_videos
            queue_end
            threshold_trigger_fetch
        />
    }
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
        ..
    } = expect_context();

    let auth_cans = authenticated_canisters();

    let fetch_video_action: Action<_, _, LocalStorage> = Action::new_local(move |_| {
        let auth_cans = auth_cans;
        let (nsfw_enabled, _, _) = use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
        async move {
            {
                let mut prio_q = priority_q.write();
                let mut cnt = 0;
                while let Some((next, _)) = prio_q.pop_max() {
                    video_queue.update(|vq| {
                        if vq.insert(next) {
                            cnt += 1;
                        }
                    });
                    if cnt >= 25 {
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

                let canisters = auth_cans.await;
                let cans_true = Canisters::from_wire(canisters.unwrap(), expect_context()).unwrap();

                let mut fetch_stream = VideoFetchStream::new(&cans_true, cursor);
                let chunks = fetch_stream
                    .fetch_post_uids_hybrid(3, nsfw_enabled, video_queue.get_untracked().iter().cloned().collect())
                    .await;

                let res = try_or_redirect!(chunks);
                let mut chunks = res.posts_stream;
                let mut cnt = 0usize;
                while let Some(chunk) = chunks.next().await {
                    for uid in chunk {
                        let post_detail = try_or_redirect!(uid);
                        if video_queue
                            .with_untracked(|vq| vq.len())
                            .saturating_sub(current_idx.get_untracked())
                            <= 25
                        {
                            video_queue.update(|vq| {
                                let _ = vq.insert(post_detail);
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
        }
    });

    view! { <CommonPostViewWithUpdates initial_post fetch_video_action threshold_trigger_fetch=50 /> }
}

#[component]
pub fn PostView() -> impl IntoView {
    let params = use_params::<PostParams>();
    let initial_canister_and_post = RwSignal::new(params.get_untracked().ok());

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

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let initial_post = fetch_first_video_uid.await.ok()?;
                { Some(view! { <PostViewWithUpdatesMLFeed initial_post /> }) }
            })}
        </Suspense>
    }
    .into_any()
}
