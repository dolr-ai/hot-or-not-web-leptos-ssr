use crate::event_streaming::events::account_connected_reader;
use candid::Principal;
use leptos::prelude::GetUntracked;
use leptos::prelude::RwSignal;
use leptos::prelude::*;
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
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

/// Generic helper: serializes `props` and calls Mixpanel.track
pub fn track_event<T>(event_name: &str, props: T)
where
    T: Serialize,
{
    let js_props = to_value(&props).expect("failed to serialize Mixpanel props");
    let _ = track(event_name, js_props);
}

#[derive(Serialize, Clone)]
pub struct UserCanisterAndPrincipal {
    pub user_id: String,
    pub canister_id: String,
}

#[derive(Clone)]
pub struct IsHotOrNot {
    post: RwSignal<BTreeMap<(Principal, u64), bool>>,
}

impl IsHotOrNot {
    pub fn register() {
        provide_context(IsHotOrNot {
            post: RwSignal::new(BTreeMap::new()),
        });
    }

    pub fn set(&self, canister_id: Principal, post_id: u64, is_hot_or_not: bool) {
        self.post.update(|f| {
            f.insert((canister_id, post_id), is_hot_or_not);
        });
    }

    pub fn get(&self, canister_id: Principal, post_id: u64) -> bool {
        *self
            .post
            .get_untracked()
            .get(&(canister_id, post_id))
            .unwrap_or(&false)
    }
}

impl UserCanisterAndPrincipal {
    pub fn try_get(cans: &Canisters<true>) -> Option<Self> {
        Some(UserCanisterAndPrincipal {
            user_id: cans.user_principal().to_text(),
            canister_id: cans.user_canister().to_text(),
        })
    }
}

/// Fired once a video has been watched for ≥3 seconds
#[derive(Serialize)]
pub struct MixpanelVideoViewedProps {
    pub publisher_user_id: String,
    pub user_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: Option<String>,
    pub video_id: String,
    pub is_nsfw: bool,
    pub is_hotor_not: bool,
    pub view_count: u64,
    pub like_count: u64,
    // pub share_count: u64,
}

/// Fired once a video upload completes successfully
#[derive(Serialize)]
pub struct MixpanelVideoUploadSuccessfulProps {
    pub user_id: Option<String>,
    pub canister_id: Option<String>,
    pub is_nsfw: bool,
    pub is_hotor_not: bool,
}

/// Fired once the user has logged in successfully
#[derive(Serialize)]
pub struct MixpanelLoginSuccessfulProps {
    pub user_id: String,
    pub canister_id: Option<String>,
    pub referred_by: Option<String>,
}

/// Fired whenever the NSFW toggle is flipped
#[derive(Serialize)]
pub struct MixpanelNsfwToggleProps {
    pub user_id: Option<String>,
    pub publisher_user_id: String,
    pub is_logged_in: bool,
    pub canister_id: Option<String>,
    pub video_id: String,
}

/// Fired when the user taps “Like”
#[derive(Serialize)]
pub struct MixpanelLikeVideoProps {
    pub publisher_user_id: String,
    pub user_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: Option<String>,
    pub video_id: String,
    pub is_nsfw: bool,
    pub is_hotor_not: bool,
    pub view_count: u64,
    pub like_count: u64,
    // pub share_count: u64,
}

/// Fired when the user taps Hot/Not
#[derive(Serialize)]
pub struct MixpanelHotOrNotPlayedProps {
    pub publisher_user_id: String,
    pub user_id: Option<String>,
    pub is_logged_in: bool,
    pub canister_id: Option<String>,
    pub video_id: String,
    pub is_nsfw: bool,
    pub is_hotor_not: bool,
    pub view_count: u64,
    pub like_count: u64,
    // pub creator_commission: f64,
}

/// Fired once a Hot/Not game round concludes
#[derive(Serialize)]
pub struct MixpanelUserGameConcludedProps {
    pub user_id: Option<String>,
    pub video_id: String,
    pub vote_direction: String, // e.g. "hot" or "not"
    pub conclusion: String,     // e.g. "won" or "lost"
    pub is_nsfw: bool,
    pub like_count: u64,
    pub stake_amount: f64,
    pub won_amount: f64,
    pub updated_cents_wallet_balance: u64,
}

/// Fired when the user withdraws cents → DOLR
#[derive(Serialize)]
pub struct MixpanelCentsToDolrProps {
    pub user_id: Option<String>,
    pub cents_converted: f64,
    pub updated_cents_wallet_balance: f64,
    // pub updated_dolr_wallet_balance: f64,
    pub conversion_ratio: f64,
}

/// Fired when the user sends DOLR to a 3rd‑party wallet
#[derive(Serialize)]
pub struct MixpanelDolrTo3rdPartyWalletProps {
    pub user_id: Option<String>,
    pub token_transferred: f64,
    // pub updated_wallet_balance: f64,
    pub transferred_wallet: String,
    pub gas_fee: f64,
    pub token_name: String,
}

pub struct MixPanelEvent;

impl MixPanelEvent {
    pub fn track_video_viewed(mut p: MixpanelVideoViewedProps) {
        let (is_connected, _) = account_connected_reader();
        p.is_logged_in = is_connected.get_untracked();
        track_event("video_viewed", p);
    }

    pub fn track_video_upload_successful(p: MixpanelVideoUploadSuccessfulProps) {
        track_event("video_upload_successful", p);
    }

    pub fn track_login_successful(p: MixpanelLoginSuccessfulProps) {
        track_event("login_successful", p);
    }

    pub fn track_nsfw_true(mut p: MixpanelNsfwToggleProps) {
        let (is_connected, _) = account_connected_reader();
        p.is_logged_in = is_connected.get_untracked();
        track_event("NSFW_True", p);
    }

    pub fn track_nsfw_false(mut p: MixpanelNsfwToggleProps) {
        let (is_connected, _) = account_connected_reader();
        p.is_logged_in = is_connected.get_untracked();
        track_event("NSFW_False", p);
    }

    pub fn track_like_video(mut p: MixpanelLikeVideoProps) {
        let (is_connected, _) = account_connected_reader();
        p.is_logged_in = is_connected.get_untracked();
        track_event("like_video", p);
    }

    pub fn track_hot_or_not_played(mut p: MixpanelHotOrNotPlayedProps) {
        let (is_connected, _) = account_connected_reader();
        p.is_logged_in = is_connected.get_untracked();
        track_event("hot_or_not_played", p);
    }

    pub fn track_user_game_concluded(p: MixpanelUserGameConcludedProps) {
        track_event("user_game_concluded", p);
    }

    pub fn track_cents_to_dolr(p: MixpanelCentsToDolrProps) {
        track_event("cents_to_DOLR", p);
    }

    pub fn track_yral_to_3rd_party_wallet(p: MixpanelDolrTo3rdPartyWalletProps) {
        track_event("DOLR_to_3rd_party_wallet", p);
    }
}
