mod ic;
pub mod overlay;
mod posts;
mod profile_iter;
pub mod profile_post;
mod speculation;
mod tokens;

use candid::Principal;
use component::{connect::ConnectLogin, spinner::FullScreenSpinner};
use indexmap::IndexSet;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;
use leptos_router::{components::Redirect, hooks::use_params, params::Params};
use posts::ProfilePosts;
use speculation::ProfileSpeculations;
use state::{
    app_state::AppState,
    canisters::{auth_state, unauth_canisters},
};
use tokens::ProfileTokens;
use utils::send_wrap;
use yral_canisters_common::utils::{posts::PostDetails, profile::ProfileDetails};

#[derive(Clone, Default)]
pub struct ProfilePostsContext {
    video_queue: RwSignal<IndexSet<PostDetails>>,
    start_index: RwSignal<usize>,
    current_index: RwSignal<usize>,
    queue_end: RwSignal<bool>,
}

#[derive(Params, PartialEq, Clone)]
struct ProfileParams {
    id: String,
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
            .unwrap_or_else(move |_| "tokens".to_string())
    });

    let current_tab = Memo::new(move |_| match tab.get().as_str() {
        "posts" => 0,
        "stakes" => 1,
        "tokens" => 2,
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
        <div class="relative flex flex-row w-11/12 md:w-9/12 text-center text-xl md:text-2xl">
            <a class=move || tab_class(0) href=move || format!("/profile/{user_principal}/posts")>
                <Icon icon=icondata::FiGrid />
            </a>
            <a
                class=move || tab_class(1)
                href=move || format!("/profile/{user_principal}/stakes")
            >
                <Icon icon=icondata::BsTrophy />
            </a>
            <a
                class=move || tab_class(2)
                href=move || format!("/profile/{user_principal}/tokens")
            >
                <Icon icon=icondata::AiDollarCircleOutlined />
            </a>
        </div>

        <div class="flex flex-col gap-y-12 justify-center pb-12 w-11/12 sm:w-7/12">
            <Show when=move || current_tab() == 0>
                <ProfilePosts user_canister />
            </Show>
            <Show when=move || current_tab() == 1>
                <ProfileSpeculations user_canister user_principal />
            </Show>
            <Show when=move || current_tab() == 2>
                <ProfileTokens user_canister user_principal />
            </Show>
        </div>
    }
}

#[component]
fn ProfileViewInner(user: ProfileDetails, user_canister: Principal) -> impl IntoView {
    let user_principal = user.principal;
    let username_or_principal = user.username_or_principal();
    let profile_pic = user.profile_pic_or_random();
    let display_name = user.display_name_or_fallback();
    let _earnings = user.lifetime_earnings;

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    view! {
        <div class="min-h-screen bg-black text-white overflow-y-auto pt-10 pb-12">
            <div class="grid grid-cols-1 gap-5 justify-normal justify-items-center w-full">
                <div class="flex flex-row w-11/12 sm:w-7/12 justify-center">
                    <div class="flex flex-col justify-center items-center">
                        <img
                            class="h-24 w-24 rounded-full"
                            alt=username_or_principal.clone()
                            src=profile_pic
                        />
                        <div class="flex flex-col text-center items-center">
                            <span
                                class="text-md text-white font-bold"
                                class=("w-full", is_connected)
                                class=("w-5/12", move || !is_connected())
                                class=("truncate", move || !is_connected())
                            >
                                {display_name}
                            </span>
                            <Suspense>
                            {move || auth.user_principal.get().map(|v| {
                                view! {
                                    <Show when=move || !is_connected() && v == Ok(user_principal)>
                                        <div class="md:w-4/12 w-6/12 pt-5">
                                            <ConnectLogin
                                                cta_location="profile"
                                                redirect_to=format!("/profile/posts")
                                            />
                                        </div>
                                    </Show>
                                }
                            })}
                            </Suspense>
                        </div>
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
    view! {
        <Title text=page_title />
    }
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
                let principal = auth.user_principal.await;
                match principal {
                    Ok(principal) => view! {
                        {move || tab().map(|tab| {
                            view! {
                                <Redirect path=format!("/profile/{principal}/{tab}") />
                            }
                        })}
                    }.into_any(),
                    Err(_) => view! {
                        <Redirect path="/" />
                    }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
pub fn ProfileView() -> impl IntoView {
    let params = use_params::<ProfileParams>();

    let param_principal = move || {
        params.with(|p| {
            let ProfileParams { id, .. } = p.as_ref().ok()?;
            Principal::from_text(id).ok()
        })
    };

    let auth = auth_state();
    let cans = unauth_canisters();
    let user_details = Resource::new(param_principal, move |profile_principal| {
        let cans = cans.clone();
        send_wrap(async move {
            let profile_principal =
                profile_principal.ok_or_else(|| ServerFnError::new("Invalid principal"))?;
            if let Some(user_can) = auth
                .auth_cans_if_available(cans.clone())
                .filter(|can| can.user_principal() == profile_principal)
            {
                return Ok::<_, ServerFnError>((
                    user_can.profile_details(),
                    user_can.user_canister(),
                ));
            }

            let user_canister = cans
                .get_individual_canister_by_user_principal(profile_principal)
                .await?
                .ok_or_else(|| ServerFnError::new("Failed to get user canister"))?;
            let user = cans.individual_user(user_canister).await;
            let user_details = user.get_profile_details().await?;

            Ok::<_, ServerFnError>((ProfileDetails::from(user_details), user_canister))
        })
    });

    view! {
        <ProfilePageTitle />
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let res = user_details.await;

                match res {
                    Ok((user, user_canister)) => view! {
                        <ProfileComponent user user_canister />
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
pub fn ProfileComponent(user: ProfileDetails, user_canister: Principal) -> impl IntoView {
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

    view! {
        <ProfileViewInner user user_canister />
    }
}
