use once_cell::sync::Lazy;
use reqwest::Url;

pub static METADATA_API_BASE: Lazy<Url> =
    Lazy::new(|| Url::parse("https://yral-metadata.fly.dev").unwrap());

pub const AGENT_URL: &str = "https://ic0.app";

pub static PUMP_AND_DUMP_WORKER_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://yral-pump-n-dump.go-bazzinga.workers.dev/").unwrap());

pub const STDB_URL: &str = "https://maincloud.spacetimedb.com";

// TODO: switch to prod module
pub const BACKEND_MODULE_IDENTITY: &str = "pr-6-yral-backend";

// TODO: load secret from env
pub static STDB_ACCESS_TOKEN: Lazy<Option<String>> = Lazy::new(|| None);
