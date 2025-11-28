pub mod api;
pub mod overlay;

use crate::post_view::PostDetailsCacheCtx;
use crate::scrolling_post_view::{PostDetailResolver, ScrollingPostView};
use component::spinner::FullScreenSpinner;
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use indexmap::IndexSet;
use leptos::prelude::*;
use leptos_meta::Title;
use state::canisters::{auth_state, unauth_canisters};
use std::collections::HashMap;
use utils::ml_feed::QuickPostDetails;
use utils::posts::FeedPostCtx;
use utils::send_wrap;
use web_time::Duration;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::delegated_identity::DelegatedIdentityWire;

use api::{fetch_pending_approval_videos, PendingVideo};

/// Post item for approval queue - same structure as MlPostItem
#[derive(Debug, Clone)]
pub struct ApprovalPostItem {
    pub canister_id: candid::Principal,
    pub post_id: String,
    pub video_uid: String,
    pub publisher_user_id: candid::Principal,
}

impl std::cmp::PartialEq for ApprovalPostItem {
    fn eq(&self, other: &Self) -> bool {
        self.canister_id == other.canister_id && self.post_id == other.post_id
    }
}

impl std::cmp::Eq for ApprovalPostItem {}

impl std::hash::Hash for ApprovalPostItem {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.canister_id.hash(state);
        state.write(self.post_id.as_bytes());
    }
}

impl From<PendingVideo> for ApprovalPostItem {
    fn from(v: PendingVideo) -> Self {
        Self {
            video_uid: v.video_id,
            canister_id: v
                .canister_id
                .parse()
                .unwrap_or(candid::Principal::anonymous()),
            post_id: v.post_id,
            publisher_user_id: v.user_id.parse().unwrap_or(candid::Principal::anonymous()),
        }
    }
}

impl PostDetailResolver for ApprovalPostItem {
    fn get_quick_post_details(&self) -> QuickPostDetails {
        QuickPostDetails {
            video_uid: self.video_uid.clone(),
            canister_id: self.canister_id,
            post_id: self.post_id.clone(),
            publisher_user_id: self.publisher_user_id,
            nsfw_probability: 0.0,
        }
    }

    async fn get_post_details(&self) -> Result<PostDetails, ServerFnError> {
        let canisters = unauth_canisters();
        let post_details =
            send_wrap(canisters.get_post_details_from_canister(self.canister_id, &self.post_id))
                .await?;
        post_details.ok_or_else(|| {
            ServerFnError::new(format!(
                "Couldn't find post {}/{}",
                self.canister_id, &self.post_id
            ))
        })
    }
}

/// Context for approval view - same as PostViewCtx
#[derive(Clone, Default)]
pub struct ApprovalViewCtx {
    pub video_queue: RwSignal<IndexSet<ApprovalPostItem>>,
    pub video_queue_for_feed: RwSignal<Vec<FeedPostCtx<ApprovalPostItem>>>,
    pub current_idx: RwSignal<usize>,
    pub queue_end: RwSignal<bool>,
}

impl ApprovalViewCtx {
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

/// Main entry - same as PostView
#[component]
pub fn ApprovalView() -> impl IntoView {
    let ctx = ApprovalViewCtx::new();
    provide_context(ctx);
    provide_context(PostDetailsCacheCtx {
        post_details: StoredValue::new(HashMap::new()),
    });

    // Store identity wire in context for overlay to use
    let identity_wire: RwSignal<Option<DelegatedIdentityWire>> = RwSignal::new(None);
    provide_context(identity_wire);

    view! {
        <Title text="Video Approval" />
        <Suspense fallback=FullScreenSpinner>
            <ApprovalViewInner />
        </Suspense>
    }
}

#[component]
fn ApprovalViewInner() -> impl IntoView {
    let auth = auth_state();
    let identity_wire: RwSignal<Option<DelegatedIdentityWire>> = expect_context();

    // Fetch identity first
    let identity_res = Resource::new(
        || (),
        move |_| async move {
            let new_identity = auth.user_identity.await?;
            Ok::<_, ServerFnError>(new_identity.id_wire)
        },
    );

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let wire = identity_res.await.ok()?;
                identity_wire.set(Some(wire));
                Some(view! { <ApprovalFeedWithUpdates /> })
            })}
        </Suspense>
    }
}

/// Same as CommonPostViewWithUpdates
#[component]
fn ApprovalFeedWithUpdates() -> impl IntoView {
    let ApprovalViewCtx {
        video_queue,
        video_queue_for_feed,
        current_idx,
        queue_end,
    } = expect_context();

    let identity_wire: RwSignal<Option<DelegatedIdentityWire>> = expect_context();

    let recovering_state = RwSignal::new(false);

    let fetch_video_action = Action::new(move |_: &()| async move {
        let Some(wire) = identity_wire.get_untracked() else {
            return;
        };

        let offset = video_queue.with_untracked(|q| q.len()) as u32;

        match fetch_pending_approval_videos(wire, offset, 20).await {
            Ok(response) => {
                if response.videos.is_empty() && offset == 0 {
                    queue_end.set(true);
                    return;
                }

                for video in response.videos {
                    let item: ApprovalPostItem = video.into();
                    video_queue.update(|vq| {
                        if vq.insert(item.clone()) {
                            let len_vq = vq.len();
                            if len_vq > video_queue_for_feed.with_untracked(|vqf| vqf.len()) {
                                return;
                            }
                            video_queue_for_feed.update(|vqf| {
                                vqf[len_vq - 1].value.set(Some(item.clone()));
                            });
                        }
                    });
                }

                if video_queue.with_untracked(|q| q.len()) >= response.total_count {
                    queue_end.set(true);
                }
            }
            Err(e) => {
                leptos::logging::error!("Failed to fetch: {:?}", e);
            }
        }
    });

    // Initial fetch
    Effect::new(move |_| {
        if !recovering_state.get_untracked() {
            fetch_video_action.dispatch(());
        }
    });

    let next_videos = move || {
        if !fetch_video_action.pending().get_untracked() && !queue_end.get_untracked() {
            fetch_video_action.dispatch(());
        }
    };

    let hard_refresh_target = RwSignal::new("/approve".to_string());

    view! {
        <ScrollingPostView
            video_queue
            video_queue_for_feed
            current_idx
            recovering_state
            fetch_next_videos=next_videos
            queue_end
            threshold_trigger_fetch=10
            _hard_refresh_target=hard_refresh_target
        />
    }
}
