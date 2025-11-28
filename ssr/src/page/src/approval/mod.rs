pub mod api;
pub mod overlay;

use crate::post_view::video_loader::VideoViewForQueue;
use crate::post_view::PostDetailsCacheCtx;
use crate::scrolling_post_view::{MuteUnmuteOverlay, PostDetailResolver};
use component::spinner::FullScreenSpinner;
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use indexmap::IndexSet;
use leptos::prelude::*;
use leptos_meta::Title;
use state::audio_state::AudioState;
use state::canisters::auth_state;
use std::collections::HashMap;
use utils::ml_feed::QuickPostDetails;
use utils::posts::FeedPostCtx;
use yral_canisters_common::utils::posts::PostDetails;
use yral_types::delegated_identity::DelegatedIdentityWire;

use api::{fetch_pending_approval_videos, PendingVideo};
use overlay::ApprovalOverlay;

/// Context for the approval view
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

/// Post item for approval queue
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApprovalPostItem {
    pub video_uid: String,
    pub canister_id: candid::Principal,
    pub post_id: String,
    pub publisher_user_id: candid::Principal,
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
        use state::canisters::unauth_canisters;
        use utils::send_wrap;

        let canisters = unauth_canisters();
        let post_details = send_wrap(canisters.get_post_details_with_nsfw_info(
            self.canister_id,
            self.post_id.clone(),
            Some(0.0),
        ))
        .await?;

        post_details.ok_or_else(|| {
            ServerFnError::new(format!(
                "Couldn't find post {}/{}",
                self.canister_id, &self.post_id
            ))
        })
    }
}

/// Main entry point for the approval page
#[component]
pub fn ApprovalView() -> impl IntoView {
    // Provide contexts
    let ctx = ApprovalViewCtx::new();
    provide_context(ctx.clone());
    provide_context(PostDetailsCacheCtx {
        post_details: StoredValue::new(HashMap::new()),
    });

    view! {
        <Title text="Video Approval" />
        <Suspense fallback=FullScreenSpinner>
            <ApprovalFeed />
        </Suspense>
    }
}

/// The main approval feed component
#[component]
fn ApprovalFeed() -> impl IntoView {
    let ApprovalViewCtx {
        video_queue,
        video_queue_for_feed,
        current_idx,
        queue_end,
    } = expect_context();

    let recovering_state = RwSignal::new(false);
    let hard_refresh_target = RwSignal::new("/approve".to_string());

    // Get auth state for identity
    let auth = auth_state();

    // Store the identity wire for use in API calls
    let identity_wire: RwSignal<Option<DelegatedIdentityWire>> = RwSignal::new(None);

    // Fetch identity when available
    let identity_resource = LocalResource::new(move || async move {
        let new_identity = auth.user_identity.await?;
        Ok::<_, ServerFnError>(new_identity.id_wire)
    });

    // Update identity_wire when resource resolves
    Effect::new(move |_| {
        if let Some(Ok(wire)) = identity_resource.get() {
            identity_wire.set(Some(wire));
        }
    });

    // Action to fetch pending videos - matches PostViewWithUpdatesMLFeed pattern
    let fetch_action = Action::new(move |_: &()| async move {
        let Some(wire) = identity_wire.get_untracked() else {
            leptos::logging::warn!("Cannot fetch pending videos: identity not loaded");
            return;
        };

        let offset = video_queue.with_untracked(|q| q.len()) as u32;
        match fetch_pending_approval_videos(wire, offset, 20).await {
            Ok(response) => {
                if response.videos.is_empty() {
                    queue_end.set(true);
                } else {
                    // Match the pattern from PostViewWithUpdatesMLFeed
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
                    // Check if we've reached the total count
                    let queue_len = video_queue.with_untracked(|q| q.len());
                    if queue_len >= response.total_count {
                        queue_end.set(true);
                    }
                }
            }
            Err(e) => {
                leptos::logging::error!("Failed to fetch pending videos: {:?}", e);
            }
        }
    });

    // Initial fetch - matches CommonPostViewWithUpdates pattern
    Effect::new(move |_| {
        // Only dispatch if identity is loaded
        if identity_wire.get().is_some() && !recovering_state.get_untracked() {
            fetch_action.dispatch(());
        }
    });

    // Next videos function - matches the pattern exactly
    let next_videos = move || {
        if !fetch_action.pending().get_untracked() && !queue_end.get_untracked() {
            fetch_action.dispatch(());
        }
    };

    view! {
        <ApprovalScrollingView
            video_queue
            video_queue_for_feed
            current_idx
            recovering_state
            fetch_next_videos=next_videos
            queue_end
            threshold_trigger_fetch=10
            _hard_refresh_target=hard_refresh_target
            identity_wire
        />
    }
}

/// Custom scrolling view for approval that uses ApprovalOverlay instead of VideoDetailsOverlay
#[component]
fn ApprovalScrollingView<F: Fn() -> V + Clone + 'static + Send + Sync, V>(
    video_queue: RwSignal<IndexSet<ApprovalPostItem>>,
    video_queue_for_feed: RwSignal<Vec<FeedPostCtx<ApprovalPostItem>>>,
    current_idx: RwSignal<usize>,
    #[prop(optional)] fetch_next_videos: Option<F>,
    recovering_state: RwSignal<bool>,
    queue_end: RwSignal<bool>,
    threshold_trigger_fetch: usize,
    #[prop(optional, into)] _hard_refresh_target: RwSignal<String>,
    identity_wire: RwSignal<Option<DelegatedIdentityWire>>,
) -> impl IntoView {
    use leptos::html;
    use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};

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
                                let Some(visible) = entry.first().filter(|e| e.is_intersecting())
                                else {
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
                                    if let Some(fetch_fn) = next_videos.as_ref() {
                                        fetch_fn();
                                    }
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
                                    <ApprovalBgView video_queue idx=queue_idx current_idx identity_wire>
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
                        <span>No more videos to approve!</span>
                    </div>
                </Show>

                <MuteUnmuteOverlay muted />
            </div>
        </div>
    }.into_any()
}

/// Background view for approval that uses ApprovalOverlay
#[component]
fn ApprovalBgView(
    video_queue: RwSignal<IndexSet<ApprovalPostItem>>,
    idx: usize,
    current_idx: RwSignal<usize>,
    identity_wire: RwSignal<Option<DelegatedIdentityWire>>,
    children: Children,
) -> impl IntoView {
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

        let details = match cached {
            Some(d) => d,
            None => {
                let d = utils::send_wrap(resolver.get_post_details()).await?;
                post_details_cache
                    .try_update_value(|m| m.insert((canister_id, post_id), d.clone()));
                d
            }
        };

        Ok::<_, ServerFnError>(Some(details))
    });

    let uid = move || {
        post_with_prev()
            .as_ref()
            .map(|q| q.get_quick_post_details().video_uid)
            .unwrap_or_default()
    };

    view! {
        <div class="overflow-hidden relative w-full h-full bg-transparent">
            <div
                class="absolute top-0 left-0 w-full h-full bg-center bg-cover z-1 blur-lg bg-black"
                style:background-image=move || format!("url({})", utils::bg_url(uid()))
            ></div>
            <Suspense>
                {move || Suspend::new(async move {
                    let post = post_details_res.await
                        .inspect_err(|e| leptos::logging::error!("Failed to load post: {e:#?}"))
                        .ok()?;
                    Some(view! { <ApprovalOverlay post=post? current_idx high_priority=idx < 3 identity_wire /> }.into_view())
                })}
            </Suspense>
            {children()}
        </div>
    }.into_any()
}
