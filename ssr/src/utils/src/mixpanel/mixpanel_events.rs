use candid::Principal;
use codee::string::FromToStringCodec;
use consts::AUTH_JOURNET;
use consts::DEVICE_ID;
use consts::NSFW_TOGGLE_STORE;
use consts::REFERRAL_REWARD;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::storage::use_local_storage;
use leptos_use::use_timeout_fn;
use leptos_use::UseTimeoutFnReturn;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yral_canisters_common::utils::vote::VoteKind;
use yral_canisters_common::Canisters;

use crate::event_streaming::events::EventCtx;
use crate::event_streaming::events::HistoryCtx;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = mixpanel, catch)]
    fn track(event_name: &str, properties: JsValue) -> Result<(), JsValue>;

    /// mixpanel.identify(user_id)
    #[wasm_bindgen(js_namespace = mixpanel, catch)]
    fn identify(user_id: &str) -> Result<(), JsValue>;
}

/// Call once you know the logged-in user's ID
pub fn identify_user(user_id: &str) {
    let _ = identify(user_id);
}

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

/// Generic helper: serializes `props` and calls Mixpanel.track
pub fn track_event<T>(event_name: &str, props: T)
where
    T: Serialize,
{
    let track_props = serde_wasm_bindgen::to_value(&props);
    if let Ok(track_props) = track_props {
        let _ = track(event_name, track_props);
    }
    send_event_to_server(event_name, props);
}

fn send_event_to_server<T>(event_name: &str, props: T)
where
    T: Serialize,
{
    let mut props = serde_json::to_value(&props).unwrap();
    props["event"] = event_name.into();
    props["$device_id"] = MixpanelGlobalProps::get_device_id().into();
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
    spawn_local(async {
        let res = track_event_server_fn(props).await;
        match res {
            Ok(_) => {}
            Err(e) => logging::error!("Error tracking Mixpanel event: {}", e),
        }
    });
}

/// Global properties for Mixpanel events
#[derive(Clone, Serialize)]
pub struct MixpanelGlobalProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
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
        }
    }
    /// Load global state (login, principal, NSFW toggle)
    pub fn try_get(cans: &Canisters<true>, is_logged_in: bool) -> Self {
        let (is_nsfw_enabled, _, _) =
            use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
        let is_nsfw_enabled = is_nsfw_enabled.get_untracked();

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
        }
    }

    pub fn get_device_id() -> String {
        let (device_id, set_device_id, _) =
            use_local_storage::<String, FromToStringCodec>(DEVICE_ID);
        // Extracting the device ID value
        let device_id_value = device_id.get_untracked();
        if device_id_value.is_empty() {
            let new_device_id = uuid::Uuid::new_v4().to_string();
            set_device_id.set(new_device_id.clone());
            new_device_id
        } else {
            device_id_value
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
            let (is_nsfw_enabled, _, _) =
                use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
            let is_nsfw_enabled = is_nsfw_enabled.get_untracked();

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
        }
    }
}

#[derive(Serialize)]
pub struct MixpanelBottomBarPageViewedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
}

#[derive(Serialize, Clone)]
pub struct MixpanelDeleteAccountClickedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub page_name: String,
}

#[derive(Serialize)]
pub struct MixpanelReferAndEarnPageViewedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub referral_bonus: u64,
}
#[derive(Serialize)]
pub struct MixpanelVideoUploadFailureProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub error: String,
}
#[derive(Serialize)]
pub struct MixpanelProfilePageViewedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub is_own_profile: bool,
    pub publisher_user_id: String,
}
#[derive(Serialize)]
pub struct MixpanelWithdrawTokenClickedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub token_clicked: StakeType,
}

#[derive(Serialize)]
pub struct MixpanelClaimAirdropClickedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub token_type: StakeType,
}

#[derive(Serialize)]
pub struct MixpanelAirdropClaimedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub token_type: StakeType,
    pub is_success: bool,
    pub claimed_amount: u64,
}

#[derive(Serialize, Clone)]
pub struct MixpanelPageViewedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub page: String,
}

#[derive(Serialize)]
pub struct MixpanelBottomNavigationProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub category_name: BottomNavigationCategory,
}

use std::convert::TryFrom;

impl TryFrom<String> for BottomNavigationCategory {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.contains("/profile/") {
            return Ok(BottomNavigationCategory::Profile);
        } else if value.contains("/wallet/") {
            return Ok(BottomNavigationCategory::Wallet);
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
pub struct MixpanelSignupSuccessProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub is_referral: bool,
    pub referrer_user_id: Option<String>,
    pub auth_journey: String,
}

#[derive(Serialize)]
pub struct MixpanelLoginSuccessProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub auth_journey: String,
}

#[derive(Serialize)]
pub struct MixpanelSatsToBtcConvertedProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub sats_converted: f64,
    pub conversion_ratio: f64,
}

#[derive(Serialize)]
pub struct MixpanelNsfwToggleProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub publisher_user_id: String,
    pub video_id: String,
    pub is_nsfw: bool,
}

#[derive(Serialize)]
pub struct MixpanelVideoClickedProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub publisher_user_id: String,
    pub like_count: u64,
    pub view_count: u64,
    pub is_game_enabled: bool,
    pub video_id: String,
    pub game_type: MixpanelPostGameType,
    pub cta_type: MixpanelVideoClickedCTAType,
    pub is_nsfw: bool,
}

#[derive(Serialize)]
pub struct MixpanelReferAndEarnProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub refer_link: String,
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
}

#[derive(Serialize)]
pub struct MixpanelVideoViewedProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub video_id: String,
    pub publisher_user_id: String,
    pub game_type: MixpanelPostGameType,
    pub like_count: u64,
    pub view_count: u64,
    pub is_nsfw: bool,
    pub is_game_enabled: bool,
}

#[derive(Serialize)]
pub struct MixpanelVideoStartedProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub video_id: String,
    pub publisher_user_id: String,
    pub game_type: MixpanelPostGameType,
    pub like_count: u64,
    pub view_count: u64,
    pub is_nsfw: bool,
    pub is_game_enabled: bool,
}

#[derive(Serialize)]
pub struct MixpanelGamePlayedProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub video_id: String,
    pub publisher_user_id: String,
    pub game_type: MixpanelPostGameType,
    pub stake_amount: u64,
    pub stake_type: StakeType,
    pub option_chosen: ChosenGameOption,
    pub like_count: u64,
    pub view_count: u64,
    pub is_game_enabled: bool,
    pub conclusion: GameConclusion,
    pub won_loss_amount: String,
    pub creator_commision_percentage: u64,
    pub is_nsfw: bool,
}

#[derive(Serialize)]
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
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BottomNavigationCategory {
    UploadVideo,
    Profile,
    Menu,
    Home,
    Wallet,
}

#[derive(Serialize)]
pub struct MixpanelVideoUploadSuccessProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub video_id: String,
    pub creator_commision_percentage: u64,
    // pub publisher_user_id: String,
    pub is_game_enabled: bool,
    pub game_type: MixpanelPostGameType,
}

#[derive(Serialize)]
pub struct MixpanelCentsToDolrProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub cents_converted: f64,
    pub updated_cents_wallet_balance: f64,
    pub conversion_ratio: f64,
}

#[derive(Serialize)]
pub struct MixpanelThirdPartyWalletTransferredProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
    pub token_transferred: f64,
    // pub updated_token_wallet_balance: f64,
    pub transferred_to: String,
    pub token_name: String,
    pub gas_fee: f64,
}

pub struct MixPanelEvent;
impl MixPanelEvent {
    /// Call once you know the logged-in user's ID
    pub fn identify_user(user_id: &str) {
        let _ = identify(user_id);
    }
    pub fn track_home_page_viewed(p: MixpanelBottomBarPageViewedProps) {
        track_event("home_page_viewed", p);
    }
    pub fn track_wallet_page_viewed(p: MixpanelBottomBarPageViewedProps) {
        send_event_to_server("wallet_page_viewed", p);
    }
    pub fn track_upload_page_viewed(p: MixpanelBottomBarPageViewedProps) {
        send_event_to_server("upload_video_page_viewed", p);
    }
    pub fn track_menu_page_viewed(p: MixpanelBottomBarPageViewedProps) {
        send_event_to_server("menu_page_viewed", p);
    }
    pub fn track_delete_account_clicked(p: MixpanelDeleteAccountClickedProps) {
        send_event_to_server("delete_account_clicked", p);
    }
    pub fn track_delete_account_confirmed(p: MixpanelDeleteAccountClickedProps) {
        send_event_to_server("delete_account_confirmed", p);
    }
    pub fn track_account_deleted(p: MixpanelDeleteAccountClickedProps) {
        send_event_to_server("account_deleted", p);
    }
    pub fn track_refer_and_earn_page_viewed(p: MixpanelReferAndEarnPageViewedProps) {
        send_event_to_server("refer_and_earn_page_viewed", p);
    }
    // pub fn track_profile_page_viewed(p: MixpanelProfilePageViewedProps) {
    //     send_event_to_server("profile_page_viewed", p);
    // }
    pub fn track_withdraw_tokens_clicked(p: MixpanelWithdrawTokenClickedProps) {
        send_event_to_server("withdraw_tokens_clicked", p);
    }
    pub fn track_claim_airdrop_clicked(p: MixpanelClaimAirdropClickedProps) {
        send_event_to_server("claim_airdrop_clicked", p);
    }
    pub fn track_airdrop_claimed(p: MixpanelAirdropClaimedProps) {
        send_event_to_server("airdrop_claimed", p);
    }
    pub fn track_referral_link_copied(p: MixpanelReferAndEarnPageViewedProps) {
        send_event_to_server("referral_link_copied", p);
    }
    pub fn track_share_invites_clicked(p: MixpanelReferAndEarnPageViewedProps) {
        send_event_to_server("share_invites_clicked", p);
    }
    pub fn track_video_upload_error_shown(p: MixpanelVideoUploadFailureProps) {
        send_event_to_server("video_upload_error_shown", p);
    }

    pub fn track_page_viewed(p: MixpanelPageViewedProps) {
        let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
            move |_| {
                let props = p.clone();
                if props.page == "/" {
                    let home_props: MixpanelPageViewedProps = props.clone();
                    Self::track_home_page_viewed(MixpanelBottomBarPageViewedProps {
                        user_id: home_props.user_id,
                        visitor_id: home_props.visitor_id,
                        is_logged_in: home_props.is_logged_in,
                        canister_id: home_props.canister_id,
                        is_nsfw_enabled: home_props.is_nsfw_enabled,
                    });
                }
                if props.page.contains("wallet") {
                    let home_props: MixpanelPageViewedProps = props.clone();
                    Self::track_wallet_page_viewed(MixpanelBottomBarPageViewedProps {
                        user_id: home_props.user_id,
                        visitor_id: home_props.visitor_id,
                        is_logged_in: home_props.is_logged_in,
                        canister_id: home_props.canister_id,
                        is_nsfw_enabled: home_props.is_nsfw_enabled,
                    });
                }
                if props.page == "/refer-earn" {
                    let home_props: MixpanelPageViewedProps = props.clone();
                    Self::track_refer_and_earn_page_viewed(MixpanelReferAndEarnPageViewedProps {
                        user_id: home_props.user_id,
                        visitor_id: home_props.visitor_id,
                        is_logged_in: home_props.is_logged_in,
                        canister_id: home_props.canister_id,
                        is_nsfw_enabled: home_props.is_nsfw_enabled,
                        referral_bonus: REFERRAL_REWARD,
                    });
                }
                if props.page == "/menu" {
                    let home_props: MixpanelPageViewedProps = props.clone();
                    Self::track_menu_page_viewed(MixpanelBottomBarPageViewedProps {
                        user_id: home_props.user_id,
                        visitor_id: home_props.visitor_id,
                        is_logged_in: home_props.is_logged_in,
                        canister_id: home_props.canister_id,
                        is_nsfw_enabled: home_props.is_nsfw_enabled,
                    });
                }
                if props.page == "/upload" {
                    let home_props: MixpanelPageViewedProps = props.clone();
                    Self::track_upload_page_viewed(MixpanelBottomBarPageViewedProps {
                        user_id: home_props.user_id,
                        visitor_id: home_props.visitor_id,
                        is_logged_in: home_props.is_logged_in,
                        canister_id: home_props.canister_id,
                        is_nsfw_enabled: home_props.is_nsfw_enabled,
                    });
                }
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
                track_event("page_viewed", props);
            },
            10.0,
        );
        start(());
    }

    pub fn track_bottom_navigation_clicked(p: MixpanelBottomNavigationProps) {
        send_event_to_server("bottom_navigation_clicked", p);
    }

    pub fn track_signup_success(p: MixpanelSignupSuccessProps) {
        track_event("signup_success", p);
    }

    pub fn track_login_success(p: MixpanelLoginSuccessProps) {
        track_event("login_success", p);
    }

    pub fn track_sats_to_btc_conversion_failed(p: MixpanelSatsToBtcConvertedProps) {
        track_event("sats_to_btc_converted", p);
    }

    pub fn track_sats_to_btc_converted(p: MixpanelSatsToBtcConvertedProps) {
        track_event("sats_to_btc_converted", p);
    }

    pub fn track_nsfw_true(p: MixpanelNsfwToggleProps) {
        track_event("nsfw_enabled", p);
    }

    pub fn track_nsfw_false(p: MixpanelNsfwToggleProps) {
        track_event("NSFW_False", p);
    }

    pub fn track_video_clicked(p: MixpanelVideoClickedProps) {
        track_event("video_clicked", p);
    }

    pub fn track_refer_and_earn(p: MixpanelReferAndEarnProps) {
        track_event("refer_and_earn", p);
    }

    pub fn track_video_viewed(p: MixpanelVideoViewedProps) {
        track_event("video_viewed", p);
    }

    pub fn track_video_started(p: MixpanelVideoStartedProps) {
        track_event("video_started", p);
    }

    pub fn track_game_played(p: MixpanelGamePlayedProps) {
        track_event("game_played", p);
    }

    pub fn track_video_upload_success(p: MixpanelVideoUploadSuccessProps) {
        track_event("video_upload_success", p);
    }

    pub fn track_cents_to_dolr(p: MixpanelCentsToDolrProps) {
        track_event("cents_to_DOLR", p);
    }

    pub fn track_third_party_wallet_transferred(p: MixpanelThirdPartyWalletTransferredProps) {
        track_event("third_party_wallet_transferred", p);
    }
}
