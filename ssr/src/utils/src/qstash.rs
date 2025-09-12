use std::sync::Arc;

use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderMap, HeaderValue,
};
use reqwest::{Client, Url};
use serde_json::Value;

use consts::ANALYTICS_SERVER_URL;

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

    pub async fn send_analytics_event_to_qstash(
        &self,
        req: Value,
        token: String,
    ) -> Result<(), reqwest::Error> {
        let off_chain_ep = ANALYTICS_SERVER_URL.join("api/send_event").unwrap();
        let path = format!("publish/{off_chain_ep}");
        let ep = self.base_url.join(&path).unwrap();

        let res = self
            .client
            .post(ep)
            .json(&req)
            .header(CONTENT_TYPE, "application/json")
            .header("upstash-method", "POST")
            .header("Upstash-Forward-Authorization", format!("Bearer {token}"))
            .header("Upstash-Retries", "0") // TODO: change to 1 on Mon 15th Sept 2025
            .header("Upstash-Flow-Control-Key", "MIXPANEL_ANALYTICS")
            .header("Upstash-Flow-Control-Value", "Parallelism=500")
            .send()
            .await?;
        if res.status() != 200 {
            let e = res.text().await?;
            log::error!("Error sending analytics to qstash: {e:?}");
        }
        Ok(())
    }
}
