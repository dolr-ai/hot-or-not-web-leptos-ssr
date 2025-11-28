pub mod api;
pub mod overlay;

use crate::post_view::video_loader::VideoViewForQueue;
use crate::post_view::PostDetailsCacheCtx;
use crate::scrolling_post_view::{MuteUnmuteOverlay, PostDetailResolver};
use component::spinner::FullScreenSpinner;
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use indexmap::IndexSet;
use leptos::html;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};
use overlay::ApprovalOverlay;
use state::audio_state::AudioState;
use state::canisters::auth_state;
use std::collections::HashMap;
use utils::ml_feed::QuickPostDetails;
use utils::posts::FeedPostCtx;
use utils::{bg_url, send_wrap};
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
        // Return minimal details - we only need the video to play with approve/disapprove buttons
        Ok(PostDetails {
            canister_id: self.canister_id,
            post_id: self.post_id.clone(),
            uid: self.video_uid.clone(),
            description: String::new(),
            views: 0,
            likes: 0,
            display_name: None,
            username: None,
            propic_url: String::new(),
            liked_by_user: None,
            poster_principal: self.publisher_user_id,
            creator_follows_user: None,
            user_follows_creator: None,
            creator_bio: None,
            hastags: vec![],
            is_nsfw: false,
            hot_or_not_feed_ranking_score: None,
            created_at: web_time::Duration::ZERO,
            nsfw_probability: 0.0,
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

/// BgView for approval - uses ApprovalOverlay instead of VideoDetailsOverlay
#[component]
fn ApprovalBgView(
    video_queue: RwSignal<IndexSet<ApprovalPostItem>>,
    idx: usize,
    current_idx: RwSignal<usize>,
    children: Children,
) -> impl IntoView {
    let identity_wire: RwSignal<Option<DelegatedIdentityWire>> = expect_context();

    let post_with_prev = Memo::new(move |_| video_queue.with(|q| q.get_index(idx).cloned()));

    let PostDetailsCacheCtx {
        post_details: post_details_cache,
    } = expect_context();

    let post_details_res = LocalResource::new(move || async move {
        let Some(resolver) = post_with_prev.get() else {
            return Ok(None);
        };

        let QuickPostDetails {
            canister_id,
            post_id,
            ..
        } = resolver.get_quick_post_details();

        let post_id_c = post_id.clone();
        let cached = post_details_cache
            .try_with_value(|m| m.get(&(canister_id, post_id_c)).cloned())
            .flatten();

        let post_details = match cached {
            Some(details) => details,
            None => {
                let details = send_wrap(resolver.get_post_details()).await?;
                post_details_cache
                    .try_update_value(|m| m.insert((canister_id, post_id), details.clone()));
                details
            }
        };
        Ok::<_, ServerFnError>(Some(post_details))
    });

    let uid = move || {
        post_with_prev()
            .as_ref()
            .map(|q| q.get_quick_post_details().video_uid)
            .unwrap_or_default()
    };

    let high_priority = idx < 3;

    view! {
        <div class="overflow-hidden relative w-full h-full bg-transparent">
            <div
                class="absolute top-0 left-0 w-full h-full bg-center bg-cover z-1 blur-lg bg-black"
                style:background-image=move || format!("url({})", bg_url(uid()))
            ></div>
            <Suspense>
                {move || Suspend::new(async move {
                    let post = post_details_res.await.ok()??;
                    Some(view! { <ApprovalOverlay post=post current_idx=current_idx high_priority=high_priority identity_wire=identity_wire /> }.into_view())
                })}
            </Suspense>
            {children()}
        </div>
    }
    .into_any()
}

/// Custom scrolling view for approval that uses ApprovalBgView
#[component]
fn ApprovalScrollingPostView<F: Fn() + Clone + 'static + Send + Sync>(
    video_queue: RwSignal<IndexSet<ApprovalPostItem>>,
    video_queue_for_feed: RwSignal<Vec<FeedPostCtx<ApprovalPostItem>>>,
    current_idx: RwSignal<usize>,
    fetch_next_videos: F,
    recovering_state: RwSignal<bool>,
    queue_end: RwSignal<bool>,
    threshold_trigger_fetch: usize,
) -> impl IntoView {
    let AudioState { muted, volume } = AudioState::get();
    let scroll_root: NodeRef<html::Div> = NodeRef::new();

    view! {
        <div class="overflow-hidden overflow-y-auto w-full h-full">
            <div
                node_ref=scroll_root
                class="overflow-y-scroll bg-black snap-mandatory snap-y h-dvh w-dvw"
                style:scroll-snap-points-y="repeat(100vh)"
            >
                <For
                    each=move || video_queue_for_feed.get()
                    key=move |feedpost| feedpost.key
                    children=move |feedpost| {
                        let queue_idx = feedpost.key;
                        let post = feedpost.value;
                        let container_ref = NodeRef::<html::Div>::new();
                        let next_videos = fetch_next_videos.clone();

                        use_intersection_observer_with_options(
                            container_ref,
                            move |entry, _| {
                                let Some(visible) = entry.first().filter(|e| e.is_intersecting()) else {
                                    return;
                                };

                                let current = current_idx.get_untracked();
                                if queue_idx == current {
                                    return;
                                }

                                let rect = visible.bounding_client_rect();
                                if rect.y() == rect.height() {
                                    return;
                                }

                                current_idx.set(queue_idx);

                                let queue_len = video_queue.with_untracked(|q| q.len());
                                let remaining = queue_len.saturating_sub(queue_idx);

                                if remaining <= threshold_trigger_fetch {
                                    next_videos();
                                }
                            },
                            UseIntersectionObserverOptions::default()
                                .thresholds(vec![0.83])
                                .root(Some(scroll_root)),
                        );

                        if recovering_state.get_untracked() && current_idx.get_untracked() == queue_idx {
                            Effect::new(move |_| {
                                if let Some(container) = container_ref.get() {
                                    container.scroll_into_view();
                                    recovering_state.set(false);
                                }
                            });
                        }

                        let show_video = Memo::new(move |_| {
                            (queue_idx as i32 - current_idx() as i32) >= -2
                        });
                        let to_load = Memo::new(move |_| {
                            let cidx = current_idx.get() as i32;
                            queue_idx <= 5 || ((queue_idx as i32 - cidx) <= 10 && (queue_idx as i32 - cidx) >= -2)
                        });

                        view! {
                            <div node_ref=container_ref class="w-full h-full snap-always snap-end" class:hidden=move || post.get().is_none()>
                                <Show when=show_video>
                                    <ApprovalBgView video_queue=video_queue idx=queue_idx current_idx=current_idx>
                                        <VideoViewForQueue
                                            post
                                            current_idx
                                            idx=queue_idx
                                            muted
                                            volume
                                            to_load
                                        />
                                    </ApprovalBgView>
                                </Show>
                            </div>
                        }.into_any()
                    }
                />

                <Show when=queue_end>
                    <div class="flex relative top-0 left-0 justify-center items-center w-full h-full text-xl bg-inherit z-21 snap-always snap-end text-white/80">
                        <span>You have reached the end!</span>
                    </div>
                </Show>

                <MuteUnmuteOverlay muted />
            </div>
        </div>
    }.into_any()
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

    view! {
        <ApprovalScrollingPostView
            video_queue
            video_queue_for_feed
            current_idx
            recovering_state
            fetch_next_videos=next_videos
            queue_end
            threshold_trigger_fetch=10
        />
    }
}
