pub mod edit;
mod ic;
pub mod overlay;
mod posts;
mod profile_iter;
pub mod profile_post;

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
use serde::{Deserialize, Serialize};
use state::{app_state::AppState, canisters::auth_state};

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
use yral_metadata_client::MetadataClient;
use yral_username_gen::random_username_from_principal;

#[cfg(feature = "ssr")]
use leptos::prelude::ServerFnError;

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

// Smiley Game Stats data structure
#[derive(Clone, Debug, Deserialize, Serialize)]
struct SmileyGameStats {
    principal_id: String,
    wins: u64,
    losses: u64,
    total_games: u64,
}

// Unlock Profile component for non-logged-in users
#[component]
fn UnlockProfile() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-[30px] items-center w-full py-12">
            <div class="flex flex-col gap-[10px] items-center text-center">
                <p class="font-semibold text-base text-neutral-50">
                    "Unlock Profile"
                </p>
                <div class="font-normal text-sm text-neutral-400">
                    <p class="mb-0">"Log in to claim rewards, build your profile,"</p>
                    <p>"and join the YRAL community."</p>
                </div>
            </div>
            <div class="w-full max-w-[270px]">
                <ConnectLogin
                    cta_location="profile_unlock"
                    redirect_to=format!("/profile/posts")
                />
            </div>
        </div>
    }
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
            .get_followers(
                self.user_principal,
                self.cursor.get_untracked(),
                limit,
                Some(true),
            )
            .await
            .map_err(|e| FollowerProviderError(format!("Failed to fetch followers: {e}")))?;

        let response = match result {
            Result1::Ok(resp) => resp,
            Result1::Err(e) => return Err(FollowerProviderError(format!("API error: {e}"))),
        };

        // Update cursor for next fetch
        if let Some(next_cursor) = response.next_cursor {
            self.cursor.set(Some(next_cursor));
        }

        // Bulk fetch metadata for all principals
        let principals: Vec<Principal> = response
            .followers
            .iter()
            .map(|item| item.principal_id)
            .collect();

        let metadata_client = MetadataClient::default();
        let metadata_map = metadata_client
            .get_user_metadata_bulk(principals)
            .await
            .unwrap_or_default();

        // Convert FollowerItem to FollowerData with actual usernames
        let data = response
            .followers
            .into_iter()
            .map(|item| {
                let metadata = metadata_map
                    .get(&item.principal_id)
                    .and_then(|m| m.as_ref());

                // Use actual username if available and not empty, otherwise generate one
                let username = metadata
                    .and_then(|m| {
                        if !m.user_name.trim().is_empty() {
                            Some(m.user_name.clone())
                        } else {
                            None
                        }
                    })
                    .or_else(|| Some(random_username_from_principal(item.principal_id, 15)));

                // Use profile pic from API if available, otherwise generate one
                let profile_pic = item
                    .profile_picture_url
                    .or_else(|| Some(propic_from_principal(item.principal_id)));

                FollowerData {
                    principal_id: item.principal_id,
                    username,
                    profile_pic,
                    caller_follows: item.caller_follows,
                }
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
            .get_following(
                self.user_principal,
                self.cursor.get_untracked(),
                limit,
                Some(true),
            )
            .await
            .map_err(|e| FollowerProviderError(format!("Failed to fetch following: {e}")))?;

        let response = match result {
            Result2::Ok(resp) => resp,
            Result2::Err(e) => return Err(FollowerProviderError(format!("API error: {e}"))),
        };

        // Update cursor for next fetch
        if let Some(next_cursor) = response.next_cursor {
            self.cursor.set(Some(next_cursor));
        }

        // Bulk fetch metadata for all principals
        let principals: Vec<Principal> = response
            .following
            .iter()
            .map(|item| item.principal_id)
            .collect();

        let metadata_client = MetadataClient::default();
        let metadata_map = metadata_client
            .get_user_metadata_bulk(principals)
            .await
            .unwrap_or_default();

        // Convert FollowingItem to FollowerData with actual usernames
        let data = response
            .following
            .into_iter()
            .map(|item| {
                let metadata = metadata_map
                    .get(&item.principal_id)
                    .and_then(|m| m.as_ref());

                // Use actual username if available and not empty, otherwise generate one
                let username = metadata
                    .and_then(|m| {
                        if !m.user_name.trim().is_empty() {
                            Some(m.user_name.clone())
                        } else {
                            None
                        }
                    })
                    .or_else(|| Some(random_username_from_principal(item.principal_id, 15)));

                // Use profile pic from API if available, otherwise generate one
                let profile_pic = item
                    .profile_picture_url
                    .or_else(|| Some(propic_from_principal(item.principal_id)));

                FollowerData {
                    principal_id: item.principal_id,
                    username,
                    profile_pic,
                    caller_follows: item.caller_follows,
                }
            })
            .collect();

        Ok(PageEntry {
            data,
            end: response.next_cursor.is_none(),
        })
    }
}

async fn follow_user_via_agent(
    target_principal: Principal,
    delegated_identity_wire: yral_types::delegated_identity::DelegatedIdentityWire,
    follower_username: Option<String>,
) -> Result<(), String> {
    use consts::OFF_CHAIN_AGENT_URL;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct FollowRequest {
        delegated_identity_wire: yral_types::delegated_identity::DelegatedIdentityWire,
        target_principal: Principal,
        follower_username: Option<String>,
    }

    #[derive(Deserialize)]
    struct FollowResponse {
        success: bool,
    }

    let url = OFF_CHAIN_AGENT_URL
        .join("api/v1/user/follow")
        .map_err(|e| format!("Failed to construct URL: {e}"))?;

    let request_body = FollowRequest {
        delegated_identity_wire,
        target_principal,
        follower_username,
    };

    #[cfg(feature = "ssr")]
    {
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Follow failed: {error_text}"));
        }

        let follow_response: FollowResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        if follow_response.success {
            Ok(())
        } else {
            Err("Follow operation failed".to_string())
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        use gloo::net::http::Request;

        let response = Request::post(url.as_str())
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {e}"))?
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Follow failed: {error_text}"));
        }

        let follow_response: FollowResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        if follow_response.success {
            Ok(())
        } else {
            Err("Follow operation failed".to_string())
        }
    }
}

#[component]
fn FollowAndAuthCanLoader(
    user_principal: Principal,
    caller_follows_user: Option<bool>,
    user_follows_caller: Option<bool>,
    #[prop(optional, into)] on_follow_success: Option<Callback<()>>,
    #[prop(optional, into)] on_unfollow_success: Option<Callback<()>>,
) -> impl IntoView {
    let auth = auth_state();
    let follows = RwSignal::new(caller_follows_user);
    let (is_loading, set_is_loading) = signal(false);

    // Client-side action for follow/unfollow
    let follow_toggle = Action::new(move |&()| {
        let target = user_principal;
        send_wrap(async move {
            set_is_loading.set(true);

            let Ok(canisters) = auth.auth_cans().await else {
                log::warn!("Trying to toggle follow without auth");
                set_is_loading.set(false);
                return;
            };

            let should_follow = {
                let mut follows_w = follows.write();
                let current = follows_w.unwrap_or_default();
                *follows_w = Some(!current);
                !current
            };

            let result = if should_follow {
                // Call off-chain API for follow (with notification)
                match auth.user_identity.await {
                    Ok(identity) => {
                        let follower_username = canisters.profile_details().username.clone();
                        follow_user_via_agent(target, identity.id_wire, follower_username).await
                    }
                    Err(e) => Err(format!("Failed to get identity: {e}")),
                }
            } else {
                // Keep unfollow as direct canister call for now
                let service = canisters.user_info_service().await;
                match service.unfollow_user(target).await {
                    Ok(yral_canisters_client::user_info_service::Result_::Ok) => Ok(()),
                    Ok(yral_canisters_client::user_info_service::Result_::Err(e)) => Err(e),
                    Err(e) => Err(format!("{e:?}")),
                }
            };

            match result {
                Ok(()) => {
                    // Success - keep the optimistic update and call callback
                    if should_follow {
                        if let Some(ref cb) = on_follow_success {
                            cb.run(());
                        }
                    } else if let Some(ref cb) = on_unfollow_success {
                        cb.run(());
                    }
                }
                Err(e) => {
                    log::warn!("Error toggling follow status: {e}");
                    // Rollback on error
                    follows.update(|f| _ = f.as_mut().map(|f| *f = !*f));
                }
            }

            set_is_loading.set(false);
        })
    });

    // Resource for fetching follow status (if needed for refresh)
    let follow_fetch = auth.derive_resource(
        || (),
        move |_cans: Canisters<true>, _| async move {
            // If we had initial data, use it. Otherwise we would fetch it here
            // For now, just return the initial value
            Ok::<_, ServerFnError>(caller_follows_user.unwrap_or(false))
        },
    );

    view! {
        <FollowButton
            is_following=follows
            is_loading=is_loading
            user_follows_caller=user_follows_caller
            on_click=move || { follow_toggle.dispatch(()); }
        />
        <Suspense>
            {move || Suspend::new(async move {
                match follow_fetch.await {
                    Ok(res) => {
                        follows.set(Some(res));
                    }
                    Err(e) => {
                        log::warn!("failed to fetch follow status {e}");
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn FollowButton(
    is_following: RwSignal<Option<bool>>,
    is_loading: ReadSignal<bool>,
    user_follows_caller: Option<bool>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let button_text = move || {
        if is_loading.get() {
            "Loading..."
        } else if is_following.get().unwrap_or(false) {
            "Unfollow"
        } else if user_follows_caller.unwrap_or(false) {
            "Follow Back"
        } else {
            "Follow"
        }
    };

    let button_class = move || {
        if is_following.get().unwrap_or(false) {
            "w-full bg-transparent border border-neutral-700 rounded-lg px-5 py-2.5 flex items-center justify-center"
        } else if user_follows_caller.unwrap_or(false) {
            "w-full bg-primary-600 border border-primary-600 rounded-lg px-5 py-2.5 flex items-center justify-center"
        } else {
            "w-full bg-[#212121] border border-neutral-700 rounded-lg px-5 py-2.5 flex items-center justify-center"
        }
    };

    view! {
        <button
            on:click=move |_| on_click()
            class=button_class
            disabled=is_loading
        >
            <span class="font-semibold text-sm text-neutral-50">
                {button_text}
            </span>
        </button>
    }
}

#[server]
async fn fetch_smiley_game_stats(
    principal_id: Principal,
) -> Result<SmileyGameStats, ServerFnError> {
    use consts::SMILEY_GAME_STATS_URL;
    use reqwest::Client;

    let client = Client::new();
    let url = format!(
        "{}/get_smiley_game_stats?principal_id={}",
        SMILEY_GAME_STATS_URL.as_str(),
        principal_id.to_text()
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to fetch game stats: {e}")))?;

    if response.status().is_success() {
        let stats = response
            .json::<SmileyGameStats>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to parse game stats: {e}")))?;
        Ok(stats)
    } else {
        // Return default stats if API fails
        Ok(SmileyGameStats {
            principal_id: principal_id.to_text(),
            wins: 0,
            losses: 0,
            total_games: 0,
        })
    }
}

// Component for displaying a user in the followers/following list
#[component]
fn UserListItem(
    data: FollowerData,
    node_ref: Option<NodeRef<html::Div>>,
    #[prop(default = false)] is_following_tab: bool,
    #[prop(default = false)] is_own_profile: bool,
    #[prop(default = true)] viewer_is_connected: bool,
    #[prop(optional)] show_popup: Option<RwSignal<bool>>,
) -> impl IntoView {
    let navigate = use_navigate();
    let auth = auth_state();

    // Get current user's principal
    let current_user_principal = auth
        .user_principal
        .get_untracked()
        .and_then(|res| res.ok())
        .unwrap_or(Principal::anonymous());

    // Don't show follow button for own profile OR if viewer is not logged in
    let show_follow_button = data.principal_id != current_user_principal && viewer_is_connected;

    // In the following tab of your own profile, you're already following everyone
    let caller_follows_override = if is_following_tab && is_own_profile {
        Some(true)
    } else {
        Some(data.caller_follows)
    };

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
                    if let Some(show) = show_popup {
                        show.set(false);
                    }
                    navigate(&format!("/profile/{principal_for_nav}/posts"), Default::default());
                }
            >
                // Avatar
                <img
                    class="w-8 h-8 rounded-full object-cover"
                    src=data.profile_pic.clone().unwrap_or_default()
                    alt=data.username.clone().unwrap_or_default()
                />

                // Username (always available now - either real or generated)
                <span class="font-semibold text-sm text-neutral-50">
                    {data.username.clone().unwrap_or_else(|| random_username_from_principal(data.principal_id, 15))}
                </span>
            </div>

            // Follow button
            <Show when=move || show_follow_button>
                <div class="shrink-0">
                    <FollowAndAuthCanLoader
                        user_principal=data.principal_id
                        caller_follows_user=caller_follows_override
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
                        if let Some(show) = show_popup {
                            show.set(false);
                        }
                        navigate(&format!("/profile/{principal_for_nav}/posts"), Default::default());
                    }
                >
                    // Avatar
                    <img
                        class="w-8 h-8 rounded-full object-cover"
                        src=data.profile_pic.clone().unwrap_or_default()
                        alt=data.username.clone().unwrap_or_default()
                    />

                    // Username (always available now - either real or generated)
                    <span class="font-semibold text-sm text-neutral-50">
                        {data.username.clone().unwrap_or_else(|| random_username_from_principal(data.principal_id, 15))}
                    </span>
                </div>

                // Follow button
                <Show when=move || show_follow_button>
                    <div class="shrink-0">
                        <FollowAndAuthCanLoader
                            user_principal=data.principal_id
                            caller_follows_user=caller_follows_override
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
    viewer_is_connected: bool,
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(initial_tab);

    // Check if this is the current user's profile
    let auth = auth_state();
    let is_own_profile = auth
        .user_principal
        .get_untracked()
        .and_then(|res| res.ok())
        .map(|p| p == user_principal)
        .unwrap_or(false);

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
                                        view! {
                                            <UserListItem
                                                data=item
                                                node_ref=node_ref
                                                is_following_tab=true
                                                is_own_profile=is_own_profile
                                                viewer_is_connected=viewer_is_connected
                                                show_popup=show
                                            />
                                        }
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
                                view! {
                                    <UserListItem
                                        data=item
                                        node_ref=node_ref
                                        is_following_tab=false
                                        is_own_profile=is_own_profile
                                        viewer_is_connected=viewer_is_connected
                                        show_popup=show
                                    />
                                }
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
    is_own_profile: RwSignal<bool>,
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
    let is_connected = auth.is_logged_in_with_oauth();

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
        _ => 0,
    });

    view! {
        <div class="flex flex-col gap-y-12 justify-center pb-12 w-11/12 sm:w-6/12">
            <Show when=move || current_tab() == 0>
                // Show UnlockProfile for non-logged-in users viewing their own profile
                {
                    let username = username.clone();
                    move || {
                        if is_own_profile.get() && !is_connected() {
                            view! { <UnlockProfile /> }.into_any()
                        } else {
                            view! { <ProfilePosts user_canister user_principal username=username.clone()/> }.into_any()
                        }
                    }
                }
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

    // Make counts reactive for dynamic updates
    let followers_count = RwSignal::new(user.followers_cnt);
    let following_count = RwSignal::new(user.following_cnt);
    let games_played_count = RwSignal::new(0u64);
    let bio = user.bio.clone().unwrap_or_default();
    let website_url = user.website_url.clone().unwrap_or_default();

    // Create a resource to fetch game stats
    let _game_stats_resource = LocalResource::new(move || async move {
        match fetch_smiley_game_stats(user_principal).await {
            Ok(stats) => {
                games_played_count.set(stats.total_games);
                stats
            }
            Err(e) => {
                log::warn!("Failed to fetch game stats: {e}");
                // Return default stats on error
                SmileyGameStats {
                    principal_id: user_principal.to_text(),
                    wins: 0,
                    losses: 0,
                    total_games: 0,
                }
            }
        }
    });

    view! {
        <div class="overflow-y-auto pb-12 min-h-screen text-white bg-black">
            <Show when=move || notification_panel.get()>
                <NotificationPage close=notification_panel />
            </Show>
            // Header with title and navigation icons - aligned with content width
            <div class="flex justify-center w-full bg-black pt-4">
                <div class="flex h-12 items-center justify-between px-4 sm:px-0 py-3 w-11/12 sm:w-6/12">
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
                <div class="flex flex-col gap-5 w-11/12 sm:w-6/12">
                    // Profile header with avatar and stats
                    <div class="flex justify-between items-start">
                        // Avatar
                        <img
                            class=move || {
                                if is_own_profile.get() {
                                    "w-[60px] h-[60px] rounded-full shrink-0 ring-2 ring-pink-500"
                                } else {
                                    "w-[60px] h-[60px] rounded-full shrink-0"
                                }
                            }
                            alt=username_or_fallback.clone()
                            src=profile_pic
                        />

                        // Stats section - moved to the right
                        <div class="flex gap-0">
                            // Followers
                            <button
                                class="flex flex-col gap-1 items-center text-center w-[85px] cursor-pointer hover:opacity-80 transition-opacity"
                                on:click=move |_| {
                                    popup_initial_tab.set(0);
                                    show_followers_popup.set(true);
                                }
                            >
                                <span class="font-semibold text-base text-neutral-50">
                                    {move || followers_count.get()}
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
                                    {move || following_count.get()}
                                </span>
                                <span class="font-normal text-sm text-neutral-50">
                                    "Following"
                                </span>
                            </button>

                            // Games Played
                            <div class="flex flex-col gap-1 items-center text-center w-[85px]">
                                <span class="font-semibold text-base text-neutral-50">
                                    {move || games_played_count.get()}
                                </span>
                                <span class="font-normal text-sm text-neutral-50">
                                    "Games"
                                </span>
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
                                        {(!website_url.is_empty()).then(|| {
                                            let formatted_url = if !website_url.starts_with("http://") && !website_url.starts_with("https://") {
                                                format!("https://{website_url}")
                                            } else {
                                                website_url.clone()
                                            };
                                            view! {
                                                <a
                                                    href=formatted_url
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    class="text-[#3d8eff] hover:underline"
                                                >
                                                    {website_url.clone()}
                                                </a>
                                            }
                                        })}
                                    </p>
                                })}
                            </div>
                        </div>

                        // Social Links button - commented out for now
                        // <div class="flex gap-1 items-center justify-center bg-[#212121] rounded-full px-2.5 py-1.5 self-start">
                        //     <Icon icon=icondata::FiPlus attr:class="text-base text-neutral-300" />
                        //     <span class="font-semibold text-xs text-neutral-50 whitespace-nowrap">
                        //         "Social Links"
                        //     </span>
                        // </div>
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
                                            // Show Edit Profile button for own profile ONLY if logged in with OAuth
                                            <Show when=move || user_principal == authenticated_princ && is_connected()>
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
                                        // Show Follow button for other profiles only if viewer is logged in
                                        <Show when=move || {
                                            user_principal != authenticated_princ && is_connected()
                                        }>
                                            <FollowAndAuthCanLoader
                                                user_principal=user_principal
                                                caller_follows_user=user.caller_follows_user
                                                user_follows_caller=user.user_follows_caller
                                                on_follow_success=Callback::new(move |_| {
                                                    followers_count.update(|c| *c += 1);
                                                })
                                                on_unfollow_success=Callback::new(move |_| {
                                                    followers_count.update(|c| *c = c.saturating_sub(1));
                                                })
                                            />
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
                <ListSwitcher1 user_canister user_principal username=username_or_fallback.clone() is_own_profile/>
            </div>

            // Followers/Following popup
            <Show when=move || show_followers_popup.get()>
                <FollowersFollowingPopup
                    show=show_followers_popup
                    user_principal=user_principal
                    username=username_or_fallback.clone()
                    initial_tab=popup_initial_tab.get_untracked()
                    viewer_is_connected=is_connected()
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

    // Use derive_resource which will handle auth state properly
    let user_details = auth.derive_resource(
        param_id,
        move |auth_cans: Canisters<true>, profile_id| async move {
            let profile_id = profile_id.ok_or_else(|| ServerFnError::new("Invalid ID"))?;

            // First check if this is the logged-in user's own profile
            if match &profile_id {
                UsernameOrPrincipal::Principal(princ) => *princ == auth_cans.user_principal(),
                UsernameOrPrincipal::Username(u) => {
                    Some(u) == auth_cans.profile_details().username.as_ref()
                }
            } {
                leptos::logging::log!("User canister found for profile ID: {}", profile_id);
                return Ok::<_, ServerFnError>(auth_cans.profile_details());
            }

            leptos::logging::log!("Fetching other user's profile with auth");

            // Use authenticated canisters to get correct follow status
            let user_details = auth_cans
                .get_profile_details(profile_id.to_string())
                .await?;

            leptos::logging::log!(
                "User details fetched with auth: {:?} with principal {:?}",
                user_details.clone(),
                user_details
                    .clone()
                    .map(|u| u.principal.to_text())
                    .unwrap_or_else(|| "None".to_string())
            );

            user_details.ok_or_else(|| ServerFnError::new("User not found"))
        },
    );

    view! {
        <ProfilePageTitle />
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                match user_details.await {
                    Ok(user) => view! {
                        <ProfileComponent user />
                    }.into_any(),
                    Err(e) => {
                        leptos::logging::log!("Error loading profile: {}", e);
                        view! {
                            <Redirect path="/" />
                        }.into_any()
                    }
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
