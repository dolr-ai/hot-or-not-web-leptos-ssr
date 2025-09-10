use crate::post_view::video_loader::{BgView, VideoViewForQueue};
use indexmap::IndexSet;
use leptos::html;
use leptos::html::Audio;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};

use state::audio_state::AudioState;
use utils::{ml_feed::QuickPostDetails, posts::FeedPostCtx};
use yral_canisters_common::utils::posts::PostDetails;

/// A trait that requires some post details to be accessible instantly while others may be suspended
pub trait PostDetailResolver {
    fn get_quick_post_details(&self) -> QuickPostDetails;
    fn get_post_details(
        &self,
    ) -> impl std::future::Future<Output = Result<PostDetails, ServerFnError>> + Send;
}

// Implementing this trait for post details for backwards compatibility
impl PostDetailResolver for PostDetails {
    fn get_quick_post_details(&self) -> QuickPostDetails {
        QuickPostDetails {
            video_uid: self.uid.clone(),
            canister_id: self.canister_id,
            post_id: self.post_id.clone(),
            publisher_user_id: self.poster_principal,
            nsfw_probability: self.nsfw_probability,
        }
    }

    async fn get_post_details(&self) -> Result<PostDetails, ServerFnError> {
        Ok(self.clone())
    }
}

#[component]
pub fn MuteUnmuteOverlay(muted: RwSignal<bool>) -> impl IntoView {
    view! {
        <div
            class="fixed top-1/2 left-1/2 z-20 text-[5rem] pointer-events-none transform -translate-x-1/2 -translate-y-1/2"
        >
            <Show
                when=move || muted.get()
                fallback=|| view! {
                    <Icon
                        attr:class="text-white/80 mute-indicator"
                        icon=icondata::BiVolumeFullSolid
                    />
                }
            >
            <Icon
                attr:class="text-white/80 mute-indicator"
                icon=icondata::BiVolumeMuteSolid
            />
            </Show>
        </div>
    }
}

#[component]
pub fn ScrollingPostView<
    F: Fn() -> V + Clone + 'static + Send + Sync,
    V,
    DetailResolver: PostDetailResolver + PartialEq + Clone + Sync + Send + 'static,
>(
    video_queue: RwSignal<IndexSet<DetailResolver>>,
    video_queue_for_feed: RwSignal<Vec<FeedPostCtx<DetailResolver>>>,
    current_idx: RwSignal<usize>,
    #[prop(optional)] fetch_next_videos: Option<F>,
    recovering_state: RwSignal<bool>,
    queue_end: RwSignal<bool>,
    #[prop(optional, into)] overlay: Option<ViewFn>,
    threshold_trigger_fetch: usize,
    #[prop(optional, into)] _hard_refresh_target: RwSignal<String>,
) -> impl IntoView {
    let AudioState { muted, volume } = AudioState::get();

    let scroll_root: NodeRef<html::Div> = NodeRef::new();
    let win_audio_ref = NodeRef::<Audio>::new();

    // Monitor current_idx and trigger hard refresh when reaching the end
    Effect::new(move |_| {
        let current = current_idx.get();
        let queue_len = video_queue_for_feed.with(|vqf| vqf.len());

        // Check if we're at the last video (or second to last to be safe)
        if queue_len > 0 && current >= queue_len.saturating_sub(2) {
            if let Some(win) = leptos::web_sys::window() {
                let target = _hard_refresh_target.get_untracked();
                if !target.is_empty() {
                    leptos::logging::log!(
                        "Hard refresh triggered: current={}, queue_len={}, target={}",
                        current,
                        queue_len,
                        target
                    );
                    let _ = win.location().set_href(&target);
                }
            }
        }
    });

    let var_name = view! {
        <div class="overflow-hidden overflow-y-auto w-full h-full">
            <audio
                class="sr-only"
                node_ref=win_audio_ref
                preload="auto"
                src="/img/hotornot/chaching.m4a"
            />
            <div
                node_ref=scroll_root
                class="overflow-y-scroll bg-black snap-mandatory snap-y h-dvh w-dvw"
                style:scroll-snap-points-y="repeat(100vh)"
            >

                {overlay.map(|o| o.run())}

                <For
                    each=move || video_queue_for_feed.get()
                    key=move |feedpost| (feedpost.key)
                    children=move |feedpost| {
                        let queue_idx = feedpost.key;
                        let post = feedpost.value;
                        let container_ref = NodeRef::<html::Div>::new();
                        let next_videos = fetch_next_videos.clone();

                        // Simplified intersection observer without recursive updates
                        use_intersection_observer_with_options(
                            container_ref,
                            move |entry, _| {
                                let Some(visible) = entry.first().filter(|e| e.is_intersecting())
                                else {
                                    return;
                                };

                                // Avoid updating if already at this index
                                let current = current_idx.get_untracked();
                                if queue_idx == current {
                                    return;
                                }

                                let rect = visible.bounding_client_rect();
                                // Check if the element is actually visible enough
                                if rect.y() == rect.height() {
                                    return;
                                }

                                // Update current index
                                current_idx.set(queue_idx);

                                // Trigger fetch if needed (without recursive calls)
                                let queue_len = video_queue.with_untracked(|q| q.len());
                                let remaining = queue_len.saturating_sub(queue_idx);

                                if remaining <= threshold_trigger_fetch {
                                    if let Some(fetch_fn) = next_videos.as_ref() {
                                        // The fetch function itself checks if it's already pending
                                        fetch_fn();
                                    }
                                }
                            },
                            UseIntersectionObserverOptions::default()
                                .thresholds(vec![0.83])
                                .root(Some(scroll_root)),
                        );

                        // Handle initial scroll position recovery only
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
                                    <BgView win_audio_ref video_queue idx=queue_idx>
                                        <VideoViewForQueue
                                            post
                                            current_idx
                                            idx=queue_idx
                                            muted
                                            volume
                                            to_load
                                        />
                                    </BgView>
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
    };
    var_name.into_any()
}
