use crate::{
    component::modal::Modal,
    state::canisters::{authenticated_canisters, Canisters},
    try_or_redirect_opt,
    utils::{
        route::failure_redirect,
        web::{copy_to_clipboard, share_url},
    },
};
use leptos::*;
use leptos_icons::*;
use leptos_use::use_window;

use super::video_iter::{post_liked_by_me, PostDetails};
use candid::Principal;

#[component]
fn LikeButtonPlaceHolder() -> impl IntoView {
    view! {
        <button disabled>
            <Icon
                class="drop-shadow-lg text-neutral-400 animate-pulse"
                icon=icondata::AiHeartFilled
            />
        </button>
    }
}

#[component]
fn LikeButton(
    canisters: Canisters<true>,
    post_canister: Principal,
    post_id: u64,
    likes: RwSignal<u64>,
    initial_liked: bool,
) -> impl IntoView {
    let liked = create_rw_signal(initial_liked);
    let icon_class = Signal::derive(move || {
        if liked() {
            TextProp::from("fill-primary-600")
        } else {
            TextProp::from("text-white")
        }
    });
    let icon_style = Signal::derive(move || {
        if liked() {
            Some(TextProp::from("filter: drop-shadow(2px 0 0 white) drop-shadow(-2px 0 0 white) drop-shadow(0 2px 0 white) drop-shadow(0 -2px 0 white);"))
        } else {
            None
        }
    });
    let like_toggle = create_action(move |&()| {
        let canisters = canisters.clone();

        async move {
            batch(move || {
                if liked.get_untracked() {
                    likes.update(|l| *l -= 1);
                    liked.set(false)
                } else {
                    likes.update(|l| *l += 1);
                    liked.set(true);
                }
            });
            let individual = canisters.individual_user(post_canister);
            match individual
                .update_post_toggle_like_status_by_caller(post_id)
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    log::warn!("Error toggling like status: {:?}", e);
                    liked.update(|l| *l = !*l);
                }
            }
        }
    });

    view! {
        <button
            on:click=move |_| like_toggle.dispatch(())
            class="drop-shadow-lg disabled:animate-pulse"
            disabled=like_toggle.pending()
        >
            <Icon class=icon_class style=icon_style icon=icondata::AiHeartFilled/>
        </button>
    }
}

#[component]
fn LikeLoader(
    canisters: Canisters<true>,
    post: PostDetails,
    likes: RwSignal<u64>,
) -> impl IntoView {
    let can_c = canisters.clone();
    let liked = create_resource(
        || (),
        move |_| {
            let canisters = can_c.clone();
            async move {
                if let Some(liked) = post.liked_by_user {
                    return liked;
                }
                match post_liked_by_me(&canisters, post.canister_id, post.post_id).await {
                    Ok(liked) => liked,
                    Err(e) => {
                        failure_redirect(e);
                        false
                    }
                }
            }
        },
    );
    let canisters = store_value(canisters);

    view! {
        <Suspense fallback=LikeButtonPlaceHolder>
            {move || {
                liked()
                    .map(move |initial_liked| {
                        view! {
                            <LikeButton
                                canisters=canisters.get_value()
                                post_canister=post.canister_id
                                post_id=post.post_id
                                likes
                                initial_liked
                            />
                        }
                    })
            }}

        </Suspense>
    }
}

#[component]
fn LikeAndAuthCanLoader(post: PostDetails) -> impl IntoView {
    let auth_cans = authenticated_canisters();
    let likes = create_rw_signal(post.likes);
    let post = store_value(post);

    view! {
        <div class="flex flex-col gap-1 items-center">
            <Suspense fallback=LikeButtonPlaceHolder>
                {move || {
                    auth_cans
                        .get()
                        .and_then(|canisters| {
                            let canisters = try_or_redirect_opt!(canisters)?;
                            Some(view! { <LikeLoader canisters post=post.get_value() likes/> })
                        })
                }}

            </Suspense>
            <span class="text-sm md:text-md">{likes}</span>
        </div>
    }
}

#[component]
pub fn VideoDetailsOverlay(post: PostDetails) -> impl IntoView {
    let show_share = create_rw_signal(false);
    let base_url = || {
        use_window()
            .as_ref()
            .and_then(|w| w.location().origin().ok())
    };
    let video_url = move || {
        base_url()
            .map(|b| format!("{b}/hot-or-not/{}/{}", post.canister_id, post.post_id))
            .unwrap_or_default()
    };

    let share = move || {
        let url = video_url();
        if share_url(&url).is_some() {
            return Some(());
        }
        show_share.set(true);
        Some(())
    };

    let profile_url = format!("/profile/{}", post.poster_principal.to_text());
    let post_c = post.clone();

    view! {
        <div class="flex flex-row flex-nowrap justify-between items-end pb-16 px-2 md:px-6 w-full text-white absolute bottom-0 left-0 bg-transparent z-[4]">
            <div class="flex flex-col gap-2 w-9/12">
                <div class="flex flex-row items-center gap-2 min-w-0">
                    <a
                        href=profile_url
                        class="w-10 md:w-12 h-10 md:h-12 overflow-clip rounded-full border-white border-2"
                    >
                        <img class="h-full w-full object-cover" src=post.propic_url/>
                    </a>
                    <div class="flex flex-col w-7/12">
                        <span class="text-md md:text-lg font-bold truncate">
                            {post.display_name}
                        </span>
                        <span class="flex flex-row gap-1 items-center text-sm md:text-md">
                            <Icon icon=icondata::AiEyeOutlined/>
                            {post.views}
                        </span>
                    </div>
                </div>
                <ExpandableText description=post.description/>
            </div>
            <div class="flex flex-col gap-6 items-end w-3/12 text-4xl">
                <a href="/refer-earn">
                    <Icon class="drop-shadow-lg" icon=icondata::AiGiftFilled/>
                </a>
                <LikeAndAuthCanLoader post=post_c/>
                <button on:click=move |_| _ = share()>
                    <Icon class="drop-shadow-lg" icon=icondata::BsSendFill/>
                </button>
            </div>
        </div>
        <Modal show=show_share>
            <div class="flex flex-col justify-center items-center gap-4 text-white">
                <span class="text-lg">Share</span>
                <div class="flex flex-row w-full gap-2">
                    <p class="text-md max-w-full bg-white/10 rounded-full p-2 overflow-x-scroll whitespace-nowrap">
                        {video_url}
                    </p>
                    <button on:click=move |_| _ = copy_to_clipboard(&video_url())>
                        <Icon class="text-xl" icon=icondata::FaCopyRegular/>
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn ExpandableText(description: String) -> impl IntoView {
    let truncated = create_rw_signal(true);

    view! {
        <span
            class="text-sm md:text-md ms-2 md:ms-4 w-full"
            class:truncate=truncated

            on:click=move |_| truncated.update(|e| *e = !*e)
        >
            {description}
        </span>
    }
}

#[component]
pub fn HomeButtonOverlay() -> impl IntoView {
    view! {
        <div class="flex w-full items-center justify-center pt-4 absolute top-0 left-0 bg-transparent z-[4]">
            // <div class="flex justify-center items-center">
            // <img src="/img/yral-logo.svg" alt="Logo"/>
            // </div>
            <div class="rounded-full p-2 text-white bg-black/20">
                <div class="flex flex-row items-center gap-1 py-2 px-6 rounded-full">
                    // <Icon class="w-3 h-3" icon=HomeSymbolFilled/>
                    <span class="font-sans font-semibold">Home Feed</span>
                </div>
            </div>
        </div>
    }
}
