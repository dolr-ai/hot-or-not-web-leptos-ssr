use crate::post_view::video_loader::{BgView, VideoViewForQueue};
use crate::post_view::FeedPostCtx;
use indexmap::IndexSet;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_use::{use_intersection_observer_with_options, UseIntersectionObserverOptions};

use state::audio_state::AudioState;
use yral_canisters_common::utils::posts::PostDetails;

#[component]
pub fn MuteIconOverlay(show_mute_icon: RwSignal<bool>) -> impl IntoView {
    view! {
        <Show when=show_mute_icon>
            <button
                class="fixed top-1/2 left-1/2 z-20 cursor-pointer pointer-events-none"
                on:click=move |_| AudioState::toggle_mute()
            >
                <Icon
                    attr:class="text-white/80 animate-ping text-4xl"
                    icon=icondata::BiVolumeMuteSolid
                />
            </button>
        </Show>
    }
}

#[component]
pub fn ScrollingPostView<F: Fn() -> V + Clone + 'static + Send + Sync, V>(
    video_queue: RwSignal<IndexSet<PostDetails>>,
    video_queue_for_feed: RwSignal<Vec<FeedPostCtx>>,
    current_idx: RwSignal<usize>,
    #[prop(optional)] fetch_next_videos: Option<F>,
    recovering_state: RwSignal<bool>,
    queue_end: RwSignal<bool>,
    #[prop(optional, into)] overlay: Option<ViewFn>,
    threshold_trigger_fetch: usize,
) -> impl IntoView {
    let AudioState {
        muted,
        show_mute_icon,
        ..
    } = AudioState::get();

    let scroll_root: NodeRef<html::Div> = NodeRef::new();

    let var_name = view! {
        <div class="overflow-hidden overflow-y-auto w-full h-full">
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
                        use_intersection_observer_with_options(
                            container_ref,
                            move |entry, _| {
                                let Some(visible) = entry.first().filter(|e| e.is_intersecting())
                                else {
                                    return;
                                };
                                let rect = visible.bounding_client_rect();
                                if rect.y() == rect.height()
                                    || queue_idx == current_idx.get_untracked()
                                {
                                    return;
                                }
                                current_idx.set(queue_idx);
                                if video_queue.with_untracked(|q| q.len()).saturating_sub(queue_idx)
                                    <= threshold_trigger_fetch
                                {
                                    next_videos.as_ref().map(|nv| { nv() });
                                }
                            },
                            UseIntersectionObserverOptions::default()
                                .thresholds(vec![0.83])
                                .root(Some(scroll_root)),
                        );
                        Effect::new(move |_| {
                            let Some(container) = container_ref.get() else {
                                return;
                            };
                            if current_idx() == queue_idx && recovering_state.get_untracked() {
                                container.scroll_into_view();
                                recovering_state.set(false);
                            }
                        });
                        let show_video = Memo::new(move |_| {
                            (queue_idx as i32 - current_idx() as i32) >= -3
                        });
                        let to_load = Signal::derive(move || {
                            let cidx = current_idx.get() as i32;
                            (queue_idx as i32 - cidx) <= 2 && (queue_idx as i32 - cidx) >= 0
                        });
                        // let post = Signal::derive(move || {
                        //     details.value.clone()
                        // });
                        view! {
                            <div node_ref=container_ref class="w-full h-full snap-always snap-end">
                                <Show when=show_video>
                                    <BgView video_queue idx=queue_idx>
                                        <VideoViewForQueue
                                            post
                                            current_idx
                                            idx=queue_idx
                                            muted
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

                <MuteIconOverlay show_mute_icon />
            </div>
        </div>
    };
    var_name.into_any()
}
