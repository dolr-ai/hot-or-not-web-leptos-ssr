use candid::Principal;
use leptos::html::Audio;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::{overlay::VideoDetailsOverlay, video_loader::VideoView};
use crate::scrolling_post_view::MuteUnmuteOverlay;
use component::{back_btn::go_back_or_fallback, spinner::FullScreenSpinner};
use leptos_router::{components::Redirect, hooks::use_params, params::Params};
use state::{
    audio_state::AudioState,
    canisters::{auth_state, unauth_canisters},
};
use utils::{bg_url, send_wrap};
use yral_canisters_common::utils::posts::PostDetails;
#[derive(Params, PartialEq, Clone)]
struct PostParams {
    canister_id: Principal,
    post_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum PostFetchError {
    Invalid,
    Unavailable,
    GetUid(String),
}

#[component]
fn SinglePostViewInner(post: PostDetails) -> impl IntoView {
    let AudioState { muted, volume } = expect_context();
    let bg_url = bg_url(&post.uid);
    let win_audio_ref = NodeRef::<Audio>::new();
    let to_load = Memo::new(|_| true);

    view! {
        <div class="w-dvw h-dvh">
            <div class="overflow-hidden relative w-full h-full bg-transparent">
                <div
                    class="absolute top-0 left-0 w-full h-full bg-center bg-cover z-1 blur-lg"
                    style:background-color="rgb(0, 0, 0)"
                    style:background-image=format!("url({bg_url})")
                ></div>
                <audio
                    class="sr-only"
                    node_ref=win_audio_ref
                    preload="auto"
                    src="/img/hotornot/chaching.m4a"
                />
                <VideoDetailsOverlay post=post.clone() prev_post=None win_audio_ref />
                <VideoView post=Some(post.into()) muted volume autoplay_at_render=true to_load />
            </div>
            <MuteUnmuteOverlay muted />
        </div>
    }
    .into_any()
}

#[component]
fn UnavailablePost() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2 justify-center items-center bg-black h-dvh w-dvw">
            <span class="text-lg text-white md:text-xl lg:text-2xl">Post is unavailable</span>
            <button
                on:click=|_| go_back_or_fallback("/")
                class="py-2 px-4 text-center text-white rounded-full bg-primary-600"
            >
                Go back
            </button>
        </div>
    }
}

#[component]
pub fn SinglePost() -> impl IntoView {
    let params = use_params::<PostParams>();

    let auth = auth_state();

    let fetch_post = Resource::new(params, move |params| {
        send_wrap(async move {
            let params = params.map_err(|_| PostFetchError::Invalid)?;
            let unauth_cans = unauth_canisters();
            let post_uid = if let Some(canisters) = auth.auth_cans_if_available() {
                canisters
                    .get_post_details(params.canister_id, params.post_id.clone())
                    .await
            } else {
                let canisters = unauth_cans;
                canisters
                    .get_post_details(params.canister_id, params.post_id.clone())
                    .await
            };
            post_uid
                .map_err(|e| PostFetchError::GetUid(e.to_string()))
                .and_then(|post| post.ok_or(PostFetchError::Unavailable))
        })
    });

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || {
                fetch_post
                    .get()
                    .map(|post| match post {
                        Ok(post) => view! { <SinglePostViewInner post /> }.into_any(),
                        Err(PostFetchError::Invalid) => view! { <Redirect path="/" /> }.into_any(),
                        Err(PostFetchError::Unavailable) => view! { <UnavailablePost /> }.into_any(),
                        Err(PostFetchError::GetUid(e)) => {
                            view! { <Redirect path=format!("/error?err={e}") /> }.into_any()
                        }
                    })
            }}

        </Suspense>
    }
}
