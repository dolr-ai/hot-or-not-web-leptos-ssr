use codee::string::FromToStringCodec;
use consts::NSFW_TOGGLE_STORE;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::storage::use_local_storage;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yral_canisters_common::utils::vote::VoteKind;
use yral_canisters_common::Canisters;

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
    let token = std::env::var("ANALYTICS_SERVER_TOKEN").expect("ANALYTICS_SERVER_TOKEN is not set");
    Client::new()
        .post("https://marketing-analytics-server.fly.dev/api/send_event")
        .json(&props)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Mixpanel track error: {e:?}")))?;
    Ok(())
}

/// Generic helper: serializes `props` and calls Mixpanel.track
pub fn track_event<T>(event_name: &str, props: T)
where
    T: Serialize,
{
    let mut props = serde_json::to_value(&props).unwrap();
    props["event"] = event_name.into();
    let user_id = props.get("user_id").and_then(Value::as_str);
    props["principal"] = if user_id.is_some() {
        user_id.into()
    } else {
        props.get("visitor_id").and_then(Value::as_str).into()
    };
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
pub struct MixpanelHomePageViewedProps {
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
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
}

#[derive(Serialize)]
pub struct MixpanelLoginSuccessProps {
    // #[serde(flatten)]
    pub user_id: Option<String>,
    pub visitor_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: String,
    pub is_nsfw_enabled: bool,
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
    pub updated_sats_wallet_balance: f64,
    pub updated_token_wallet_balance: f64,
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

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MixpanelVideoClickedCTAType {
    Like,
    Share,
    ReferAndEarn,
    Report,
    NsfwTrue,
    NsfwFalse,
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
    pub option_chosen: VoteKind,
    pub like_count: u64,
    pub view_count: u64,
    pub is_game_enabled: bool,
    pub conclusion: GameConclusion,
    pub won_amount: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameConclusion {
    Pending,
    Win,
    Loss,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StakeType {
    SATs,
    Cents,
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
    pub fn track_home_page_viewed(p: MixpanelHomePageViewedProps) {
        track_event("home_page_viewed", p);
    }

    pub fn track_signup_success(p: MixpanelSignupSuccessProps) {
        track_event("signup_success", p);
    }

    pub fn track_login_success(p: MixpanelLoginSuccessProps) {
        track_event("login_success", p);
    }

    pub fn track_sats_to_btc_converted(p: MixpanelSatsToBtcConvertedProps) {
        track_event("sats_to_btc_converted", p);
    }

    pub fn track_nsfw_true(p: MixpanelNsfwToggleProps) {
        track_event("NSFW_True", p);
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
