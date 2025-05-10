use std::time::Duration;

use crate::canister_ids::USER_INDEX_ID;
use consts::{METADATA_API_BASE, YRAL_BACKEND_CONTAINER_TAG, YRAL_METADATA_CONTAINER_TAG};
use ic_agent::{identity::Secp256k1Identity, AgentError, Identity};
use k256::SecretKey;
use state::admin_canisters::AdminCanisters;
use testcontainers::{
    core::{ContainerPort, IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, Image, ImageExt,
};
use yral_metadata_client::MetadataClient;
use yral_metadata_types::UserMetadata;
use yral_testcontainers::{
    backend::{self, YralBackend, ADMIN_SECP_BYTES},
    metadata::{self, YralMetadata},
};

type MaybeContainer<I> = Option<ContainerAsync<I>>;

/// Holds all the containers that are started for local testing
/// we are required to hold these as ContainerAsync stops the container
/// on drop
#[derive(Default)]
pub struct TestContainers {
    redis: MaybeContainer<GenericImage>,
    metadata: MaybeContainer<YralMetadata>,
    backend: MaybeContainer<YralBackend>,
}

impl TestContainers {
    async fn start_image<I: Image>(image: I, port: impl Into<ContainerPort>) -> ContainerAsync<I> {
        let port = port.into();
        image
            .with_mapped_port(port.as_u16(), port)
            .start()
            .await
            .map_err(|e| format!("Failed to start container: {e}"))
            .unwrap()
    }
    pub async fn start_redis(&mut self) {
        let redis_im = GenericImage::new("redis", "7.2.4")
            .with_exposed_port(6379.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"));
        self.redis = Some(Self::start_image(redis_im, 6379.tcp()).await);
    }

    pub async fn start_metadata(&mut self) {
        let img = YralMetadata::new(YRAL_METADATA_CONTAINER_TAG.into());
        self.metadata = Some(Self::start_image(img, metadata::REST_PORT).await);

        // Setup User Principal -> User Canister ID
        // for the admin canister
        let metadata_client: MetadataClient<false> =
            MetadataClient::with_base_url(METADATA_API_BASE.clone());
        let sk = SecretKey::from_bytes(&ADMIN_SECP_BYTES.into()).unwrap();
        let id = Secp256k1Identity::from_private_key(sk);
        let cans = AdminCanisters::new(id.clone());

        let user_index = cans.user_index_with(USER_INDEX_ID).await;
        let admin_principal = id.sender().unwrap();
        let admin_canister = loop {
            let res = user_index
                .get_user_canister_id_from_user_principal_id(admin_principal)
                .await;
            match res {
                Ok(princ) => break princ.unwrap(),
                Err(AgentError::HttpError(_) | AgentError::CertificateOutdated(_)) => {
                    tokio::time::sleep(Duration::from_secs(8)).await;
                    continue;
                }
                Err(e) => panic!("Failed to get user canister {e}"),
            }
        };
        let metadata = UserMetadata {
            user_canister_id: admin_canister,
            user_name: "".into(),
        };

        metadata_client
            .set_user_metadata(&id, metadata)
            .await
            .unwrap()
    }

    pub async fn start_backend(&mut self) {
        let img = YralBackend::new(YRAL_BACKEND_CONTAINER_TAG.into());
        self.backend = Some(Self::start_image(img, backend::AGENT_PORT).await);
    }
}
