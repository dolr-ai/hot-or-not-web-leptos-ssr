use std::marker::PhantomData;

use candid::Principal;
use component::{back_btn::BackButton, spinner::FullScreenSpinner};
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use leptos::prelude::*;
use leptos_router::params::Params;
use leptos_router::{
    hooks::{use_navigate, use_params, use_query},
    *,
};
use leptos_use::use_debounce_fn;
use state::canisters::{auth_state, unauth_canisters};
use utils::{route::failure_redirect, send_wrap, try_or_redirect};

use crate::scrolling_post_view::ScrollingPostView;

use super::{
    overlay::YourProfileOverlay,
    profile_iter::{FixedFetchCursor, ProfVideoStream, ProfileVideoStream},
    ProfilePostsContext,
};

use yral_canisters_common::utils::posts::PostDetails;

#[component]
fn ProfilePostWithUpdates<const LIMIT: u64, VidStream: ProfVideoStream<LIMIT>>(
    initial_post: PostDetails,
    user_canister: Principal,
    #[prop(optional)] _stream_phantom: PhantomData<VidStream>,
) -> impl IntoView {
    let ProfilePostsContext {
        video_queue,
        video_queue_for_feed,
        start_index,
        current_index,
        queue_end,
    } = expect_context();
    let recovering_state = RwSignal::new(true);
    // Create a RwSignal for hard refresh target
    let hard_refresh_target = RwSignal::new("/".to_string());
    // Initialize cursor to fetch posts after the ones already in video_queue
    let initial_cursor_start =
        start_index.get_untracked() + video_queue.with_untracked(|vq| vq.len());
    let fetch_cursor = RwSignal::new(FixedFetchCursor::<LIMIT> {
        start: initial_cursor_start as u64,
        limit: 10,
    });
    let auth = auth_state();
    let overlay = move || {
        view! {
            <Suspense>
                {move || {
                    auth.user_canister
                        .get()
                        .map(|canister| {
                            if canister == Ok(initial_post.canister_id) {
                                Some(view! { <YourProfileOverlay /> }.into_any())
                            } else {
                                None
                            }
                        })
                }}
            </Suspense>
        }
    };

    // Handle initial post and recovering state
    if video_queue.with_untracked(|vq| vq.is_empty()) {
        video_queue.update_untracked(|vq| {
            let _ = vq.insert(initial_post.clone());
        });
        video_queue_for_feed.update(|vqf| {
            vqf[0].value.set(Some(initial_post.clone()));
        });
        recovering_state.set(false);
    } else {
        // We're recovering from a previous state
        recovering_state.set(true);
    }

    let next_videos: Action<_, _> = Action::new_unsync(move |_| {
        leptos::logging::log!("Fetching next videos");
        let hard_refresh_target = hard_refresh_target.clone();
        async move {
            let cursor = fetch_cursor.get_untracked();

            let canisters = unauth_canisters();
            let posts_res = if let Some(canisters) = auth.auth_cans_if_available(canisters.clone())
            {
                VidStream::fetch_next_posts(cursor, &canisters, user_canister).await
            } else {
                VidStream::fetch_next_posts(cursor, &canisters, user_canister).await
            };

            leptos::logging::log!("Fetched next videos");

            let res = try_or_redirect!(posts_res);

            queue_end.set(res.end);
            res.posts.into_iter().for_each(|p| {
                video_queue.try_update(|q| {
                    if q.insert(p.clone()) {
                        let len_vq = q.len();
                        if len_vq <= MAX_VIDEO_ELEMENTS_FOR_FEED {
                            video_queue_for_feed.update(|vqf| {
                                vqf[len_vq - 1].value.set(Some(p.clone()));
                            });
                            // Update hard refresh target to this post
                            let next_start_idx = start_index.get_untracked() + len_vq;
                            hard_refresh_target.set(
                                format!("/profile/{}/post/{}?next={}", p.canister_id, p.post_id, next_start_idx)
                            );
                        }
                    }
                });
            });
            leptos::logging::log!(
                "Updated video queue with new posts, current length: {}",
                video_queue.with_untracked(|q| q.len())
            );
            fetch_cursor.try_update(|c| {
                c.advance();
            });
        }
    });

    // Trigger initial fetch if not recovering state
    Effect::new(move || {
        if !recovering_state.get_untracked() && video_queue.with_untracked(|vq| vq.len()) <= 1 {
            next_videos.dispatch(());
        }
    });

    let fetch_next_videos = use_debounce_fn(
        move || {
            if !next_videos.pending().get_untracked() && !queue_end.get_untracked() {
                log::debug!("trigger rerender");
                next_videos.dispatch(());
            }
        },
        200.0,
    );

    let current_post_base = Memo::new(move |_| {
        video_queue.with(|q| {
            let curr_index = current_index();
            let details = q.get_index(curr_index);
            details.map(|d| (d.canister_id, d.post_id))
        })
    });

    Effect::new(move |_| {
        let Some((canister_id, post_id)) = current_post_base.get() else {
            return;
        };

        if recovering_state.get_untracked() {
            return;
        }

        use_navigate()(
            &format!("/profile/{canister_id}/post/{post_id}"),
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
    });

    view! {
        <ScrollingPostView
            video_queue
            video_queue_for_feed
            current_idx=current_index
            queue_end
            recovering_state
            fetch_next_videos
            overlay
            threshold_trigger_fetch=10
            hard_refresh_target
        />
    }
    .into_any()
}

#[component]
fn ProfilePostBase<
    IV: IntoView + 'static,
    C: Fn(PostDetails) -> IV + Clone + 'static + Send + Sync,
>(
    #[prop(into)] canister_and_post: Signal<Option<(Principal, u64)>>,
    #[prop(into)] next_start_idx: Signal<Option<usize>>,
    children: C,
) -> impl IntoView {
    let ProfilePostsContext {
        video_queue,
        video_queue_for_feed,
        current_index,
        start_index,
        ..
    } = expect_context();

    // Set start_index from the passed parameter if available
    Effect::new(move |_| {
        if let Some(next_idx) = next_start_idx.get() {
            start_index.set(next_idx);
        }
    });

    let intial_post = Resource::new(canister_and_post, move |params| {
        let canisters = unauth_canisters();
        send_wrap(async move {
            let Some((canister_id, post_id)) = params else {
                failure_redirect("Invalid profile post");
                return None;
            };

            let post_idx = video_queue.with_untracked(|vq| {
                vq.iter()
                    .position(|post| post.canister_id == canister_id && post.post_id == post_id)
            });

            let _ = video_queue.update(|vq| {
                leptos::logging::log!(
                    "Post index in video queue: {:?} for canister: {}, post_id: {} ; vide_q len : {:?}",
                    post_idx,
                    canister_id.to_string(),
                    post_id,
                    vq.len()
                );

                if let Some(idx) = post_idx {
                    // Remove all posts before the target post
                    if idx > 0 {
                        vq.drain(0..idx);

                        // Update start_index to account for the removed posts
                        start_index.update(|si| *si += idx);
                    }
                    
                    // Always update video_queue_for_feed to reflect the new state
                    video_queue_for_feed.update(|vqf| {
                        // Clear all entries
                        for i in 0..MAX_VIDEO_ELEMENTS_FOR_FEED {
                            vqf[i].value.set(None);
                        }
                        // Re-populate from the updated video_queue
                        for (i, post) in vq.iter().take(MAX_VIDEO_ELEMENTS_FOR_FEED).enumerate()
                        {
                            vqf[i].value.set(Some(post.clone()));
                        }
                    });
                    
                    // Now current_index is 0 since we removed all previous posts
                    current_index.set(0);

                    leptos::logging::log!(
                        "Retrieved post from video queue: {:?} for canister: {}, post_id: {} ; start_index: {:?}",
                        vq.get_index(0),
                        canister_id.to_string(),
                        post_id,
                        start_index.get_untracked()
                    );

                }
            });

            if let Some(post) = video_queue.with_untracked(|vq| vq.get_index(0).cloned()) {
                return Some(post);
            }

            match canisters.get_post_details(canister_id, post_id).await {
                Ok(res) => res,
                Err(e) => {
                    failure_redirect(e);
                    None
                }
            }
        })
    });
    let children_s = StoredValue::new(children);

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || {
                intial_post
                    .get()
                    .flatten()
                    .map(|pd| {
                        Some(
                            view! {
                                <div class="absolute top-4 left-4 z-10 text-white bg-transparent">
                                    <BackButton fallback="/".to_string() />
                                </div>
                                {(children_s.get_value())(pd)}
                            },
                        )
                    })
            }}

        </Suspense>
    }
    .into_any()
}

#[derive(Params, PartialEq)]
struct ProfileVideoParams {
    canister_id: Principal,
    post_id: u64,
}

#[derive(Params, PartialEq, Clone)]
struct ProfileQueryParams {
    next: Option<usize>,
}

const PROFILE_POST_LIMIT: u64 = 25;
type DefProfileVidStream = ProfileVideoStream<PROFILE_POST_LIMIT>;

#[component]
pub fn ProfilePost() -> impl IntoView {
    let params = use_params::<ProfileVideoParams>();
    let query_params = use_query::<ProfileQueryParams>();

    let canister_and_post = Signal::derive(move || {
        params.with_untracked(|p| {
            let p = p.as_ref().ok()?;
            Some((p.canister_id, p.post_id))
        })
    });
    
    let next_start_idx = Signal::derive(move || {
        query_params.with_untracked(|q| {
            q.as_ref().ok()?.next
        })
    });

    view! {
        <ProfilePostBase canister_and_post next_start_idx let:pd>
            <ProfilePostWithUpdates<
            PROFILE_POST_LIMIT,
            DefProfileVidStream,
        > user_canister=pd.canister_id initial_post=pd />
        </ProfilePostBase>
    }
    .into_any()
}

// TODO: handle custom context management for bets
// #[derive(Params, PartialEq)]
// struct ProfileBetsParams {
//     bet_canister: Principal,
//     post_canister: Principal,
//     post_id: u64,
// }

// const PROFILE_POST_BET_LIMIT: u64 = 10;

// #[component]
// pub fn ProfilePostBets() -> impl IntoView {
//     let params = use_params::<ProfileBetsParams>();

//     let user_canister = params.with_untracked(|p| p.as_ref().map(|p| p.bet_canister).unwrap_or(Principal::anonymous()));
//     let canister_and_post = Signal::derive(move || {
//         params.with_untracked(|p| {
//             let p = p.as_ref().ok()?;
//             Some((p.post_canister, p.post_id))
//         })
//     });

//     view! {
//         <ProfilePostBase canister_and_post let:pd>
//             <ProfilePostWithUpdates<PROFILE_POST_BET_LIMIT, ProfileVideoBetsStream> user_canister initial_post=pd/>
//         </ProfilePostBase>
//     }
// }
