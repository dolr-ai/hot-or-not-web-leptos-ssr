use candid::Principal;
use reqwest::Client;
use reqwest::Url;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;

use consts::DOWNLOAD_UPLOAD_SERVICE;
use yral_types::delegated_identity::DelegatedIdentityWire;

#[derive(Deserialize)]
pub struct AllowPrincpalRes {
    allowed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UploadContentPayload {
    url: String,
    payload: DelegatedIdentityWire,
}

#[derive(Deserialize)]
pub struct ServerError {
    error: String,
}

#[derive(Clone, Debug)]
pub struct ContentSeedClient {
    client: Client,
    base_url: Url,
}

impl Default for ContentSeedClient {
    fn default() -> Self {
        Self {
            client: Default::default(),
            base_url: DOWNLOAD_UPLOAD_SERVICE.clone(),
        }
    }
}

impl ContentSeedClient {
    pub fn with_base_url(url: Url) -> Self {
        ContentSeedClient {
            client: Default::default(),
            base_url: url,
        }
    }

    pub async fn check_if_authorized(&self, principal: Principal) -> Result<bool, Box<dyn Error>> {
        let api_url = self
            .base_url
            .join("allowed/")
            .expect("url error")
            .join(&principal.to_string())
            .expect("url error");
        let res = self.client.get(api_url).send().await?;
        let res_json: AllowPrincpalRes = res.json().await?;
        Ok(res_json.allowed)
    }

    pub async fn upload_content(
        &self,
        url: String,
        identity: DelegatedIdentityWire,
    ) -> Result<(), Box<dyn Error>> {
        let api_url = self
            .base_url
            .join("upload-leptos/")
            .expect("url join error");
        let req_body = UploadContentPayload {
            url,
            payload: identity,
        };
        let res = self.client.post(api_url).json(&req_body).send().await?;
        if res.status().is_success() {
            return Ok(());
        }
        let ServerError { error } = res.json().await?;
        Err(error.into())
    }
}
