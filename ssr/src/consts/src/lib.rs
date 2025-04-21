#[cfg(any(feature = "local-bin", feature = "local-lib"))]
mod local;
use candid::Principal;
#[cfg(any(feature = "local-bin", feature = "local-lib"))]
pub use local::*;

#[cfg(not(any(feature = "local-bin", feature = "local-lib")))]
mod remote;
#[cfg(not(any(feature = "local-bin", feature = "local-lib")))]
pub use remote::*;

use once_cell::sync::Lazy;
use reqwest::Url;

pub const CF_STREAM_BASE: &str = "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com";
pub const FALLBACK_PROPIC_BASE: &str = "https://api.dicebear.com/7.x/big-smile/svg";
// an example URL is "https://imagedelivery.net/abXI9nS4DYYtyR1yFFtziA/gob.5/public";
pub const GOBGOB_PROPIC_URL: &str = "https://imagedelivery.net/abXI9nS4DYYtyR1yFFtziA/gob.";
pub const GOBGOB_TOTAL_COUNT: u32 = 18557;
pub const CF_WATERMARK_UID: &str = "b5588fa1516ca33a08ebfef06c8edb33";
pub const ACCOUNT_CONNECTED_STORE: &str = "account-connected-1";
pub static CF_BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://api.cloudflare.com/client/v4/").unwrap());
pub const NOTIFICATIONS_ENABLED_STORE: &str = "yral-notifications-enabled";
pub const NSFW_TOGGLE_STORE: &str = "nsfw-enabled";
pub const REFERRER_STORE: &str = "referrer";
pub const USER_CANISTER_ID_STORE: &str = "user-canister-id";
pub const USER_PRINCIPAL_STORE: &str = "user-principal";
pub const USER_ONBOARDING_STORE: &str = "user-onboarding";

pub static OFF_CHAIN_AGENT_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://icp-off-chain-agent.fly.dev").unwrap());
pub static OFF_CHAIN_AGENT_GRPC_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://icp-off-chain-agent.fly.dev:443").unwrap()); // pr-91-yral-dapp-off-chain-agent https://icp-off-chain-agent.fly.dev:443
                                                                                  // G-6W5Q2MRX0E to test locally | G-PLNNETMSLM
pub static GTAG_MEASUREMENT_ID: Lazy<&str> = Lazy::new(|| "G-PLNNETMSLM");
pub static DOWNLOAD_UPLOAD_SERVICE: Lazy<Url> =
    Lazy::new(|| Url::parse("https://download-upload-service.fly.dev").unwrap());
pub static ML_FEED_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://yral-ml-feed-server-staging.fly.dev").unwrap());

pub static FALLBACK_USER_INDEX: Lazy<Principal> =
    Lazy::new(|| Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap());

pub const ICP_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub const ICPUMP_LISTING_PAGE_SIZE: usize = 48;

pub const CDAO_SWAP_PRE_READY_TIME_SECS: u64 = 150;

pub const CDAO_SWAP_TIME_SECS: u64 = CDAO_SWAP_PRE_READY_TIME_SECS + 150;

pub const ICPUMP_SEARCH_GRPC_URL: &str = "https://prod-yral-icpumpsearch.fly.dev:443";
pub const NSFW_SERVER_URL: &str = "https://prod-yral-nsfw-classification.fly.dev:443";

pub const CF_KV_ML_CACHE_NAMESPACE_ID: &str = "ea145fc839bd42f9bf2d34b950ddbda5";
pub const CLOUDFLARE_ACCOUNT_ID: &str = "a209c523d2d9646cc56227dbe6ce3ede";

pub const NEW_USER_SIGNUP_REWARD: u64 = 1000;
pub const REFERRAL_REWARD: u64 = 500;

pub mod social {
    pub const TELEGRAM_YRAL: &str = "https://t.me/+c-LTX0Cp-ENmMzI1";
    pub const TELEGRAM_PND: &str = "https://t.me/Pump_Dump_dapp";
    pub const DISCORD: &str = "https://discord.gg/GZ9QemnZuj";
    pub const TWITTER_YRAL: &str = "https://twitter.com/Yral_app";
    pub const TWITTER_PND: &str = "https://x.com/dolr_ai";
    pub const IC_WEBSITE: &str = "https://vyatz-hqaaa-aaaam-qauea-cai.ic0.app";
}

pub mod auth {
    use web_time::Duration;

    /// Delegation Expiry, 7 days
    pub const DELEGATION_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 7);
    /// Refresh expiry, 30 days
    pub const REFRESH_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 30);
    pub const REFRESH_TOKEN_COOKIE: &str = "user-identity";
}

#[cfg(feature = "oauth-ssr")]
pub mod google {
    pub const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
    pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
    pub const GOOGLE_ISSUER_URL: &str = "https://accounts.google.com";
}
