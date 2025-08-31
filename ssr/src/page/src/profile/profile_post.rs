use candid::Principal;
use component::{back_btn::BackButton, spinner::FullScreenSpinner};
use leptos::prelude::*;
use leptos_router::params::Params;
use leptos_router::{
    hooks::{use_navigate, use_params, use_query},
    *,
};
use state::canisters::{auth_state, unauth_canisters};
use utils::{route::failure_redirect, send_wrap, try_or_redirect};

use crate::scrolling_post_view::ScrollingPostView;

use super::{
    overlay::YourProfileOverlay,
    profile_iter::{FixedFetchCursor, ProfVideoStream, ProfileVideoStream},
    ProfilePostsContext,
};

use yral_canisters_common::utils::posts::PostDetails;

const PROFILE_POST_LIMIT: u64 = 25;
type DefProfileVidStream = ProfileVideoStream<PROFILE_POST_LIMIT>;

#[component]
pub fn PostViewWithUpdatesProfile(
    initial_post: PostDetails,
    user_canister: Principal,
) -> impl IntoView {
    let ProfilePostsContext {
        video_queue,
        video_queue_for_feed,
        start_index,
        current_index,
        queue_end,
    } = expect_context();

    let auth = auth_state();
    let recovering_state = RwSignal::new(false);
    let hard_refresh_target = RwSignal::new("/".to_string());

    let username = StoredValue::new(initial_post.username.clone());

    // Initialize cursor to fetch posts after the ones already in video_queue
    let initial_cursor_start =
        start_index.get_untracked() + video_queue.with_untracked(|vq| vq.len());
    let fetch_cursor = RwSignal::new(FixedFetchCursor::<PROFILE_POST_LIMIT> {
        start: initial_cursor_start as u64,
        limit: PROFILE_POST_LIMIT,
    });

    // Create overlay function
    let overlay = move || {
        view! {
            <Suspense>
                {move || Suspend::new(async move {
                    let canister = auth.user_canister().await.ok()?;
                    (canister == initial_post.canister_id)
                        .then(|| view! { <YourProfileOverlay />  }.into_any())
                })}
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

    let fetch_video_action: Action<_, _> = Action::new_unsync(move |_| {
        async move {
            let cursor = fetch_cursor.get_untracked();
            let canisters = unauth_canisters();
            let posts_res = if let Some(canisters) = auth.auth_cans_if_available() {
                DefProfileVidStream::fetch_next_posts(
                    cursor,
                    &canisters,
                    user_canister,
                    username.get_value(),
                )
                .await
            } else {
                DefProfileVidStream::fetch_next_posts(
                    cursor,
                    &canisters,
                    user_canister,
                    username.get_value(),
                )
                .await
            };

            let res = try_or_redirect!(posts_res);

            queue_end.set(res.end);
            res.posts.into_iter().for_each(|p| {
                video_queue.try_update(|q| {
                    if q.insert(p.clone()) {
                        let len_vq = q.len();
                        if len_vq <= video_queue_for_feed.with_untracked(|vqf| vqf.len()) {
                            video_queue_for_feed.update(|vqf| {
                                vqf[len_vq - 1].value.set(Some(p.clone()));
                            });
                            // Update hard refresh target to this post
                            let next_start_idx = fetch_cursor.with_untracked(|c| c.start + c.limit);
                            hard_refresh_target.set(format!(
                                "/profile/{}/post/{}?next={}",
                                p.canister_id,
                                p.post_id,
                                next_start_idx - 1
                            ));
                        }
                    }
                });
            });

            fetch_cursor.try_update(|c| {
                c.advance();
            });
        }
    });

    // Trigger initial fetch if not recovering state
    Effect::new(move || {
        if !recovering_state.get_untracked() && video_queue.with_untracked(|vq| vq.len()) <= 1 {
            fetch_video_action.dispatch(());
        }
    });

    // Simplified fetch trigger without debounce - the fetching guard in ScrollingPostView handles rate limiting
    let fetch_next_videos = move || {
        if !fetch_video_action.pending().get_untracked() && !queue_end.get_untracked() {
            log::debug!("trigger rerender");
            fetch_video_action.dispatch(());
        }
    };

    let current_post_base = Memo::new(move |_| {
        video_queue.with(|q| {
            let curr_index = current_index();
            let details = q.get_index(curr_index);
            details.map(|d| (d.canister_id, d.post_id))
        })
    });

    Effect::new(move || {
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
            _hard_refresh_target=hard_refresh_target
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
    if let Some(next_idx) = next_start_idx.get_untracked() {
        start_index.set(next_idx);
    }

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

            video_queue.update(|vq| {
                if let Some(idx) = post_idx {
                    // Remove all posts before the target post
                    if idx > 0 {
                        vq.drain(0..idx);
                        // Update start_index to account for the removed posts
                        start_index.update(|si| *si += idx);
                    }

                    // Always update video_queue_for_feed to reflect the new state
                    video_queue_for_feed.update(|vqf| {
                        // Re-populate from the updated video_queue
                        for (i, post) in vq.iter().take(vqf.len()).enumerate() {
                            vqf[i].value.set(Some(post.clone()));
                        }
                    });
                    // Now current_index is 0 since we removed all previous posts
                    current_index.set(0);
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

#[derive(Params, PartialEq, Clone, Debug)]
struct ProfileQueryParams {
    next: Option<usize>,
}

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

    let next_start_idx =
        Signal::derive(move || query_params.with_untracked(|q| q.as_ref().ok()?.next));

    view! {
        <ProfilePostBase canister_and_post next_start_idx let:pd>
            <PostViewWithUpdatesProfile user_canister=pd.canister_id initial_post=pd />
        </ProfilePostBase>
    }
    .into_any()
}
