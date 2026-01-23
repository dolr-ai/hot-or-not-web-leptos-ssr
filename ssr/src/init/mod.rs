#[cfg(feature = "local-bin")]
pub mod containers;

use std::env;

use auth::server_impl::store::KVStoreImpl;
use axum_extra::extract::cookie::Key;
use leptos::prelude::*;
use leptos_axum::AxumRouteListing;
use state::server::AppState;
use yral_canisters_common::Canisters;

#[cfg(feature = "cloudflare")]
fn init_cf() -> gob_cloudflare::CloudflareAuth {
    use gob_cloudflare::{CloudflareAuth, Credentials};
    let creds = Credentials {
        token: env::var("CF_TOKEN").expect("`CF_TOKEN` is required!"),
        account_id: env::var("CF_ACCOUNT_ID").expect("`CF_ACCOUNT_ID` is required!"),
    };
    CloudflareAuth::new(creds)
}

fn init_cookie_key() -> Key {
    let cookie_key_raw = {
        #[cfg(not(feature = "local-bin"))]
        {
            let cookie_key_str = env::var("COOKIE_KEY").expect("`COOKIE_KEY` is required!");
            hex::decode(cookie_key_str).expect("Invalid `COOKIE_KEY` (must be length 128 hex)")
        }
        #[cfg(feature = "local-bin")]
        {
            use rand_chacha::rand_core::{OsRng, RngCore};
            let mut cookie_key = [0u8; 64];
            OsRng.fill_bytes(&mut cookie_key);
            cookie_key.to_vec()
        }
    };
    Key::from(&cookie_key_raw)
}

#[cfg(feature = "oauth-ssr")]
fn init_yral_oauth() -> auth::server_impl::yral::YralOAuthClient {
    use auth::server_impl::yral::YralOAuthClient;
    use consts::yral_auth::{
        YRAL_AUTH_AUTHORIZATION_URL, YRAL_AUTH_CLIENT_ID_ENV, YRAL_AUTH_ISSUER_URL,
        YRAL_AUTH_TOKEN_URL,
    };
    use openidconnect::{AuthType, AuthUrl, TokenUrl};
    use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl};

    let client_id = env::var(YRAL_AUTH_CLIENT_ID_ENV)
        .unwrap_or_else(|_| panic!("`{YRAL_AUTH_CLIENT_ID_ENV}` is required!"));
    let client_secret =
        env::var("YRAL_AUTH_CLIENT_SECRET").expect("`YRAL_AUTH_CLIENT_SECRET` is required!");
    let redirect_uri =
        env::var("YRAL_AUTH_REDIRECT_URL").expect("`YRAL_AUTH_REDIRECT_URL` is required!");

    YralOAuthClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        IssuerUrl::new(YRAL_AUTH_ISSUER_URL.to_string()).unwrap(),
        AuthUrl::new(YRAL_AUTH_AUTHORIZATION_URL.to_string()).unwrap(),
        Some(TokenUrl::new(YRAL_AUTH_TOKEN_URL.to_string()).unwrap()),
        None,
        Default::default(),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap())
    .set_auth_type(AuthType::RequestBody)
}

#[cfg(feature = "oauth-ssr")]
fn init_yral_auth_migration_key() -> jsonwebtoken::EncodingKey {
    let raw_pem = env::var("YRAL_AUTH_MIGRATION_ES256_PEM")
        .expect("`YRAL_AUTH_MIGRATION_ES256_PEM` is required!");
    let enc_key = jsonwebtoken::EncodingKey::from_ec_pem(raw_pem.as_bytes())
        .expect("Invalid `YRAL_AUTH_MIGRATION_ES256_PEM`");

    enc_key
}

#[cfg(feature = "ga4")]
async fn init_grpc_offchain_channel() -> tonic::transport::Channel {
    use consts::OFF_CHAIN_AGENT_GRPC_URL;
    use tonic::transport::{Channel, ClientTlsConfig};

    let tls_config = ClientTlsConfig::new().with_webpki_roots();
    let off_chain_agent_url = OFF_CHAIN_AGENT_GRPC_URL.as_ref();
    Channel::from_static(off_chain_agent_url)
        .tls_config(tls_config)
        .expect("Couldn't update TLS config for off-chain agent")
        .connect()
        .await
        .expect("Couldn't connect to off-chain agent")
}

#[cfg(feature = "backend-admin")]
fn init_admin_canisters() -> state::admin_canisters::AdminCanisters {
    use state::admin_canisters::AdminCanisters;

    #[cfg(feature = "local-bin")]
    {
        use ic_agent::identity::Secp256k1Identity;
        use k256::SecretKey;
        use yral_testcontainers::backend::ADMIN_SECP_BYTES;

        let sk = SecretKey::from_bytes(&ADMIN_SECP_BYTES.into()).unwrap();
        let identity = Secp256k1Identity::from_private_key(sk);
        AdminCanisters::new(identity)
    }

    #[cfg(not(feature = "local-bin"))]
    {
        use ic_agent::identity::Secp256k1Identity;

        let admin_id_pem =
            env::var("BACKEND_ADMIN_IDENTITY").expect("`BACKEND_ADMIN_IDENTITY` is required!");
        let admin_id_pem_by = admin_id_pem.as_bytes();
        let admin_id =
            Secp256k1Identity::from_pem(admin_id_pem_by).expect("Invalid `BACKEND_ADMIN_IDENTITY`");
        AdminCanisters::new(admin_id)
    }
}

#[cfg(feature = "qstash")]
fn init_qstash_client() -> utils::qstash::QStashClient {
    use utils::qstash::QStashClient;

    let auth_token = env::var("QSTASH_TOKEN").expect("`QSTASH_TOKEN` is required!");

    QStashClient::new(&auth_token)
}

#[cfg(feature = "alloydb")]
async fn init_alloydb_client() -> state::alloydb::AlloyDbInstance {
    use google_cloud_alloydb_v1::client::AlloyDBAdmin;
    use google_cloud_auth::credentials::service_account::Builder as CredBuilder;
    use state::alloydb::AlloyDbInstance;

    let sa_json_raw = env::var("ALLOYDB_SERVICE_ACCOUNT_JSON")
        .expect("`ALLOYDB_SERVICE_ACCOUNT_JSON` is required!");
    let sa_json: serde_json::Value =
        serde_json::from_str(&sa_json_raw).expect("Invalid `ALLOYDB_SERVICE_ACCOUNT_JSON`");
    let credentials = CredBuilder::new(sa_json)
        .build()
        .expect("Invalid `ALLOYDB_SERVICE_ACCOUNT_JSON`");

    let client = AlloyDBAdmin::builder()
        .with_credentials(credentials)
        .build()
        .await
        .expect("Failed to create AlloyDB client");

    let instance = env::var("ALLOYDB_INSTANCE").expect("`ALLOYDB_INSTANCE` is required!");
    let db_name = env::var("ALLOYDB_DB_NAME").expect("`ALLOYDB_DB_NAME` is required!");
    let db_user = env::var("ALLOYDB_DB_USER").expect("`ALLOYDB_DB_USER` is required!");
    let db_password = env::var("ALLOYDB_DB_PASSWORD").expect("`ALLOYDB_DB_PASSWORD` is required!");

    AlloyDbInstance::new(client, instance, db_name, db_user, db_password)
}

pub struct AppStateRes {
    pub app_state: AppState,
    #[cfg(feature = "local-bin")]
    pub containers: containers::TestContainers,
}

pub struct AppStateBuilder {
    leptos_options: LeptosOptions,
    routes: Vec<AxumRouteListing>,
    #[cfg(feature = "local-bin")]
    containers: containers::TestContainers,
}

impl AppStateBuilder {
    pub fn new(leptos_options: LeptosOptions, routes: Vec<AxumRouteListing>) -> Self {
        Self {
            leptos_options,
            routes,
            #[cfg(feature = "local-bin")]
            containers: containers::TestContainers::default(),
        }
    }

    async fn init_kv(&mut self) -> KVStoreImpl {
        #[cfg(feature = "redis-kv")]
        {
            use auth::server_impl::store::redis_kv::RedisKV;
            let redis_url: String;
            #[cfg(feature = "local-bin")]
            {
                self.containers.start_redis().await;
                redis_url = "redis://127.0.0.1:6379".to_string();
            }
            #[cfg(not(feature = "local-bin"))]
            {
                redis_url = env::var("REDIS_URL").expect("`REDIS_URL` is required!");
            }
            KVStoreImpl::Redis(RedisKV::new(&redis_url).await.unwrap())
        }

        #[cfg(not(feature = "redis-kv"))]
        {
            use auth::server_impl::store::redb_kv::ReDBKV;
            KVStoreImpl::ReDB(ReDBKV::new().expect("Failed to initialize ReDB"))
        }
    }

    pub async fn build(mut self) -> AppStateRes {
        let kv = self.init_kv().await;
        #[cfg(feature = "local-bin")]
        {
            self.containers.start_backend().await;
            self.containers.start_metadata().await;
        }

        let app_state = AppState {
            leptos_options: self.leptos_options,
            canisters: Canisters::default(),
            routes: self.routes,
            #[cfg(feature = "backend-admin")]
            admin_canisters: init_admin_canisters(),
            #[cfg(feature = "cloudflare")]
            cloudflare: init_cf(),
            kv,
            cookie_key: init_cookie_key(),
            #[cfg(feature = "oauth-ssr")]
            yral_oauth_client: init_yral_oauth(),
            #[cfg(feature = "oauth-ssr")]
            yral_auth_migration_key: init_yral_auth_migration_key(),
            #[cfg(feature = "ga4")]
            grpc_offchain_channel: init_grpc_offchain_channel().await,
            #[cfg(feature = "qstash")]
            qstash: init_qstash_client(),
            #[cfg(feature = "alloydb")]
            alloydb: init_alloydb_client().await,
            #[cfg(any(feature = "alloydb", feature = "sats-airdrop"))]
            hon_worker_jwt: {
                use state::server::HonWorkerJwt;
                let jwt = env::var("HON_WORKER_JWT").expect("`HON_WORKER_JWT` is required!");

                HonWorkerJwt(std::sync::Arc::new(jwt))
            },
            // #[cfg(feature = "dolr-airdrop")]
            // dolr_airdrop_db: {
            //     let url = env::var("DOLR_AIRDROP_NEON_DB_URL")
            //         .expect("`DOLR_AIRDROP_NEON_DB_URL` is required!");

            //     dolr_airdrop::db::DolrAirdrop::connect_and_migrate(url)
            //         .await
            //         .expect("connect to neon postgres")
            // },
            // #[cfg(feature = "sats-airdrop")]
            // sats_airdrop_db: {
            //     let url = env::var("SATS_AIRDROP_NEON_DB_URL")
            //         .expect("`SATS_AIRDROP_NEON_DB_URL` is required!");

            //     sats_airdrop::db::SatsAirdrop::connect_and_migrate(url)
            //         .await
            //         .expect("connect to neon postgres")
            // },
        };

        AppStateRes {
            app_state,
            #[cfg(feature = "local-bin")]
            containers: self.containers,
        }
    }
}
