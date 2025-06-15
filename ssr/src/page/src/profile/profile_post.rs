use std::marker::PhantomData;

use candid::Principal;
use component::{back_btn::BackButton, spinner::FullScreenSpinner};
use leptos::prelude::*;
use leptos_router::params::Params;
use leptos_router::{
    hooks::{use_navigate, use_params},
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
        start_index,
        current_index,
        queue_end,
    } = expect_context();
    let recovering_state = RwSignal::new(true);
    let fetch_cursor = RwSignal::new(FixedFetchCursor::<LIMIT> {
        start: start_index.get_untracked() as u64,
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

    if start_index.get_untracked() == 0 {
        video_queue.update_untracked(|vq| {
            let _ = vq.insert(initial_post.clone());
        });
        queue_end.set(true)
    }

    let next_videos: Action<_, _> = Action::new_unsync(move |_| async move {
        let cursor = fetch_cursor.get_untracked();

        let canisters = unauth_canisters();
        let posts_res = if let Some(canisters) = auth.auth_cans_if_available(canisters.clone()) {
            VidStream::fetch_next_posts(cursor, &canisters, user_canister).await
        } else {
            VidStream::fetch_next_posts(cursor, &canisters, user_canister).await
        };

        let res = try_or_redirect!(posts_res);

        queue_end.set(res.end);
        res.posts.into_iter().for_each(|p| {
            video_queue.try_update(|q| {
                let _ = q.insert(p);
            });
        });
        fetch_cursor.try_update(|c| {
            c.advance();
        });
    });

    let fetch_next_videos = use_debounce_fn(
        move || {
            if !next_videos.pending().get_untracked() && !queue_end.get_untracked() {
                log::debug!("trigger rerender");
                next_videos.dispatch(video_queue);
            }
        },
        500.0,
    );

    let current_post_base = Memo::new(move |_| {
        video_queue.with(|q| {
            let details = q.get_index(current_index());
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
            current_idx=current_index
            queue_end
            recovering_state
            fetch_next_videos
            overlay
            threshold_trigger_fetch=10
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
    children: C,
) -> impl IntoView {
    let ProfilePostsContext {
        video_queue,
        current_index,
        ..
    } = expect_context();

    let intial_post = Resource::new(canister_and_post, move |params| {
        let canisters = unauth_canisters();
        send_wrap(async move {
            let Some((canister_id, post_id)) = params else {
                failure_redirect("Invalid profile post");
                return None;
            };

            let retrieved_post = video_queue.with_untracked(|vq| {
                let post_idx = vq
                    .iter()
                    .position(|post| post.canister_id == canister_id && post.post_id == post_id);
                current_index.update(|idx| *idx = post_idx.unwrap_or(0));
                post_idx.and_then(|p_idx| vq.get_index(p_idx)).cloned()
            });

            if let Some(post) = retrieved_post {
                return Some(post);
            };

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

const PROFILE_POST_LIMIT: u64 = 25;
type DefProfileVidStream = ProfileVideoStream<PROFILE_POST_LIMIT>;

#[component]
pub fn ProfilePost() -> impl IntoView {
    let params = use_params::<ProfileVideoParams>();

    let canister_and_post = Signal::derive(move || {
        params.with_untracked(|p| {
            let p = p.as_ref().ok()?;
            Some((p.canister_id, p.post_id))
        })
    });

    view! {
        <ProfilePostBase canister_and_post let:pd>
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
