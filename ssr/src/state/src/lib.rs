#[cfg(feature = "backend-admin")]
pub mod admin_canisters;
pub mod app_state;
pub mod app_type;
pub mod audio_state;
pub mod auth;
pub mod canisters;
pub mod content_seed_client;
pub mod local_storage;

#[cfg(feature = "ssr")]
pub mod server {

    use auth::server_impl::store::KVStoreImpl;
    use utils::token::{icpump::ICPumpSearchGrpcChannel, nsfw::ICPumpNSFWGrpcChannel};

    use axum::extract::FromRef;
    use axum_extra::extract::cookie::Key;
    use leptos::prelude::*;
    use leptos_axum::AxumRouteListing;
    use yral_canisters_common::Canisters;

    #[derive(FromRef, Clone)]
    pub struct AppState {
        pub leptos_options: LeptosOptions,
        pub canisters: Canisters<false>,
        #[cfg(feature = "backend-admin")]
        pub admin_canisters: super::admin_canisters::AdminCanisters,
        #[cfg(feature = "cloudflare")]
        pub cloudflare: gob_cloudflare::CloudflareAuth,
        pub kv: KVStoreImpl,
        pub routes: Vec<AxumRouteListing>,
        pub cookie_key: Key,
        #[cfg(feature = "oauth-ssr")]
        pub google_oauth_clients: auth::core_clients::CoreClients,
        #[cfg(feature = "ga4")]
        pub grpc_offchain_channel: tonic::transport::Channel,
        #[cfg(feature = "firestore")]
        pub firestore_db: firestore::FirestoreDb,
        #[cfg(feature = "qstash")]
        pub qstash: utils::qstash::QStashClient,
        pub grpc_icpump_search_channel: ICPumpSearchGrpcChannel,
        pub grpc_nsfw_channel: ICPumpNSFWGrpcChannel,
    }
}
