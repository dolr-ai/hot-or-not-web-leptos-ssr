use std::sync::Arc;

use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderMap, HeaderValue,
};
use reqwest::{Client, Url};
use serde_json::Value;
use yral_qstash_types::{ClaimTokensRequest, ParticipateInSwapRequest};

use consts::{
    ANALYTICS_SERVER_URL, CDAO_SWAP_PRE_READY_TIME_SECS, CDAO_SWAP_TIME_SECS, OFF_CHAIN_AGENT_URL,
};

#[derive(Clone, Debug)]
pub struct QStashClient {
    client: Client,
    base_url: Arc<Url>,
}

impl QStashClient {
    pub fn new(auth_token: &str) -> Self {
        let mut bearer: HeaderValue = format!("Bearer {auth_token}")
            .parse()
            .expect("Invalid QStash auth token");
        bearer.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, bearer);

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create QStash client");
        let base_url = Url::parse("https://qstash.upstash.io/v2/").unwrap();

        Self {
            client,
            base_url: Arc::new(base_url),
        }
    }

    pub async fn enqueue_claim_token(&self, req: ClaimTokensRequest) -> Result<(), reqwest::Error> {
        let off_chain_ep = OFF_CHAIN_AGENT_URL.join("qstash/claim_tokens").unwrap();

        let path = format!("publish/{off_chain_ep}");
        let ep = self.base_url.join(&path).unwrap();

        self.client
            .post(ep)
            .json(&req)
            .header(CONTENT_TYPE, "application/json")
            .header("upstash-method", "POST")
            .header("upstash-delay", format!("{CDAO_SWAP_TIME_SECS}s"))
            .send()
            .await?;
        Ok(())
    }

    pub async fn enqueue_participate_in_swap(
        &self,
        req: ParticipateInSwapRequest,
    ) -> Result<(), reqwest::Error> {
        let off_chain_ep = OFF_CHAIN_AGENT_URL
            .join("qstash/participate_in_swap")
            .unwrap();
        let path = format!("publish/{off_chain_ep}");
        let ep = self.base_url.join(&path).unwrap();

        self.client
            .post(ep)
            .json(&req)
            .header(CONTENT_TYPE, "application/json")
            .header("upstash-method", "POST")
            .header("upstash-delay", format!("{CDAO_SWAP_PRE_READY_TIME_SECS}s"))
            .send()
            .await?;
        Ok(())
    }

    pub async fn send_analytics_event_to_qstash(
        &self,
        req: Value,
        token: String,
    ) -> Result<(), reqwest::Error> {
        let off_chain_ep = ANALYTICS_SERVER_URL.join("api/send_event").unwrap();
        let path = format!("publish/{off_chain_ep}");
        let ep = self.base_url.join(&path).unwrap();

        self.client
            .post(ep)
            .json(&req)
            .header(CONTENT_TYPE, "application/json")
            .header("upstash-method", "POST")
            .header("Upstash-Forward-Authorization", format!("Bearer {token}"))
            .send()
            .await?;
        Ok(())
    }
}
