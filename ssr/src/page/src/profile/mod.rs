pub mod edit;
mod ic;
pub mod overlay;
mod posts;
mod profile_iter;
pub mod profile_post;
mod speculation;

use candid::Principal;
use component::{
    connect::ConnectLogin, icons::notification_icon::NotificationIcon,
    notification::NotificationPage, spinner::FullScreenSpinner,
};
use consts::MAX_VIDEO_ELEMENTS_FOR_FEED;
use indexmap::IndexSet;
use leptos::{html, prelude::*};
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

use component::{infinite_scroller::InfiniteScroller, overlay::ShadowOverlay};
use utils::{mixpanel::mixpanel_events::*, posts::FeedPostCtx, send_wrap, UsernameOrPrincipal};
use yral_canisters_client::user_info_service::{Result1, Result2};
use yral_canisters_common::{
    cursored_data::{CursoredDataProvider, KeyedData, PageEntry},
    utils::{
        posts::PostDetails,
        profile::{propic_from_principal, ProfileDetails},
    },
    Canisters,
};

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;
#[cfg(feature = "ssr")]
use leptos::server;

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

// Follower/Following data types for the popup
#[derive(Clone, Debug)]
struct FollowerData {
    pub principal_id: Principal,
    pub username: Option<String>,
    pub profile_pic: Option<String>,
    pub caller_follows: bool,
}

// Error type for providers
#[derive(Debug, Clone)]
struct FollowerProviderError(String);

impl std::fmt::Display for FollowerProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for FollowerProviderError {}

impl KeyedData for FollowerData {
    type Key = Principal;

    fn key(&self) -> Self::Key {
        self.principal_id
    }
}

// Provider for fetching followers
#[derive(Clone)]
struct FollowersProvider {
    user_principal: Principal,
    cursor: RwSignal<Option<Principal>>,
}

impl FollowersProvider {
    fn new(user_principal: Principal) -> Self {
        Self {
            user_principal,
            cursor: RwSignal::new(None),
        }
    }
}

impl CursoredDataProvider for FollowersProvider {
    type Data = FollowerData;
    type Error = FollowerProviderError;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        let limit = (end - start).min(20) as u64;

        // Get default canisters for query call
        let canisters = Canisters::default();
        let service = canisters.user_info_service().await;

        // Fetch followers
        let result = service
            .get_followers(self.user_principal, self.cursor.get_untracked(), limit)
            .await
            .map_err(|e| FollowerProviderError(format!("Failed to fetch followers: {}", e)))?;

        let response = match result {
            Result1::Ok(resp) => resp,
            Result1::Err(e) => return Err(FollowerProviderError(format!("API error: {}", e))),
        };

        // Update cursor for next fetch
        if let Some(next_cursor) = response.next_cursor {
            self.cursor.set(Some(next_cursor));
        }

        // Convert FollowerItem to FollowerData
        // For now, we'll use principal as username and generate profile pic
        let data = response
            .followers
            .into_iter()
            .map(|item| FollowerData {
                principal_id: item.principal_id,
                username: Some(format!(
                    "@{}",
                    item.principal_id
                        .to_text()
                        .chars()
                        .take(8)
                        .collect::<String>()
                )),
                profile_pic: Some(propic_from_principal(item.principal_id)),
                caller_follows: item.caller_follows,
            })
            .collect();

        Ok(PageEntry {
            data,
            end: response.next_cursor.is_none(),
        })
    }
}

// Provider for fetching following
#[derive(Clone)]
struct FollowingProvider {
    user_principal: Principal,
    cursor: RwSignal<Option<Principal>>,
}

impl FollowingProvider {
    fn new(user_principal: Principal) -> Self {
        Self {
            user_principal,
            cursor: RwSignal::new(None),
        }
    }
}

impl CursoredDataProvider for FollowingProvider {
    type Data = FollowerData;
    type Error = FollowerProviderError;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        let limit = (end - start).min(20) as u64;

        // Get default canisters for query call
        let canisters = Canisters::default();
        let service = canisters.user_info_service().await;

        // Fetch following
        let result = service
            .get_following(self.user_principal, self.cursor.get_untracked(), limit)
            .await
            .map_err(|e| FollowerProviderError(format!("Failed to fetch following: {}", e)))?;

        let response = match result {
            Result2::Ok(resp) => resp,
            Result2::Err(e) => return Err(FollowerProviderError(format!("API error: {}", e))),
        };

        // Update cursor for next fetch
        if let Some(next_cursor) = response.next_cursor {
            self.cursor.set(Some(next_cursor));
        }

        // Convert FollowerItem to FollowerData
        let data = response
            .following
            .into_iter()
            .map(|item| FollowerData {
                principal_id: item.principal_id,
                username: Some(format!(
                    "@{}",
                    item.principal_id
                        .to_text()
                        .chars()
                        .take(8)
                        .collect::<String>()
                )),
                profile_pic: Some(propic_from_principal(item.principal_id)),
                caller_follows: item.caller_follows,
            })
            .collect();

        Ok(PageEntry {
            data,
            end: response.next_cursor.is_none(),
        })
    }
}

#[server]
pub async fn follow_user_action(target: Principal) -> Result<(), ServerFnError> {
    use auth::server_impl::extract_identity_impl;
    use yral_canisters_common::Canisters;

    // Extract identity from cookies
    let identity_wire = extract_identity_impl()
        .await?
        .ok_or_else(|| ServerFnError::new("User not authenticated"))?;

    // Authenticate with network using the identity
    let canisters = match Canisters::<true>::authenticate_with_network(identity_wire).await {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::ServerError(format!("Auth failed: {}", e))),
    };

    // Get the service canister
    let service = canisters.user_info_service().await;

    // Call follow_user
    match service.follow_user(target).await {
        Ok(result) => match result {
            yral_canisters_client::user_info_service::Result_::Ok => Ok(()),
            yral_canisters_client::user_info_service::Result_::Err(e) => {
                Err(ServerFnError::ServerError(format!("Follow failed: {}", e)))
            }
        },
        Err(e) => Err(ServerFnError::ServerError(format!("Network error: {}", e))),
    }
}

#[server]
pub async fn unfollow_user_action(target: Principal) -> Result<(), ServerFnError> {
    use auth::server_impl::extract_identity_impl;
    use yral_canisters_common::Canisters;

    // Extract identity from cookies
    let identity_wire = extract_identity_impl()
        .await?
        .ok_or_else(|| ServerFnError::new("User not authenticated"))?;

    // Authenticate with network using the identity
    let canisters = match Canisters::<true>::authenticate_with_network(identity_wire).await {
        Ok(c) => c,
        Err(e) => return Err(ServerFnError::ServerError(format!("Auth failed: {}", e))),
    };

    // Get the service canister
    let service = canisters.user_info_service().await;

    // Call unfollow_user
    match service.unfollow_user(target).await {
        Ok(result) => match result {
            yral_canisters_client::user_info_service::Result_::Ok => Ok(()),
            yral_canisters_client::user_info_service::Result_::Err(e) => Err(
                ServerFnError::ServerError(format!("Unfollow failed: {}", e)),
            ),
        },
        Err(e) => Err(ServerFnError::ServerError(format!("Network error: {}", e))),
    }
}

#[component]
fn FollowButton(
    user_principal: Principal,
    caller_follows_user: Option<bool>,
    user_follows_caller: Option<bool>,
) -> impl IntoView {
    let (is_following, set_is_following) = signal(caller_follows_user.unwrap_or(false));
    let (is_loading, set_is_loading) = signal(false);

    let follow_action = Action::new(move |_: &()| {
        let target = user_principal;
        async move {
            set_is_loading.set(true);
            let result = follow_user_action(target).await;
            set_is_loading.set(false);
            if result.is_ok() {
                set_is_following.set(true);
            }
            result
        }
    });

    let unfollow_action = Action::new(move |_: &()| {
        let target = user_principal;
        async move {
            set_is_loading.set(true);
            let result = unfollow_user_action(target).await;
            set_is_loading.set(false);
            if result.is_ok() {
                set_is_following.set(false);
            }
            result
        }
    });

    let button_text = move || {
        if is_loading.get() {
            "Loading..."
        } else if is_following.get() {
            "Unfollow"
        } else if user_follows_caller.unwrap_or(false) {
            "Follow Back"
        } else {
            "Follow"
        }
    };

    let button_class = move || {
        if is_following.get() {
            "w-full bg-transparent border border-neutral-700 rounded-lg px-5 py-2.5 flex items-center justify-center"
        } else if user_follows_caller.unwrap_or(false) {
            "w-full bg-primary-600 border border-primary-600 rounded-lg px-5 py-2.5 flex items-center justify-center"
        } else {
            "w-full bg-[#212121] border border-neutral-700 rounded-lg px-5 py-2.5 flex items-center justify-center"
        }
    };

    let on_click = move |_| {
        if !is_loading.get() {
            if is_following.get() {
                unfollow_action.dispatch(());
            } else {
                follow_action.dispatch(());
            }
        }
    };

    view! {
        <button
            on:click=on_click
            class=button_class
            disabled=is_loading
        >
            <span class="font-semibold text-sm text-neutral-50">
                {button_text}
            </span>
        </button>
    }
}

// Component for displaying a user in the followers/following list
#[component]
fn UserListItem(data: FollowerData, node_ref: Option<NodeRef<html::Div>>) -> impl IntoView {
    let navigate = use_navigate();
    let auth = auth_state();

    // Get current user's principal
    let current_user_principal = auth
        .user_principal
        .get()
        .and_then(|res| res.ok())
        .unwrap_or(Principal::anonymous());

    // Don't show follow button for own profile
    let show_follow_button = data.principal_id != current_user_principal;

    let principal_for_nav = data.principal_id;

    // Build the view with optional node_ref
    if let Some(nr) = node_ref {
        view! {
            <div
                class="flex items-center justify-between px-4 py-3 hover:bg-neutral-900/50 transition-colors"
                node_ref=nr
            >
            // User info section
            <div
                class="flex items-center gap-3 flex-1 cursor-pointer"
                on:click=move |_| {
                    navigate(&format!("/profile/{}", principal_for_nav), Default::default());
                }
            >
                // Avatar
                <img
                    class="w-8 h-8 rounded-full object-cover"
                    src=data.profile_pic.clone().unwrap_or_default()
                    alt=data.username.clone().unwrap_or_default()
                />

                // Username
                <span class="font-semibold text-sm text-neutral-50">
                    {data.username.clone().unwrap_or_else(|| format!("@{}", data.principal_id.to_text().chars().take(8).collect::<String>()))}
                </span>
            </div>

            // Follow button
            <Show when=move || show_follow_button>
                <div class="shrink-0">
                    <FollowButton
                        user_principal=data.principal_id
                        caller_follows_user=Some(data.caller_follows)
                        user_follows_caller=None
                    />
                </div>
            </Show>
        </div>
        }.into_any()
    } else {
        view! {
            <div
                class="flex items-center justify-between px-4 py-3 hover:bg-neutral-900/50 transition-colors"
            >
                // User info section
                <div
                    class="flex items-center gap-3 flex-1 cursor-pointer"
                    on:click=move |_| {
                        navigate(&format!("/profile/{}", principal_for_nav), Default::default());
                    }
                >
                    // Avatar
                    <img
                        class="w-8 h-8 rounded-full object-cover"
                        src=data.profile_pic.clone().unwrap_or_default()
                        alt=data.username.clone().unwrap_or_default()
                    />

                    // Username
                    <span class="font-semibold text-sm text-neutral-50">
                        {data.username.clone().unwrap_or_else(|| format!("@{}", data.principal_id.to_text().chars().take(8).collect::<String>()))}
                    </span>
                </div>

                // Follow button
                <Show when=move || show_follow_button>
                    <div class="shrink-0">
                        <FollowButton
                            user_principal=data.principal_id
                            caller_follows_user=Some(data.caller_follows)
                            user_follows_caller=None
                        />
                    </div>
                </Show>
            </div>
        }.into_any()
    }
}

// Popup component for showing followers/following lists
#[component]
fn FollowersFollowingPopup(
    show: RwSignal<bool>,
    user_principal: Principal,
    username: String,
    initial_tab: usize, // 0 = Followers, 1 = Following
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(initial_tab);

    // Create providers for each tab using StoredValue for stable references
    let followers_provider = StoredValue::new(FollowersProvider::new(user_principal));
    let following_provider = StoredValue::new(FollowingProvider::new(user_principal));

    let tab_class = move |tab_idx: usize| {
        if active_tab() == tab_idx {
            "flex-1 py-2.5 text-sm font-semibold text-neutral-50 border-b-2 border-primary-600 text-center transition-all"
        } else {
            "flex-1 py-2.5 text-sm font-medium text-neutral-400 text-center cursor-pointer hover:text-neutral-300 transition-all"
        }
    };

    view! {
        <ShadowOverlay show>
            <div class="flex flex-col bg-neutral-900 rounded-lg w-full max-w-md max-h-[70vh] mx-4">
                // Header
                <div class="flex items-center justify-between p-4 border-b border-neutral-700">
                    <h2 class="text-lg font-bold text-neutral-50">
                        {username.clone()}
                    </h2>
                    <button
                        on:click=move |_| show.set(false)
                        class="p-1 text-neutral-400 hover:text-neutral-200 transition-colors"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                </div>

                // Tabs
                <div class="flex border-b border-neutral-700">
                    <button
                        class=move || tab_class(0)
                        on:click=move |_| set_active_tab.set(0)
                    >
                        "Followers"
                    </button>
                    <button
                        class=move || tab_class(1)
                        on:click=move |_| set_active_tab.set(1)
                    >
                        "Following"
                    </button>
                </div>

                // Content
                <div class="flex-1 overflow-y-auto">
                    <Show
                        when=move || active_tab() == 0
                        fallback=move || {
                            // Following tab
                            view! {
                                <InfiniteScroller
                                    provider=following_provider.get_value()
                                    fetch_count=20
                                    children=move |item: FollowerData, node_ref| {
                                        view! { <UserListItem data=item node_ref=node_ref /> }
                                    }
                                    empty_content=move || view! {
                                        <div class="flex flex-col items-center justify-center py-12 px-4">
                                            <p class="text-neutral-400 text-center">
                                                "Not following anyone yet"
                                            </p>
                                        </div>
                                    }
                                />
                            }
                        }
                    >
                        // Followers tab
                        <InfiniteScroller
                            provider=followers_provider.get_value()
                            fetch_count=20
                            children=move |item: FollowerData, node_ref| {
                                view! { <UserListItem data=item node_ref=node_ref /> }
                            }
                            empty_content=move || view! {
                                <div class="flex flex-col items-center justify-center py-12 px-4">
                                    <p class="text-neutral-400 text-center">
                                        "No followers yet"
                                    </p>
                                </div>
                            }
                        />
                    </Show>
                </div>
            </div>
        </ShadowOverlay>
    }
}

#[component]
fn ListSwitcher1(
    user_canister: Principal,
    user_principal: Principal,
    username: String,
) -> impl IntoView {
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
                <ProfilePosts user_canister user_principal username=username.clone()/>
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
    let username_for_header = username_or_fallback.clone();
    let profile_pic = user.profile_pic_or_random();
    let _earnings = user.lifetime_earnings;

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    let _edit_icon_mount_point = NodeRef::<html::Div>::new();

    let ev_ctx = auth.event_ctx();
    let nav = use_navigate();
    let nav_clone2 = nav.clone();
    let nav_menu = nav.clone();

    // Notification panel signal
    let notification_panel = RwSignal::new(false);

    // Followers/Following popup signals
    let show_followers_popup = RwSignal::new(false);
    let popup_initial_tab = RwSignal::new(0usize); // 0 = Followers, 1 = Following

    // Get the authenticated user principal for comparison
    let auth_user_principal = auth.user_principal;

    // Simple signal that will be set in the Suspense block
    let is_own_profile = RwSignal::new(false);

    // Get actual data from user ProfileDetails
    let followers_count = user.followers_cnt;
    let following_count = user.following_cnt;
    let games_played = 100u64; // TODO: Get actual games played count when available
    let bio = user.bio.clone().unwrap_or_else(|| "".to_string());
    let website_url = user.website_url.clone().unwrap_or_else(|| "".to_string());

    view! {
        <div class="overflow-y-auto pb-12 min-h-screen text-white bg-black">
            <Show when=move || notification_panel.get()>
                <NotificationPage close=notification_panel />
            </Show>
            // Header with title and navigation icons - aligned with content width
            <div class="flex justify-center w-full bg-black">
                <div class="flex h-12 items-center justify-between px-4 sm:px-0 py-3 w-11/12 sm:w-7/12">
                    <p class="font-bold text-xl text-neutral-50">
                        {move || {
                            if is_own_profile.get() {
                                "My Profile".to_string()
                            } else {
                                format!("@{}", username_for_header.clone())
                            }
                        }}
                    </p>
                    <div class="flex gap-5 items-center justify-end">
                        <Show when=is_own_profile>
                            <button
                                on:click=move |_| {
                                    notification_panel.set(true);
                                }
                            >
                                <NotificationIcon show_dot=false class="w-6 h-6 text-neutral-300" />
                            </button>
                        </Show>
                        <Show when=is_own_profile>
                            {
                                let nav_menu = nav_menu.clone();
                                view! {
                                    <button
                                        on:click=move |_| {
                                            nav_menu("/menu", Default::default());
                                        }
                                        class="p-1"
                                    >
                                        <Icon icon=icondata::BiMenuRegular attr:class="text-2xl text-neutral-300" />
                                    </button>
                                }
                            }
                        </Show>
                    </div>
                </div>
            </div>

            <div class="flex flex-col gap-5 items-center w-full pt-5">
                <div class="flex flex-col gap-5 w-11/12 sm:w-7/12">
                    // Profile header with avatar and stats
                    <div class="flex gap-6 items-start">
                        // Avatar
                        <img
                            class="w-[60px] h-[60px] rounded-full shrink-0"
                            alt=username_or_fallback.clone()
                            src=profile_pic
                        />

                        // Stats section
                        <div class="flex-1 flex flex-col gap-2.5">
                            <div class="flex gap-0 items-start">
                                // Followers
                                <button
                                    class="flex flex-col gap-1 items-center text-center w-[85px] cursor-pointer hover:opacity-80 transition-opacity"
                                    on:click=move |_| {
                                        popup_initial_tab.set(0);
                                        show_followers_popup.set(true);
                                    }
                                >
                                    <span class="font-semibold text-base text-neutral-50">
                                        {followers_count}
                                    </span>
                                    <span class="font-normal text-sm text-neutral-50">
                                        "Followers"
                                    </span>
                                </button>

                                // Following
                                <button
                                    class="flex flex-col gap-1 items-center text-center w-[88px] cursor-pointer hover:opacity-80 transition-opacity"
                                    on:click=move |_| {
                                        popup_initial_tab.set(1);
                                        show_followers_popup.set(true);
                                    }
                                >
                                    <span class="font-semibold text-base text-neutral-50">
                                        {following_count}
                                    </span>
                                    <span class="font-normal text-sm text-neutral-50">
                                        "Following"
                                    </span>
                                </button>

                                // Games Played
                                <div class="flex-1 flex flex-col gap-1 items-start">
                                    <span class="font-semibold text-base text-neutral-50">
                                        {games_played}
                                    </span>
                                    <span class="font-normal text-sm text-neutral-50 whitespace-nowrap">
                                        "Games Played"
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Username and bio section
                    <div class="flex flex-col gap-4">
                        <div class="flex flex-col gap-2">
                            <span class="font-semibold text-sm text-neutral-50">
                                {username_or_fallback.clone()}
                            </span>
                            <div class="font-normal text-xs text-neutral-50">
                                {(!bio.is_empty() || !website_url.is_empty()).then(|| view! {
                                    <p>
                                        {(!bio.is_empty()).then(|| view! {
                                            <>
                                                {bio.clone()}
                                                {(!website_url.is_empty()).then(|| view! { <br /> })}
                                            </>
                                        })}
                                        {(!website_url.is_empty()).then(|| view! {
                                            <span class="text-[#3d8eff]">{website_url.clone()}</span>
                                        })}
                                    </p>
                                })}
                            </div>
                        </div>

                        // Social Links button
                        <div class="flex gap-1 items-center justify-center bg-[#212121] rounded-full px-2.5 py-1.5 self-start">
                            <Icon icon=icondata::FiPlus attr:class="text-base text-neutral-300" />
                            <span class="font-semibold text-xs text-neutral-50 whitespace-nowrap">
                                "Social Links"
                            </span>
                        </div>
                    </div>

                    // Edit Profile button for own profile or Follow button for others
                    <Suspense>
                        {{
                            let nav = nav_clone2;
                            move || {
                                auth_user_principal
                                    .get()
                                    .map(|v| {
                                        let authenticated_princ = v.unwrap_or(Principal::anonymous());

                                        // Set the is_own_profile signal here
                                        is_own_profile.set(user_principal == authenticated_princ);

                                        let nav = nav.clone();
                                        view! {
                                            // Show Edit Profile button for own profile
                                            <Show when=move || user_principal == authenticated_princ>
                                                <button
                                                    on:click={let nav = nav.clone(); move |ev: leptos::web_sys::MouseEvent| {
                                                        ev.prevent_default();
                                                        if let Some(props) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                                                            MixPanelEvent::track_edit_profile_clicked(props, "profile".into());
                                                        }
                                                        nav("/profile/edit", Default::default());
                                                    }}
                                                class="w-full bg-[#212121] border border-neutral-700 rounded-lg px-5 py-2.5 flex items-center justify-center"
                                            >
                                                <span class="font-semibold text-sm text-neutral-50">
                                                    "Edit Profile"
                                                </span>
                                            </button>
                                        </Show>
                                        // Show Follow button for other profiles
                                        <Show when=move || user_principal != authenticated_princ>
                                            <FollowButton
                                                user_principal=user_principal
                                                caller_follows_user=user.caller_follows_user
                                                user_follows_caller=user.user_follows_caller
                                            />
                                        </Show>
                                        <Show when=move || {
                                            !is_connected() && user_principal == authenticated_princ
                                        }>
                                            <div class="w-full">
                                                <ConnectLogin
                                                    cta_location="profile"
                                                    redirect_to=format!("/profile/posts")
                                                />
                                            </div>
                                        </Show>
                                    }
                                    })
                            }
                        }}
                    </Suspense>

                    // Divider
                    <div class="w-full h-[1px] bg-[#212121]" />
                </div>

                // Tabs
                <ListSwitcher1 user_canister user_principal username=username_or_fallback.clone()/>
            </div>

            // Followers/Following popup
            <Show when=move || show_followers_popup.get()>
                <FollowersFollowingPopup
                    show=show_followers_popup
                    user_principal=user_principal
                    username=username_or_fallback.clone()
                    initial_tab=popup_initial_tab.get_untracked()
                />
            </Show>
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
