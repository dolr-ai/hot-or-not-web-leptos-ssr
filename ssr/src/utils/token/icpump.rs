use serde::{Deserialize, Serialize};

use futures::stream::BoxStream;
use futures::StreamExt;

use leptos::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenListItemFS {
    pub user_id: String,
    pub name: String,
    pub token_name: String,
    pub token_symbol: String,
    pub logo: String,
    pub description: String,
    pub created_at: String,
    #[serde(default)]
    pub link: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenListItem {
    pub user_id: String,
    pub name: String,
    pub token_name: String,
    pub token_symbol: String,
    pub logo: String,
    pub description: String,
    pub created_at: String,
    pub formatted_created_at: String,
    pub link: String,
}

#[server]
pub async fn get_paginated_token_list(page: u32) -> Result<Vec<TokenListItem>, ServerFnError> {
    #[cfg(feature = "firestore")]
    {
        use firestore::*;
        use speedate::DateTime;

        use crate::consts::ICPUMP_LISTING_PAGE_SIZE;

        let firestore_db: firestore::FirestoreDb = expect_context();

        const TEST_COLLECTION_NAME: &str = "tokens-list"; //"test-tokens-3"

        let object_stream: BoxStream<TokenListItemFS> = firestore_db
            .fluent()
            .select()
            .from(TEST_COLLECTION_NAME)
            .order_by([(
                path!(TokenListItem::created_at),
                FirestoreQueryDirection::Descending,
            )])
            .offset((page - 1) * ICPUMP_LISTING_PAGE_SIZE as u32)
            .limit(ICPUMP_LISTING_PAGE_SIZE as u32)
            .obj()
            .stream_query()
            .await
            .expect("failed to stream");

        let as_vec: Vec<TokenListItemFS> = object_stream.collect().await;

        let res_vec: Vec<TokenListItem> = as_vec
            .iter()
            .map(|item| {
                let created_at_str = item.created_at.clone();
                let created_at = DateTime::parse_str(&created_at_str).unwrap().timestamp();
                let now = DateTime::now(0).unwrap().timestamp();
                let elapsed = now - created_at;

                let elapsed_str = if elapsed < 60 {
                    format!("{}s ago", elapsed)
                } else if elapsed < 3600 {
                    format!("{}m ago", elapsed / 60)
                } else if elapsed < 86400 {
                    format!("{}h ago", elapsed / 3600)
                } else {
                    format!("{}d ago", elapsed / 86400)
                };

                TokenListItem {
                    user_id: item.user_id.clone(),
                    name: item.name.clone(),
                    token_name: item.token_name.clone(),
                    token_symbol: item.token_symbol.clone(),
                    logo: item.logo.clone(),
                    description: item.description.clone(),
                    created_at: item.created_at.clone(),
                    formatted_created_at: elapsed_str,
                    link: item.link.clone(),
                }
            })
            .collect();

        Ok(res_vec)
    }

    #[cfg(not(feature = "firestore"))]
    {
        Ok(vec![])
    }
}
