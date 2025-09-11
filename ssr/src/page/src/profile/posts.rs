use leptos::prelude::*;
use leptos_icons::*;

use candid::Principal;

use component::profile_placeholders::NoMorePostsGraphic;
use state::canisters::{auth_state, unauth_canisters};
use utils::{
    bg_url, event_streaming::events::ProfileViewVideo, mixpanel::mixpanel_events::*,
    profile::PostsProvider,
};

use super::ic::ProfileStream;
use super::ProfilePostsContext;
use leptos::html;
use yral_canisters_common::utils::posts::PostDetails;

#[component]
fn Post(
    details: PostDetails,
    user_canister: Principal,
    _ref: NodeRef<html::Div>,
    post_index: Option<usize>,
) -> impl IntoView {
    let image_error = RwSignal::new(false);

    let profile_post_url = format!("/profile/{user_canister}/post/{}", details.post_id);

    let handle_image_error =
        move |_| _ = image_error.try_update(|image_error| *image_error = !*image_error);

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    let post_details = details.clone();
    let video_click = move || {
        ProfileViewVideo.send_event(ev_ctx, post_details.clone());
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            let logged_in_canister = global.canister_id.clone();
            MixPanelEvent::track_video_clicked_profile(
                global,
                post_details.poster_principal.to_text(),
                post_details.likes,
                post_details.views,
                true,
                post_details.uid.clone(),
                MixpanelPostGameType::HotOrNot,
                MixpanelVideoClickedCTAType::VideoPlay,
                post_index.map(|i| i as u64 + 1),
                user_canister.to_text() == logged_in_canister,
                post_details.is_nsfw,
                "profile".to_string(),
            );
        }
    };

    view! {
        <div node_ref=_ref class="relative w-full basis-1/3 md:basis-1/4 xl:basis-1/5">
            <div class="relative m-2 h-full rounded-md border aspect-9/16 border-white/20">
                <a class="w-full h-full" href=profile_post_url on:click=move |_| video_click()>
                    <Show
                        when=image_error
                        fallback=move || {
                            view! {
                                <img
                                    class="object-cover w-full h-full"
                                    on:error=handle_image_error
                                    src=bg_url(details.uid.clone())
                                />
                            }
                        }
                    >

                        <div class="flex flex-col items-center place-content-center h-full text-center text-white">
                            <Icon attr:class="h-8 w-8" icon=icondata::TbCloudX />
                            <span class="text-md">Not Available</span>
                        </div>
                    </Show>

                    <div class="grid absolute bottom-1 left-1 grid-cols-2 gap-1 items-center">
                        <Icon
                            attr:class="h-5 w-5 p-1 text-primary-500 rounded-full bg-black/30"
                            icon=icondata::AiHeartOutlined
                        />
                        <span class="text-xs text-white">{details.likes}</span>
                    </div>
                    <div class="grid absolute right-1 bottom-1 grid-cols-2 gap-1 items-center">
                        <Icon
                            attr:class="h-5 w-5 p-1 text-white rounded-full bg-black/30"
                            icon=icondata::AiEyeOutlined
                        />
                        <span class="text-xs text-white">{details.views}</span>
                    </div>
                </a>
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn ProfilePosts(
    user_principal: Principal,
    user_canister: Principal,
    username: String,
) -> impl IntoView {
    let ProfilePostsContext {
        video_queue,
        start_index,
        ..
    } = expect_context();

    let provider = PostsProvider::new(
        unauth_canisters(),
        video_queue,
        start_index,
        username,
        user_principal,
        user_canister,
    );

    view! {
        <ProfileStream
            provider
            empty_graphic=NoMorePostsGraphic
            empty_text="No Videos Uploaded yet"
            children=move |details, _ref| {
                view! {
                    <Post
                        post_index=video_queue.get_untracked().get_index_of(&details)
                        details=details
                        user_canister=user_canister
                        _ref=_ref.unwrap_or_default()
                    />
                }
            }
        />
    }
    .into_any()
}
