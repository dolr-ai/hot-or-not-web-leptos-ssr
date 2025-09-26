#[cfg(any(feature = "local-bin", feature = "local-lib"))]
mod local;

use std::collections::BTreeSet;

use candid::Principal;
#[cfg(any(feature = "local-bin", feature = "local-lib"))]
pub use local::*;

#[cfg(not(any(feature = "local-bin", feature = "local-lib")))]
mod remote;
#[cfg(not(any(feature = "local-bin", feature = "local-lib")))]
pub use remote::*;

use once_cell::sync::Lazy;
use reqwest::Url;
use serde::{Deserialize, Serialize};

// TODO: make it consistent with the actual bet amount
pub const CENTS_IN_E6S: u64 = 1_000_000;
pub const SATS_TO_BTC_CONVERSION_RATIO: f64 = 0.00000001;
pub const CF_STREAM_BASE: &str = "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com";
pub const FALLBACK_PROPIC_BASE: &str = "https://api.dicebear.com/7.x/big-smile/svg";
// an example URL is "https://imagedelivery.net/abXI9nS4DYYtyR1yFFtziA/gob.5/public";
pub const GOBGOB_PROPIC_URL: &str = "https://imagedelivery.net/abXI9nS4DYYtyR1yFFtziA/gob.";
pub const GOBGOB_TOTAL_COUNT: u32 = 18557;
pub const CF_WATERMARK_UID: &str = "b5588fa1516ca33a08ebfef06c8edb33";
pub const ACCOUNT_CONNECTED_STORE: &str = "account-connected-1";
pub const DEVICE_ID: &str = "device_id";
pub const CUSTOM_DEVICE_ID: &str = "custom_device_id";
pub const AUTH_JOURNET: &str = "auth_journey";
pub const AUTH_JOURNEY_PAGE: &str = "auth_journey_page";
pub static CF_BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://api.cloudflare.com/client/v4/").unwrap());
pub const NOTIFICATIONS_ENABLED_STORE: &str = "yral-notifications-enabled";
pub const NOTIFICATION_MIGRATED_STORE: &str = "notifications-migrated";
pub const NSFW_TOGGLE_STORE: &str = "nsfw-enabled";
pub const NSFW_ENABLED_COOKIE: &str = "nsfw-enabled-cookie";
pub const REFERRER_COOKIE: &str = "referrer";
pub const USER_CANISTER_ID_STORE: &str = "user-canister-id";
pub const USER_PRINCIPAL_STORE: &str = "user-principal";
pub const USER_INTERNAL_STORE: &str = "user-internal";
pub const WALLET_BALANCE_STORE_KEY: &str = "wallet-balance-sats";

pub static OFF_CHAIN_AGENT_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://pr-293-dolr-ai-off-chain-agent.fly.dev").unwrap());

pub static ANALYTICS_SERVER_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://marketing-analytics-server.fly.dev").unwrap());

pub static SMILEY_GAME_STATS_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://us-central1-yral-mobile.cloudfunctions.net").unwrap());

pub static OFF_CHAIN_AGENT_GRPC_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://pr-293-dolr-ai-off-chain-agent.fly.dev:443").unwrap());
pub static DOWNLOAD_UPLOAD_SERVICE: Lazy<Url> =
    Lazy::new(|| Url::parse("https://download-upload-service.fly.dev").unwrap());

pub static FALLBACK_USER_INDEX: Lazy<Principal> =
    Lazy::new(|| Principal::from_text("rimrc-piaaa-aaaao-aaljq-cai").unwrap());

pub const ICP_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

pub const CF_KV_ML_CACHE_NAMESPACE_ID: &str = "ea145fc839bd42f9bf2d34b950ddbda5";
pub const CLOUDFLARE_ACCOUNT_ID: &str = "a209c523d2d9646cc56227dbe6ce3ede";

pub const AUTH_UTIL_COOKIES_MAX_AGE_MS: i64 = 400 * 24 * 60 * 60 * 1000; // 400 days

pub const MAX_VIDEO_ELEMENTS_FOR_FEED: usize = 200;

pub mod social {
    pub const TELEGRAM_YRAL: &str = "https://t.me/+c-LTX0Cp-ENmMzI1";
    pub const DISCORD: &str = "https://discord.gg/GZ9QemnZuj";
    pub const TWITTER_YRAL: &str = "https://twitter.com/Yral_app";
    pub const IC_WEBSITE: &str = "https://vyatz-hqaaa-aaaam-qauea-cai.ic0.app";
}

pub mod auth {
    use web_time::Duration;

    /// Delegation Expiry, 7 days
    pub const DELEGATION_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 7);
    /// Refresh expiry, 29 days
    pub const REFRESH_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 29);
    pub const REFRESH_TOKEN_COOKIE: &str = "user-identity";
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoginProvider {
    Any,
    Google,
    Apple,
}

#[cfg(feature = "oauth-ssr")]
pub mod yral_auth {
    use jsonwebtoken::DecodingKey;
    use std::sync::LazyLock;

    pub const YRAL_AUTH_AUTHORIZATION_URL: &str = "https://auth.yral.com/oauth/auth";
    pub const YRAL_AUTH_TOKEN_URL: &str = "https://auth.yral.com/oauth/token";
    pub const YRAL_AUTH_ISSUER_URL: &str = "https://auth.yral.com";

    pub static YRAL_AUTH_TRUSTED_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
        let pem = "-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEoqN3/0RNfrnrnYGxKBgy/qHnmITr
+6ucjxStx7tjA30QJZlWzo0atxmY8y9dUR+eKQI0SnbQds4xLEU8+JGm8Q==
-----END PUBLIC KEY-----";
        DecodingKey::from_ec_pem(pem.as_bytes()).unwrap()
    });

    pub const YRAL_AUTH_CLIENT_ID_ENV: &str = "YRAL_AUTH_CLIENT_ID";
}

pub const UPLOAD_URL: &str = "https://yral-upload-video.go-bazzinga.workers.dev";

pub const DOLR_AI_ROOT_CANISTER: &str = "67bll-riaaa-aaaaq-aaauq-cai";
pub const DOLR_AI_LEDGER_CANISTER: &str = "6rdgd-kyaaa-aaaaq-aaavq-cai";
pub const CKBTC_LEDGER_CANISTER: &str = "mxzaz-hqaaa-aaaar-qaada-cai";
pub const USDC_LEDGER_CANISTER: &str = "xevnm-gaaaa-aaaar-qafnq-cai";

// Hetzner S3 Configuration
pub mod hetzner_s3 {
    use once_cell::sync::Lazy;
    use reqwest::Url;

    pub const BUCKET_NAME: &str = "yral-profile";
    pub const REGION: &str = "eu-central";
    pub const ENDPOINT: &str = "https://hel1.your-objectstorage.com";
    pub const NETWORK_ZONE: &str = "eu-central";

    pub static S3_ENDPOINT_URL: Lazy<Url> = Lazy::new(|| Url::parse(ENDPOINT).unwrap());

    // Access credentials from environment variables
    pub fn get_access_key() -> String {
        std::env::var("HETZNER_S3_ACCESS_KEY").expect("HETZNER_S3_ACCESS_KEY not set")
    }

    pub fn get_secret_key() -> String {
        std::env::var("HETZNER_S3_SECRET_KEY").expect("HETZNER_S3_SECRET_KEY not set")
    }

    // Helper to construct S3 URLs for objects
    pub fn get_object_url(key: &str) -> String {
        format!("{ENDPOINT}/{BUCKET_NAME}/{key}")
    }
}

pub const SATS_CKBTC_CANISTER: &str =
    "zg7n3-345by-nqf6o-3moz4-iwxql-l6gko-jqdz2-56juu-ja332-unymr-fqe";

pub const USER_ONBOARDING_STORE_KEY: &str = "user-onboarding";
#[derive(Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct UserOnboardingStore {
    pub has_seen_onboarding: bool,
    pub has_seen_hon_bet_help: bool,
}

pub static WHITELIST_FOR_SATS_CLEARING: Lazy<BTreeSet<&'static str>> = Lazy::new(|| {
    BTreeSet::from([
        "p5uh7-k3l7t-qztp2-cqwf4-tzird-7tzpa-mhzyf-udy5f-oj6fg-qnh34-5qe",
        "uau2g-57gtt-6vcnr-zalkn-t76ad-tejac-cdt62-qdv25-wfl5l-fimkk-yae",
        "ebp2n-emlpn-pz52l-oymqw-gwyt3-uadux-4wamr-sbkva-ruwat-i2dbf-qqe",
        "gugug-6npnh-vw2m2-mnklf-dy3e6-y2vy6-iuk7v-wipro-pck2i-uubrx-qae",
        "ifzkt-bygsq-27zoa-2raqh-fiyly-ishen-cbzbs-zqczb-thb75-igxbv-iqe",
        "bcvey-te2p5-rrec4-dbl5p-3rgvv-xugvq-rtrls-4wmew-zzbtq-pmtg2-wqe",
        "4ru3m-prz2p-5cpf2-xr6hf-mrfm5-2ezep-2xeyn-emhlm-rs4bl-7ybdc-nae",
        "ihn6s-7fdnu-kn7gy-5uony-4d7ml-f5mll-rw44e-mqzi6-ounkl-it2yx-bae",
        "nvvil-oucwc-2rug5-rwsld-bvlnw-nmmal-bpiht-qeffm-hhgmh-uqtfi-uae",
        "fe46h-leqvj-s7erb-3qrtr-fqfhr-rgyh6-2xu5f-gb57y-yxi4d-ashjb-iqe",
        "hwh4g-55ttk-kqell-bxnkl-4aypf-mhosp-27m7q-y7b45-44bk7-5f2f2-5ae",
        "laxmg-tq2ji-wxggj-25l2f-4io5o-mo2ao-2t24w-qsg3d-er5s3-r7zwe-xae",
        "34kr4-lmwqy-fgnqd-pspk6-fccjo-ch6mt-o6sfx-ao5c4-psalf-nd3oa-bqe",
        "nba27-vdzlk-qsnd5-dxm7w-7lztn-3js2u-lrx6u-hicfq-imrx4-nsobn-7qe",
        "cp7dg-n36pb-3bcja-caqkm-vcanj-t37c7-p7ptb-h3tls-6srot-2jz7m-6ae",
        "7vovb-nk3ke-4cptr-p57qb-wtcrl-rlc2f-4kweo-tksld-pfq2p-ptkiw-pqe",
        "bq5wq-gug6n-aone7-ae234-7yb34-zgg7u-ittv6-xx3jp-qi3qi-qzwez-jae",
        "34yzw-zrmgu-vg6ms-2uj2a-czql2-7y4bu-mt5so-ckrtz-znelw-yyvr4-2ae",
        "jzkxb-xd5wj-mfcgt-zvu4x-qyyfn-ec42s-ms65i-aalxv-aoc4o-donmx-gqe",
        "4ag7l-5julz-krtnd-5dpvv-rs63v-uliqm-tl2hc-tncke-g2weg-w3tou-5ae",
        "v6uzq-up7cy-os5rl-oxyp6-vodok-prm66-lbrwu-r6qes-otn5a-kwfeb-hae",
        "yjdyb-ueeju-oh7mq-sgt2f-auhwf-qwmqm-4tcps-44qhw-afmqd-uv7kf-mae",
        "l5vxu-jqbm3-neige-mzqus-rzquu-75gud-pkt3d-okgzs-flu6r-ef5zk-jqe",
        "itcvb-jhhlu-7scol-dtxgr-nivib-s4gdn-o2cmz-ymnzs-4ljdi-jotpn-zae",
        "3dx6o-c4iql-jihvn-7hx5g-gmqxz-7oyap-tu2ef-ap7nj-6vliy-u727q-cqe",
        "acpnt-m2nr5-5hjsi-h25wj-r3j55-kydra-2dshv-lq6ds-bqldu-jpjyo-fqe",
        "dc23f-7vyti-xp4vt-gqhlt-3qq2p-qoocg-iweu4-vv4wv-ur56b-jq4ap-nae",
        "nwfrx-xxjzx-uaveh-sqctt-nngud-stze4-k2ogj-npntl-cjyle-oda6r-aae",
        "fbhxs-yfeo2-e3zxa-2nitl-ormfo-imck5-4m57n-i3367-zkuci-7usfo-nqe",
    ])
});
