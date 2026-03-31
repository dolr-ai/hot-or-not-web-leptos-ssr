use once_cell::sync::Lazy;
use reqwest::Url;

pub static METADATA_API_BASE: Lazy<Url> =
    Lazy::new(|| Url::parse("https://metadata.yral.com").unwrap());

pub const AGENT_URL: &str = "https://ic0.app";

pub static PUMP_AND_DUMP_WORKER_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://yral-pump-n-dump.go-bazzinga.workers.dev/").unwrap());

pub const BACKEND_MODULE_IDENTITY: &str = "yral-backend";
