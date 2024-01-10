mod error;
mod video_iter;
mod video_loader;

use std::pin::pin;

use candid::Principal;
use futures::StreamExt;
use leptos::{html::Video, *};
use leptos_router::*;

use crate::{component::spinner::Spinner, state::canisters::Canisters, try_or_redirect};
use video_iter::{get_post_uid, VideoFetchStream};
use video_loader::{BgView, HlsVideo, ThumbView};

use self::video_iter::PostDetails;

#[derive(Params, PartialEq)]
struct PostParams {
    canister_id: String,
    post_id: u64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct FetchCursor {
    start: u64,
    limit: u64,
}

#[derive(Clone)]
struct VideoCtx {
    video_queue: ReadSignal<Vec<PostDetails>>,
    current_idx: RwSignal<usize>,
    trigger_fetch: Action<(), ()>,
}

const POST_CNT: usize = 25;
const PLAYER_CNT: usize = 5;

// Infinite Scrolling View
// Basically a virtual list with 5 items visible at a time
#[component]
pub fn ScrollingView() -> impl IntoView {
    let VideoCtx {
        video_queue,
        current_idx,
        ..
    } = expect_context();
    let allow_show = create_rw_signal(true);

    let video_ref = create_node_ref::<Video>();
    // Cache wasp views to avoid re-initialization
    let _video_view = view! { <HlsVideo video_ref allow_show/> };
    let current_start = move || {
        let cur_idx = current_idx();
        cur_idx.max(PLAYER_CNT / 2) - (PLAYER_CNT / 2)
    };

    let video_enum = create_memo(move |_| {
        with!(|video_queue| {
            let start = current_start();
            video_queue[start..]
                .iter()
                .take(PLAYER_CNT)
                .enumerate()
                .map(|(idx, item)| (idx + start, item.clone()))
                .collect::<Vec<_>>()
        })
    });

    view! {
        <div
            class="snap-mandatory snap-y overflow-scroll h-screen"
            style:scroll-snap-points-y="repeat(100vh)"
        >
            <For
                each=video_enum
                key=|u| (u.0, u.1.uid.clone())
                children=move |(queue_idx, details)| {
                    view! {
                        <div class="snap-always snap-end">
                            <BgView uid=details.uid.clone()>
                                <Show
                                    when=move || queue_idx == current_idx() && allow_show()
                                    fallback=move || view! { <ThumbView idx=queue_idx/> }
                                >
                                    {video_ref}
                                </Show>
                            </BgView>
                        </div>
                    }
                }
            />

        </div>
    }
}

#[component]
pub fn PostView() -> impl IntoView {
    let params = use_params::<PostParams>();
    let canister_and_post = move || {
        params.with_untracked(|p| {
            let p = p.as_ref().ok()?;
            let canister_id = Principal::from_text(&p.canister_id).ok()?;

            Some((canister_id, p.post_id))
        })
    };

    // TODO: this is a dead simple with no GC
    // We're using virtual lists for DOM, so this doesn't consume much memory
    // as uids only occupy 32 bytes each
    // but ideally this should be cleaned up
    let (video_queue, set_video_queue) = create_signal(Vec::new());
    let current_idx = create_rw_signal(0);
    let (fetch_cursor, set_fetch_cursor) = create_signal(FetchCursor {
        start: 1,
        limit: POST_CNT as u64 - 1,
    });

    let fetch_video_uids = create_action(move |&()| async move {
        let canisters = expect_context::<Canisters>();
        let cursor = fetch_cursor.get_untracked();
        let fetch_stream = VideoFetchStream::new(&canisters, cursor);
        let chunks = try_or_redirect!(fetch_stream.fetch_post_uids_chunked(10).await);
        let mut chunks = pin!(chunks);
        while let Some(chunk) = chunks.next().await {
            set_video_queue.update(|q| {
                for uid in chunk {
                    let uid = try_or_redirect!(uid);
                    q.push(uid);
                }
            });
        }

        set_fetch_cursor.update(|cursor| {
            cursor.start += cursor.limit;
            cursor.limit = 20
        });
    });
    provide_context(VideoCtx {
        video_queue,
        current_idx,
        trigger_fetch: fetch_video_uids,
    });

    let fetch_first_video_uid = create_resource(
        || (),
        move |_| async move {
            let canisters = expect_context::<Canisters>();
            let Some((canister, post)) = canister_and_post() else {
                use_navigate()("/", Default::default());
                return;
            };
            let uid = try_or_redirect!(get_post_uid(&canisters, canister, post).await).unwrap();
            set_video_queue.update(|q| q.extend_from_slice(&[uid]));
        },
    );

    let current_post_base = create_memo(move |_| {
        with!(|video_queue| {
            let cur_idx = current_idx();
            let details = video_queue.get(cur_idx)?;
            Some((details.canister_id, details.post_id))
        })
    });

    create_effect(move |_| {
        let Some((canister_id, post_id)) = current_post_base() else {
            return;
        };
        use_navigate()(
            &format!("/hot-or-not/{canister_id}/{post_id}",),
            Default::default(),
        );
    });

    view! {
        <Suspense fallback=|| {
            view! {
                <div class="grid grid-cols-1 h-screen w-screen bg-black justify-items-center place-content-center">
                    <Spinner/>
                </div>
            }
        }>

            {move || {
                fetch_first_video_uid
                    .get()
                    .map(|_| {
                        fetch_video_uids.dispatch(());
                        view! { <ScrollingView/> }
                    })
            }}

        </Suspense>
    }
}
