use candid::Principal;
use chrono::{DateTime, NaiveDate, Utc};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use consts::AUTH_JOURNEY_PAGE;
use consts::{AUTH_JOURNET, CUSTOM_DEVICE_ID, DEVICE_ID, NSFW_ENABLED_COOKIE};
use global_constants::REFERRAL_REWARD_SATS;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::storage::use_local_storage;
use leptos_use::use_timeout_fn;
use leptos_use::{use_cookie, use_cookie_with_options, UseCookieOptions, UseTimeoutFnReturn};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use yral_canisters_common::utils::vote::VoteKind;
use yral_canisters_common::Canisters;
use yral_metadata_client::MetadataClient;

use crate::event_streaming::events::EventCtx;
use crate::event_streaming::events::HistoryCtx;
use crate::mixpanel::state::MixpanelState;

#[server]
async fn track_event_server_fn(props: Value) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use axum_extra::headers::UserAgent;
    use axum_extra::TypedHeader;
    use leptos_axum::extract;

    let mut props = props;

    // Attempt to extract headers and User-Agent
    let result: Result<(HeaderMap, TypedHeader<UserAgent>), _> = extract().await;

    let (ip, ua) = match result {
        Ok((headers, TypedHeader(user_agent))) => {
            let ip = headers
                .get("x-forwarded-for")
                .and_then(|val| val.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let ua = user_agent.as_str().to_string();
            (Some(ip), Some(ua))
        }
        Err(_) => (None, None),
    };

    // Inject metadata into props
    props["ip"] = ip.clone().into();
    props["ip_addr"] = ip.clone().into();
    props["user_agent"] = ua.clone().into();

    // check if user_type is present or not, if not get principal and fetch from metadata client
    if props.get("user_type").is_none() {
        let principal = props
            .get("principal")
            .and_then(Value::as_str)
            .and_then(|f| Principal::from_text(f).ok());
        let is_logged_in = props
            .get("is_logged_in")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if let Some(user_principal) = principal {
            let metadata_client: MetadataClient<false> = MetadataClient::default();
            let metadata = metadata_client
                .set_signup_datetime(user_principal, is_logged_in)
                .await;
            if let Ok(metadata) = metadata {
                if let Some(signup_at) = metadata.signup_at {
                    if let Some(signup_date) =
                        DateTime::<Utc>::from_timestamp(signup_at, 0).map(|dt| dt.date_naive())
                    {
                        let today_date: NaiveDate = Utc::now().date_naive();

                        props["user_type"] = if today_date > signup_date {
                            "repeat".into()
                        } else {
                            "new".into()
                        };
                    }
                }

                if let Some(email) = metadata.email {
                    props["email"] = email.into();
                }
            }
        }
    }

    #[cfg(feature = "qstash")]
    {
        let qstash_client = use_context::<crate::qstash::QStashClient>();
        if let Some(qstash_client) = qstash_client {
            let token =
                std::env::var("ANALYTICS_SERVER_TOKEN").expect("ANALYTICS_SERVER_TOKEN is not set");
            qstash_client
                .send_analytics_event_to_qstash(props, token)
                .await
                .map_err(|e| ServerFnError::new(format!("Mixpanel track error: {e:?}")))?;
        } else {
            logging::error!("QStash client not found. Gracefully continuing");
        }
    }
    Ok(())
}

pub fn parse_query_params_utm() -> Result<Vec<(String, String)>, String> {
    if let Some(storage) = window()
        .local_storage()
        .map_err(|e| format!("Failed to access localstorage: {e:?}"))?
    {
        if let Some(url_str) = storage
            .get_item("initial_url")
            .map_err(|e| format!("Failed to get utm from localstorage: {e:?}"))?
        {
            let url =
                reqwest::Url::parse(&url_str).map_err(|e| format!("Failed to parse url: {e:?}"))?;
            storage
                .remove_item("initial_url")
                .map_err(|e| format!("Failed to remove initial_url from localstorage: {e:?}"))?;
            return Ok(url
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect());
        }
    }
    Ok(Vec::new())
}

pub(super) fn send_event_to_server<T>(event_name: &str, props: T)
where
    T: Serialize,
{
    let payload = get_event_payload(event_name, props);
    spawn_local(async {
        let res = track_event_server_fn(payload).await;
        match res {
            Ok(_) => {}
            Err(e) => logging::error!("Error tracking Mixpanel event: {}", e),
        }
    });
}

pub(super) async fn send_event_to_server_async<T>(event_name: &str, props: T)
where
    T: Serialize,
{
    let payload = get_event_payload(event_name, props);
    let res = track_event_server_fn(payload).await;
    match res {
        Ok(_) => {}
        Err(e) => logging::error!("Error tracking Mixpanel event: {}", e),
    }
}

fn get_event_payload<T>(event_name: &str, props: T) -> Value
where
    T: Serialize,
{
    let mut props = serde_json::to_value(&props).unwrap();
    props["event"] = event_name.into();
    props["time"] = chrono::Utc::now().timestamp().into();
    props["$device_id"] = MixpanelGlobalProps::get_device_id().into();
    props["custom_device_id"] = MixpanelGlobalProps::get_custom_device_id().into();
    let user_id = props.get("user_id").and_then(Value::as_str);
    props["principal"] = if user_id.is_some() {
        user_id.into()
    } else {
        props.get("visitor_id").and_then(Value::as_str).into()
    };
    let current_url = window().location().href().ok();
    let origin = window()
        .location()
        .origin()
        .ok()
        .unwrap_or_else(|| "unknown".to_string());
    if let Some(url) = current_url {
        if props["event"] == "home_page_viewed" {
            props["current_url"] = origin.clone().into();
            props["$current_url"] = origin.into();
        } else {
            props["current_url"] = url.clone().into();
            props["$current_url"] = url.into();
        }
    }
    let history = use_context::<HistoryCtx>();
    if let Some(history) = history {
        if history.utm.get_untracked().is_empty() {
            if let Ok(utms) = parse_query_params_utm() {
                history.push_utm(utms);
            }
        }
        for (key, value) in history.utm.get_untracked() {
            props[key] = value.into();
        }
    } else {
        logging::error!("HistoryCtx not found. Gracefully continuing");
    }
    props
}

/// Global properties for Mixpanel events
#[derive(Clone, Serialize)]
pub struct MixpanelGlobalProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub username: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
}

impl MixpanelGlobalProps {
    pub fn new(
        user_principal: Principal,
        canister_id: Principal,
        is_logged_in: bool,
        is_nsfw_enabled: bool,
        username: Option<String>,
    ) -> Self {
        Self {
            user_id: if is_logged_in {
                Some(user_principal.to_text().clone())
            } else {
                None
            },
            visitor_id: if !is_logged_in {
                Some(user_principal.to_text())
            } else {
                None
            },
            is_logged_in,
            canister_id: canister_id.to_text(),
            is_nsfw_enabled,
            username,
        }
    }

    /// Load global state (login, principal, NSFW toggle)
    pub fn try_get(cans: &Canisters<true>, is_logged_in: bool) -> Self {
        let (is_nsfw_enabled, _) = use_cookie_with_options::<bool, FromToStringCodec>(
            NSFW_ENABLED_COOKIE,
            UseCookieOptions::default()
                .path("/")
                .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
                .same_site(leptos_use::SameSite::Lax),
        );
        let is_nsfw_enabled = is_nsfw_enabled.get_untracked().unwrap_or(false);

        Self {
            user_id: if is_logged_in {
                Some(cans.user_principal().to_text())
            } else {
                None
            },
            visitor_id: if !is_logged_in {
                Some(cans.user_principal().to_text())
            } else {
                None
            },
            is_logged_in,
            canister_id: cans.user_canister().to_text(),
            is_nsfw_enabled,
            username: cans.profile_details().username,
        }
    }

    pub fn get_device_id() -> String {
        let device_id = MixpanelState::get_device_id();
        if let Some(device_id) = device_id.get_untracked() {
            device_id
        } else {
            let device_id_val = crate::local_storage::LocalStorage::uuid_get_or_init(DEVICE_ID);
            device_id.set(Some(device_id_val.clone()));
            device_id_val
        }
    }

    pub fn get_custom_device_id() -> String {
        let custom_device_id = MixpanelState::get_custom_device_id();
        if let Some(custom_device_id) = custom_device_id.get_untracked() {
            custom_device_id
        } else {
            let custom_device_id_val =
                crate::local_storage::LocalStorage::uuid_get_or_init(CUSTOM_DEVICE_ID);
            custom_device_id.set(Some(custom_device_id_val.clone()));
            custom_device_id_val
        }
    }

    pub fn get_auth_journey() -> String {
        let (auth_journey, _, _) = use_local_storage::<String, FromToStringCodec>(AUTH_JOURNET);
        // Extracting the device ID value
        let auth_journey_value = auth_journey.get_untracked();
        if auth_journey_value.is_empty() {
            "unknown".to_string()
        } else {
            auth_journey_value
        }
    }
    pub fn set_auth_journey(auth_journey: String) {
        let (_, set_auth_journey, _) = use_local_storage::<String, FromToStringCodec>(AUTH_JOURNET);
        set_auth_journey.set(auth_journey);
    }

    pub fn from_ev_ctx(ev_ctx: EventCtx) -> Option<Self> {
        #[cfg(not(feature = "hydrate"))]
        {
            return None;
        }
        #[cfg(feature = "hydrate")]
        {
            let (is_nsfw_enabled, _) = use_cookie_with_options::<bool, FromToStringCodec>(
                NSFW_ENABLED_COOKIE,
                UseCookieOptions::default()
                    .path("/")
                    .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
                    .same_site(leptos_use::SameSite::Lax),
            );
            let is_nsfw_enabled = is_nsfw_enabled.get_untracked().unwrap_or(false);

            Self::from_ev_ctx_with_nsfw_info(ev_ctx, is_nsfw_enabled)
        }
    }

    pub fn from_ev_ctx_with_nsfw_info(ev_ctx: EventCtx, is_nsfw_enabled: bool) -> Option<Self> {
        #[cfg(not(feature = "hydrate"))]
        {
            return None;
        }
        #[cfg(feature = "hydrate")]
        {
            let user = ev_ctx.user_details()?;
            let is_logged_in = ev_ctx.is_connected();

            Some(Self {
                user_id: is_logged_in.then(|| user.details.principal()),
                visitor_id: (!is_logged_in).then(|| user.details.principal()),
                is_logged_in,
                canister_id: user.canister_id.to_text(),
                is_nsfw_enabled,
                username: user.details.username,
            })
        }
    }

    pub fn try_get_with_nsfw_info(
        cans: &Canisters<true>,
        is_logged_in: bool,
        is_nsfw_enabled: bool,
    ) -> Self {
        Self {
            user_id: if is_logged_in {
                Some(cans.user_principal().to_text())
            } else {
                None
            },
            visitor_id: if !is_logged_in {
                Some(cans.user_principal().to_text())
            } else {
                None
            },
            is_logged_in,
            canister_id: cans.user_canister().to_text(),
            is_nsfw_enabled,
            username: cans.profile_details().username,
        }
    }

    pub fn page_name(&self) -> BottomNavigationCategory {
        #[cfg(feature = "hydrate")]
        {
            let path = window().location().pathname().unwrap_or_default();
            path.try_into().unwrap_or(BottomNavigationCategory::Profile)
        }
        #[cfg(not(feature = "hydrate"))]
        {
            log::error!("calling MixpanelGlobalProps::page_name from SSR is not sane");
            BottomNavigationCategory::Profile
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MixpanelOnboardingPopupType {
    SatsCreditPopup,
}

use std::convert::TryFrom;

impl TryFrom<String> for BottomNavigationCategory {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains("/profile/") {
            return Ok(BottomNavigationCategory::Profile);
        } else if value.contains("/wallet/") {
            return Ok(BottomNavigationCategory::Wallet);
        } else if value.contains("/hot-or-not/") {
            return Ok(BottomNavigationCategory::Home);
        }

        match value.as_str() {
            "/" => Ok(BottomNavigationCategory::Home),
            "/wallet" => Ok(BottomNavigationCategory::Wallet),
            "/upload" => Ok(BottomNavigationCategory::UploadVideo),
            "/profile" => Ok(BottomNavigationCategory::Profile),
            "/menu" => Ok(BottomNavigationCategory::Menu),
            _ => Err(()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MixpanelPostGameType {
    HotOrNot,
}

impl From<VoteKind> for ChosenGameOption {
    fn from(value: VoteKind) -> Self {
        match value {
            VoteKind::Hot => Self::Hot,
            VoteKind::Not => Self::Not,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChosenGameOption {
    Hot,
    Not,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MixpanelVideoClickedCTAType {
    Like,
    Share,
    ReferAndEarn,
    Report,
    NsfwToggle,
    Mute,
    Unmute,
    CreatorProfile,
    VideoPlay,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MixpanelMenuClickedCTAType {
    TalkToTheTeam,
    TermsOfService,
    PrivacyPolicy,
    LogOut,
    FollowOn,
    ReferAndEarn,
    Leaderboard,
    Settings,
    AboutUs,
    ViewProfile,
    Follow,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MixpanelProfileClickedCTAType {
    Videos,
    GamesPlayed,
    MemeCoin,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum GameConclusion {
    Pending,
    Win,
    Loss,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum StakeType {
    Sats,
    Cents,
    DolrAi,
    Btc,
    Usdc,
    Yral,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BottomNavigationCategory {
    UploadVideo,
    Profile,
    #[default]
    Menu,
    Home,
    Wallet,
}
pub struct MixPanelEvent;

macro_rules! derive_event {
    ($name:ident = $ev:expr => { $($prop:ident: $typ:ty),* }) => {
        #[allow(non_camel_case_types)]
        #[derive(serde::Serialize)]
        struct $name {
            user_id: Option<String>,
            visitor_id: Option<String>,
            username: Option<String>,
            is_logged_in: bool,
            canister_id: String,
            is_nsfw_enabled: bool,
            $($prop: $typ),*
        }

        impl $name {
            #[allow(clippy::too_many_arguments)]
            pub fn new(
                global: MixpanelGlobalProps,
                $($prop: $typ),*
            ) -> Self {
                let MixpanelGlobalProps {
                    user_id,
                    visitor_id,
                    username,
                    is_logged_in,
                    canister_id,
                    is_nsfw_enabled,
                } = global;
                Self {
                    user_id,
                    visitor_id,
                    username,
                    is_logged_in,
                    canister_id,
                    is_nsfw_enabled,
                    $($prop),*
                }
            }
        }
        // static assert to ensure $name begins with track_
        const _: () = {
            assert!(matches!(stringify!($name).as_bytes().split_at(6), (b"track_", _)));
        };

        impl MixPanelEvent {
            #[allow(clippy::too_many_arguments)]
            pub fn $name(
                global: MixpanelGlobalProps,
                $($prop: $typ),*
            ) {
                let event = $name::new(global, $($prop),*);
                send_event_to_server($ev, event);
            }
        }
    };
    ($name:ident { $($prop:ident: $typ:ty),* }) => {
        derive_event!(
            $name = &stringify!($name)[6..] => { $($prop: $typ),* }
        );
    }
}

derive_event!(track_home_page_viewed {});

derive_event!(track_refer_and_earn_page_viewed {
    referral_bonus: u64
});

derive_event!(track_menu_page_viewed {});

derive_event!(track_upload_page_viewed {});

derive_event!(track_edit_profile_clicked { page_name: String });

derive_event!(track_unlock_higher_bets_popup_shown {
    page_name: String,
    stake_amount: u64,
    stake_type: StakeType
});

derive_event!(track_edit_username_clicked {});

derive_event!(track_wallet_page_viewed {});

derive_event!(track_menu_clicked {
    cta_type: MixpanelMenuClickedCTAType
});

derive_event!(track_profile_tab_clicked {
    is_own_profile: bool,
    publisher_user_id: String,
    cta_type: MixpanelProfileClickedCTAType
});

derive_event!(track_delete_account_clicked { page_name: String });

derive_event!(track_delete_account_confirmed { page_name: String });

derive_event!(track_account_deleted { page_name: String });

derive_event!(track_profile_page_viewed {
    is_own_profile: bool,
    publisher_user_id: String
});

derive_event!(track_withdraw_tokens_clicked {
    token_clicked: StakeType
});

derive_event!(track_claim_airdrop_clicked {
    token_type: StakeType,
    page_name: String
});

derive_event!(track_airdrop_claimed {
    token_type: StakeType,
    is_success: bool,
    claimed_amount: u64,
    page_name: String
});

derive_event!(track_referral_link_copied {
    referral_bonus: u64
});

derive_event!(track_refer_friend_clicked {
    is_airdrop_eligible: bool,
    cta_type: String,
    page_name: String
});

derive_event!(track_share_invites_clicked {
    referral_bonus: u64
});

derive_event!(track_video_upload_error_shown { error: String });

derive_event!(track_onboarding_popup_shown {
    credited_amount: u64,
    popup_type: MixpanelOnboardingPopupType
});

derive_event!(track_select_file_clicked {});

derive_event!(track_file_selection_success { file_type: String });

derive_event!(track_video_upload_initiated {
    caption_added: bool,
    hashtags_added: bool,
    upload_type: Option<String>,
    token_type: String
});

derive_event!(track_bottom_navigation_clicked {
    category_name: BottomNavigationCategory
});

derive_event!(track_enable_notifications { toggle: bool });

derive_event!(track_signup_clicked {
    page_name: BottomNavigationCategory
});

derive_event!(track_auth_screen_viewed {
    page_name: BottomNavigationCategory
});

derive_event!(track_auth_initiated = "signup_journey_selected" => {
    auth_journey: String,
    page_name: BottomNavigationCategory
});

derive_event!(track_signup_success {
    is_referral: bool,
    referrer_user_id: Option<String>,
    auth_journey: String,
    page_name: BottomNavigationCategory
});

derive_event!(track_login_success {
    auth_journey: String,
    page_name: BottomNavigationCategory
});

derive_event!(track_sats_to_btc_converted {
    sats_converted: f64,
    conversion_ratio: f64
});

derive_event!(track_enable_nsfw_popup_shown { page_name: String });

derive_event!(track_low_on_sats_popup_shown {
    is_airdrop_eligible: bool,
    page_name: String
});

derive_event!(track_nsfw_enabled {
    publisher_user_id: String,
    video_id: String,
    is_nsfw: bool,
    page_name: String,
    cta_type: Option<String>
});

derive_event!(track_nsfw_false = "NSFW_false" => {
    publisher_user_id: String,
    video_id: String,
    is_nsfw: bool,
    page_name: String,
    cta_type: Option<String>
});

derive_event!(track_video_clicked {
    publisher_user_id: String,
    is_game_enabled: bool,
    video_id: String,
    game_type: MixpanelPostGameType,
    cta_type: MixpanelVideoClickedCTAType
});

derive_event!(track_video_reported {
    publisher_user_id: String,
    is_game_enabled: bool,
    video_id: String,
    game_type: MixpanelPostGameType,
    is_nsfw: bool,
    report_reason: String
});

derive_event!(track_video_clicked_profile = "video_clicked" => {
    publisher_user_id: String,
    like_count: u64,
    view_count: u64,
    is_game_enabled: bool,
    video_id: String,
    game_type: MixpanelPostGameType,
    cta_type: MixpanelVideoClickedCTAType,
    position: Option<u64>,
    is_own_profile: bool,
    is_nsfw: bool,
    page_name: String
});

derive_event!(track_refer_and_earn { refer_link: String });

derive_event!(track_video_viewed {
    video_id: String,
    publiser_user_id: String,
    game_type: MixpanelPostGameType,
    is_game_enabled: bool
});

derive_event!(track_video_impression {
    video_id: String,
    publisher_user_id: String,
    game_type: MixpanelPostGameType,
    like_count: u64,
    view_count: u64,
    is_nsfw: bool,
    is_game_enabled: bool
});

derive_event!(track_video_started {
    video_id: String,
    publisher_user_id: String,
    game_type: MixpanelPostGameType,
    is_game_enabled: bool
});

derive_event!(track_game_played {
    video_id: String,
    publisher_user_id: String,
    game_type: MixpanelPostGameType,
    stake_amount: u64,
    stake_type: StakeType,
    option_chosen: ChosenGameOption,
    like_count: u64,
    view_count: u64,
    is_game_enabled: bool,
    conclusion: GameConclusion,
    won_loss_amount: String,
    creator_comission_percentage: u64,
    is_nsfw: bool
});

derive_event!(track_game_clicked {
    publisher_user_id: String,
    like_count: u64,
    view_count: u64,
    is_game_enabled: bool,
    video_id: String,
    game_type: MixpanelPostGameType,
    option_chosen: ChosenGameOption,
    stake_amount: u64,
    stake_type: StakeType,
    is_nsfw: bool
});

derive_event!(track_video_upload_success {
    video_id: String,
    creator_comission_percentage: u64,
    is_game_enabled: bool,
    game_type: MixpanelPostGameType,
    upload_type: Option<String>,
    token_type: String
});

derive_event!(track_cents_to_dolr = "cents_to_DOLR" => {
    cents_converted: f64,
    updated_cents_wallet_balance: f64,
    conversion_ratio: f64
});

derive_event!(track_third_party_wallet_transferred {
    token_transferred: f64,
    transferred_to: String,
    token_name: String,
    gas_fee: f64
});

derive_event!(track_how_to_play_clicked {
    video_id: String,
    game_type: MixpanelPostGameType,
    stake_amount: u64,
    stake_type: StakeType,
    option_chosen: ChosenGameOption,
    conclusion: GameConclusion
});

derive_event!(track_username_saved {});

derive_event!(track_video_upload_type_selected {
    upload_type: String
});

derive_event!(track_upload_type_continue_clicked {
    upload_type: String
});

derive_event!(track_video_generation_model_selected { model: String });

derive_event!(track_create_ai_video_clicked {
    model: String,
    token_type: String
});

derive_event!(track_ai_video_generated {
    is_success: bool,
    reason: Option<String>,
    model: String,
    token_type: String
});

derive_event!(track_regenerate_video_clicked { model: String });

impl MixPanelEvent {
    fn clear_auth_journey_page() {
        let (_, set_auth_journey_page) =
            use_cookie::<BottomNavigationCategory, JsonSerdeCodec>(AUTH_JOURNEY_PAGE);
        logging::log!("Clearing auth journey page");
        set_auth_journey_page.set(None);
    }
    pub async fn track_login_success_async(
        global: MixpanelGlobalProps,
        auth_journey: String,
        page_name: BottomNavigationCategory,
    ) {
        let props = track_login_success::new(global, auth_journey, page_name);
        send_event_to_server_async("login_success", props).await;
    }

    pub async fn track_signup_success_async(
        global: MixpanelGlobalProps,
        is_referral: bool,
        referrer_user_id: Option<String>,
        auth_journey: String,
        page_name: BottomNavigationCategory,
    ) {
        let props = track_signup_success::new(
            global,
            is_referral,
            referrer_user_id,
            auth_journey,
            page_name,
        );
        send_event_to_server_async("signup_success", props).await;
        Self::clear_auth_journey_page();
    }
    pub fn track_login_success_sync(
        global: MixpanelGlobalProps,
        auth_journey: String,
        page_name: BottomNavigationCategory,
    ) {
        let props = track_login_success::new(global, auth_journey, page_name);
        send_event_to_server("login_success", props);
        Self::clear_auth_journey_page();
    }

    pub fn track_signup_success_sync(
        global: MixpanelGlobalProps,
        is_referral: bool,
        referrer_user_id: Option<String>,
        auth_journey: String,
        page_name: BottomNavigationCategory,
    ) {
        let props = track_signup_success::new(
            global,
            is_referral,
            referrer_user_id,
            auth_journey,
            page_name,
        );
        send_event_to_server("signup_success", props);
        Self::clear_auth_journey_page();
    }

    pub fn track_page_viewed(page: String, p: MixpanelGlobalProps) {
        let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
            move |_| {
                let props = p.clone();
                match page.as_str() {
                    "/" => {
                        Self::track_home_page_viewed(props);
                    }
                    "/refer-earn" => {
                        Self::track_refer_and_earn_page_viewed(props, REFERRAL_REWARD_SATS);
                    }
                    "/menu" => {
                        Self::track_menu_page_viewed(props);
                    }
                    "/upload-options" => {
                        Self::track_upload_page_viewed(props);
                    }
                    "/profile/edit/username" => {
                        Self::track_edit_username_clicked(props);
                    }
                    page if page.contains("wallet") => {
                        Self::track_wallet_page_viewed(props);
                    }
                    _ => (),
                };
                send_event_to_server("page_viewed", p.clone());

                // TODO: Will be used later
                // if props.page.contains("/profile/") {
                //     let home_props: MixpanelPageViewedProps = props.clone();
                //     let publisher_user_id = home_props
                //         .page
                //         .split("/profile/")
                //         .nth(1)
                //         .and_then(|s| s.split('/').next())
                //         .unwrap_or_default()
                //         .to_string();

                //     if Principal::from_text(publisher_user_id.clone())
                //         .ok()
                //         .is_some()
                //     {
                //         let principal = if home_props.user_id.is_some() {
                //             home_props.user_id.clone().unwrap()
                //         } else {
                //             home_props.visitor_id.clone().unwrap()
                //         };

                //         let is_own_profile = publisher_user_id == principal;

                //         Self::track_profile_page_viewed(MixpanelProfilePageViewedProps {
                //             user_id: home_props.user_id,
                //             visitor_id: home_props.visitor_id,
                //             is_logged_in: home_props.is_logged_in,
                //             canister_id: home_props.canister_id,
                //             is_nsfw_enabled: home_props.is_nsfw_enabled,
                //             is_own_profile,
                //             publisher_user_id,
                //         });
                //     }
                // }
            },
            10.0,
        );
        start(());
    }
}
