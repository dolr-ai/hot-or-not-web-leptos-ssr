pub mod edit;
mod ic;
pub mod overlay;
mod posts;
mod profile_iter;
pub mod profile_post;
mod speculation;

use candid::Principal;
use component::{connect::ConnectLogin, icons::edit_icons::EditIcon, spinner::FullScreenSpinner};
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use indexmap::IndexSet;
use leptos::{html, portal::Portal, prelude::*};
use leptos_icons::*;
use leptos_meta::*;
use leptos_router::{
    components::Redirect,
    hooks::{use_navigate, use_params},
    params::Params,
};
use posts::ProfilePosts;
use speculation::ProfileSpeculations;
use state::{
    app_state::AppState,
    canisters::{auth_state, unauth_canisters},
};

use utils::{mixpanel::mixpanel_events::*, posts::FeedPostCtx, send_wrap, UsernameOrPrincipal};
use yral_canisters_common::utils::{posts::PostDetails, profile::ProfileDetails};

#[derive(Clone)]
pub struct ProfilePostsContext {
    video_queue: RwSignal<IndexSet<PostDetails>>,
    video_queue_for_feed: RwSignal<Vec<FeedPostCtx>>,
    start_index: RwSignal<usize>,
    current_index: RwSignal<usize>,
    queue_end: RwSignal<bool>,
}

impl Default for ProfilePostsContext {
    fn default() -> Self {
        let mut video_queue_for_feed = Vec::new();
        for i in 0..MAX_VIDEO_ELEMENTS_FOR_FEED {
            video_queue_for_feed.push(FeedPostCtx {
                key: i,
                value: RwSignal::new(None),
            });
        }

        Self {
            video_queue: RwSignal::new(IndexSet::new()),
            video_queue_for_feed: RwSignal::new(video_queue_for_feed),
            start_index: RwSignal::new(0),
            current_index: RwSignal::new(0),
            queue_end: RwSignal::new(false),
        }
    }
}

#[derive(Params, PartialEq, Clone)]
struct ProfileParams {
    id: UsernameOrPrincipal,
}

#[derive(Params, Clone, PartialEq)]
struct TabsParam {
    tab: String,
}

#[component]
fn ListSwitcher1(user_canister: Principal, user_principal: Principal) -> impl IntoView {
    let param = use_params::<TabsParam>();
    let tab = Signal::derive(move || {
        param
            .get()
            .map(|t| t.tab)
            .unwrap_or_else(move |_| "posts".to_string())
    });

    let auth = auth_state();
    let event_ctx = auth.event_ctx();
    let view_profile_clicked = move |cta_type: MixpanelProfileClickedCTAType| {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(event_ctx) {
            let logged_in_canister = global.canister_id.clone();
            MixPanelEvent::track_profile_tab_clicked(
                global,
                logged_in_canister == user_canister.to_text(),
                user_principal.to_text(),
                cta_type,
            );
        }
    };

    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(event_ctx) {
        let logged_in_caniser = global.canister_id.clone();
        MixPanelEvent::track_profile_page_viewed(
            global,
            logged_in_caniser == user_canister.to_text(),
            user_principal.to_string(),
        );
    }

    let current_tab = Memo::new(move |_| match tab.get().as_str() {
        "posts" => 0,
        "stakes" => 1,
        _ => 0,
    });

    let tab_class = move |tab_id: usize| {
        if tab_id == current_tab() {
            "text-primary-500 border-b-4 border-primary-500 flex justify-center w-full py-2"
        } else {
            "text-white flex justify-center w-full py-2"
        }
    };
    view! {
        <div class="flex relative flex-row w-11/12 text-xl text-center md:w-9/12 md:text-2xl">
            <a on:click=move |_| view_profile_clicked(MixpanelProfileClickedCTAType::Videos)  class=move || tab_class(0) href=move || format!("/profile/{user_principal}/posts")>
                <Icon icon=icondata::FiGrid />
            </a>
            <a on:click=move |_| view_profile_clicked(MixpanelProfileClickedCTAType::GamesPlayed) class=move || tab_class(1) href=move || format!("/profile/{user_principal}/stakes")>
                <Icon icon=icondata::BsTrophy />
            </a>
        </div>

        <div class="flex flex-col gap-y-12 justify-center pb-12 w-11/12 sm:w-7/12">
            <Show when=move || current_tab() == 0>
                <ProfilePosts user_canister user_principal />
            </Show>
            <Show when=move || current_tab() == 1>
                <ProfileSpeculations user_canister user_principal />
            </Show>
        </div>
    }
}

#[component]
fn ProfileViewInner(user: ProfileDetails) -> impl IntoView {
    let user_principal = user.principal;
    let user_canister = user.user_canister;
    let username_or_fallback = user.username_or_fallback();
    let profile_pic = user.profile_pic_or_random();
    let _earnings = user.lifetime_earnings;

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    let edit_icon_mount_point = NodeRef::<html::Div>::new();

    let ev_ctx = auth.event_ctx();
    let on_edit_click = move |ev: leptos::web_sys::MouseEvent| {
        ev.prevent_default();
        if let Some(props) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_edit_profile_clicked(props, "profile".into());
        }

        let nav = use_navigate();
        nav("/profile/edit", Default::default());
    };

    view! {
        <div class="overflow-y-auto pt-10 pb-12 min-h-screen text-white bg-black">
            <div class="grid grid-cols-1 gap-5 justify-items-center w-full justify-normal">
                <div class="flex flex-row justify-center w-11/12 sm:w-7/12">
                    <div class="flex flex-col justify-center items-center">
                        <div class="flex flex-row items-center justify-between w-full p-4 bg-neutral-900 rounded-lg">
                            <div class="flex flex-row items-center gap-4">
                                <img
                                    class="w-12 h-12 md:w-14 md:h-14 lg:w-16 lg:h-16 rounded-full"
                                    alt=username_or_fallback.clone()
                                    src=profile_pic
                                />
                                <div class="flex flex-col gap-2">
                                    <div node_ref=edit_icon_mount_point class="flex flex-row justify-between">
                                        <span class="font-bold text-neutral-50 text-lg line-clamp-1">
                                            @{username_or_fallback.clone()}
                                        </span>
                                    </div>
                                    <span class="text-neutral-400 text-sm line-clamp-1">
                                        {user_principal.to_text()}
                                    </span>
                                </div>
                            </div>
                        </div>
                        <Suspense>
                            {move || {
                                auth.user_principal
                                    .get()
                                    .map(|v| {
                                        let authenticated_princ = v.unwrap_or(Principal::anonymous());
                                        view! {
                                            <Show when=move || {
                                                !is_connected() && user_principal == authenticated_princ
                                            }>
                                                <div class="pt-5 w-6/12 md:w-4/12">
                                                    <ConnectLogin
                                                        cta_location="profile"
                                                        redirect_to=format!("/profile/posts")
                                                    />
                                                </div>
                                            </Show>
                                            <Show when=move || user_principal == authenticated_princ>
                                            {move || edit_icon_mount_point.get().map(|mount| view! {
                                                <Portal mount>
                                                    <a on:click=on_edit_click href="/profile/edit">
                                                        <Icon
                                                            icon=EditIcon
                                                            attr:class="text-2xl text-neutral-300"
                                                        />
                                                    </a>
                                                </Portal>
                                            })}
                                            </Show>
                                        }
                                    })
                            }}
                        </Suspense>
                    </div>
                </div>
                <ListSwitcher1 user_canister user_principal />
            </div>
        </div>
    }
    .into_any()
}

#[component]
fn ProfilePageTitle() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Profile";
    view! { <Title text=page_title /> }
}

#[component]
pub fn LoggedInUserProfileView() -> impl IntoView {
    let tab_params = use_params::<TabsParam>();
    let tab = move || tab_params.get().map(|p| p.tab).ok();
    let auth = auth_state();

    view! {
        <ProfilePageTitle />
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let id = auth.user_principal.await;
                match id {
                    Ok(id) => {
                        view! {
                            {move || {
                                tab()
                                    .map(move |tab| {
                                        view! {
                                            <Redirect path=format!("/profile/{id}/{tab}") />
                                        }
                                    })
                            }}
                        }
                            .into_any()
                    }
                    Err(_) => view! { <Redirect path="/" /> }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
pub fn ProfileView() -> impl IntoView {
    let params = use_params::<ProfileParams>();

    let param_id = move || {
        params.with(|p| {
            let ProfileParams { id, .. } = p.as_ref().ok()?;
            Some(id.clone())
        })
    };

    let auth = auth_state();
    let cans = unauth_canisters();
    let user_details = Resource::new(param_id, move |profile_id| {
        let cans = cans.clone();
        send_wrap(async move {
            let profile_id = profile_id.ok_or_else(|| ServerFnError::new("Invalid ID"))?;
            if let Some(user_can) = auth
                .auth_cans_if_available()
                .filter(|can| match &profile_id {
                    UsernameOrPrincipal::Principal(princ) => *princ == can.user_principal(),
                    UsernameOrPrincipal::Username(u) => {
                        Some(u) == can.profile_details().username.as_ref()
                    }
                })
            {
                return Ok::<_, ServerFnError>(Some(user_can.profile_details()));
            }

            let user_details = cans.get_profile_details(profile_id.to_string()).await?;

            Ok::<_, ServerFnError>(user_details)
        })
    });

    view! {
        <ProfilePageTitle />
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let res = async {
                    let maybe_user = user_details.await?;
                    if let Some(user) = maybe_user {
                        return Ok::<_, ServerFnError>(user);
                    }
                    // edge case: user is not logged in
                    let auth = auth_state();
                    let cans = auth.auth_cans().await?;
                    let my_details = cans.profile_details();
                    let id = untrack(param_id).expect("ID should be available");
                    match id {
                        UsernameOrPrincipal::Principal(princ) if princ == cans.user_principal() => {
                            Ok(my_details)
                        },
                        UsernameOrPrincipal::Username(username) if Some(&username) == my_details.username.as_ref() => {
                            Ok(my_details)
                        }
                        _ => Err(ServerFnError::new("User not found")),
                    }
                };

                match res.await {
                    Ok(user) => view! {
                        <ProfileComponent user />
                    }.into_any(),
                    _ => view! {
                        <Redirect path="/" />
                    }.into_any(),
                }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn ProfileComponent(user: ProfileDetails) -> impl IntoView {
    let ProfilePostsContext {
        video_queue,
        start_index,
        ..
    } = expect_context();

    video_queue.update_untracked(|v| {
        v.drain(..);
    });
    start_index.update_untracked(|idx| {
        *idx = 0;
    });

    view! { <ProfileViewInner user /> }
}
