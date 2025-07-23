
#[derive(Clone, Debug, PartialEq)]
pub enum AppType {
    YRAL,
    HotOrNot,
}

impl AppType {
    pub fn from_host(host: &str) -> Self {
        if host.contains("hotornot") {
            AppType::HotOrNot
        } else {
            AppType::YRAL
        }
    }

    pub fn select() -> Self {
        #[cfg(feature = "hydrate")]
        {
            let hostname = leptos::prelude::window().location().hostname().unwrap_or_default();
            AppType::from_host(&hostname)
        }

        #[cfg(not(feature = "hydrate"))]
        {
            use utils::host::get_host;
            let host = get_host();
            AppType::from_host(&host)
        }
    }
}
