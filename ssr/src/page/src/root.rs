use candid::Principal;
use codee::string::FromToStringCodec;
use component::spinner::FullScreenSpinner;
use consts::NSFW_ENABLED_COOKIE;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_query_map;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use utils::host::show_nsfw_content;
use utils::ml_feed::{
    get_ml_feed_clean, get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw, get_ml_feed_nsfw,
};
use utils::try_or_redirect_opt;
use yral_types::post::PostItemV3;

use crate::post_view::{PostViewCtx, PostViewWithUpdatesMLFeed};

fn generate_random_principal() -> Principal {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current timestamp as source of randomness
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    // Add some additional variation
    let thread_id = std::thread::current().id();
    let thread_hash = format!("{thread_id:?}")
        .chars()
        .map(|c| c as u64)
        .fold(0u64, |acc, x| acc.wrapping_add(x));

    // Combine timestamp with thread hash for uniqueness
    let unique_id = timestamp.wrapping_mul(1000000).wrapping_add(thread_hash);

    // Create principal from bytes
    let mut bytes = vec![0u8; 29];
    bytes[0] = (unique_id & 0xFF) as u8;
    bytes[1] = ((unique_id >> 8) & 0xFF) as u8;
    bytes[2] = ((unique_id >> 16) & 0xFF) as u8;
    bytes[3] = ((unique_id >> 24) & 0xFF) as u8;
    bytes[4] = ((unique_id >> 32) & 0xFF) as u8;
    bytes[5] = ((unique_id >> 40) & 0xFF) as u8;
    bytes[6] = ((unique_id >> 48) & 0xFF) as u8;
    bytes[7] = ((unique_id >> 56) & 0xFF) as u8;

    Principal::from_slice(&bytes)
}

#[server]
#[tracing::instrument]
async fn get_top_post_ids_global_clean_feed() -> Result<Vec<PostItemV3>, ServerFnError> {
    let random_principal = generate_random_principal();
    let posts = get_ml_feed_coldstart_clean(random_principal, 15, vec![], None)
        .await
        .map_err(|e| {
            leptos::logging::error!("Error getting top post id global clean feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;

    if posts.is_empty() {
        leptos::logging::error!(
            "Coldstart clean feed returned 0 results, falling back to ML feed with retries"
        );

        const MAX_RETRIES: usize = 5;
        for attempt in 1..=MAX_RETRIES {
            let fallback_principal = generate_random_principal();
            leptos::logging::log!("Attempting ML feed clean fallback - attempt {attempt}/{MAX_RETRIES} with principal: {fallback_principal:?}");

            let retry_posts = get_ml_feed_clean(fallback_principal, 15, vec![], None)
                .await
                .map_err(|e| {
                    leptos::logging::error!(
                        "Error getting ML feed clean fallback on attempt {attempt}: {e:?}"
                    );
                    ServerFnError::new(e.to_string())
                })?;

            if !retry_posts.is_empty() {
                leptos::logging::log!(
                    "ML feed clean fallback succeeded on attempt {attempt} with {} posts",
                    retry_posts.len()
                );
                return Ok(retry_posts);
            }

            leptos::logging::warn!("ML feed clean fallback attempt {attempt} returned 0 results");
        }

        leptos::logging::error!(
            "All {MAX_RETRIES} ML feed clean fallback attempts returned 0 results"
        );
        Ok(vec![])
    } else {
        Ok(posts)
    }
}

#[server]
#[tracing::instrument]
async fn get_top_post_ids_global_nsfw_feed() -> Result<Vec<PostItemV3>, ServerFnError> {
    let random_principal = generate_random_principal();
    let posts = get_ml_feed_coldstart_nsfw(random_principal, 15, vec![], None)
        .await
        .map_err(|e| {
            leptos::logging::error!("Error getting top post id global nsfw feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;

    if posts.is_empty() {
        leptos::logging::error!(
            "Coldstart nsfw feed returned 0 results, falling back to ML feed with retries"
        );

        const MAX_RETRIES: usize = 5;
        for attempt in 1..=MAX_RETRIES {
            let fallback_principal = generate_random_principal();
            leptos::logging::log!("Attempting ML feed nsfw fallback - attempt {attempt}/{MAX_RETRIES} with principal: {fallback_principal:?}");

            let retry_posts = get_ml_feed_nsfw(fallback_principal, 15, vec![], None)
                .await
                .map_err(|e| {
                    leptos::logging::error!(
                        "Error getting ML feed nsfw fallback on attempt {attempt}: {e:?}"
                    );
                    ServerFnError::new(e.to_string())
                })?;

            if !retry_posts.is_empty() {
                leptos::logging::log!(
                    "ML feed nsfw fallback succeeded on attempt {attempt} with {} posts",
                    retry_posts.len()
                );
                return Ok(retry_posts);
            }

            leptos::logging::warn!("ML feed nsfw fallback attempt {attempt} returned 0 results");
        }

        leptos::logging::error!(
            "All {MAX_RETRIES} ML feed nsfw fallback attempts returned 0 results"
        );
        Ok(vec![])
    } else {
        Ok(posts)
    }
}

#[component]
pub fn YralRootPage() -> impl IntoView {
    let params = use_query_map();

    let (nsfw_cookie_enabled, _) = use_cookie_with_options::<bool, FromToStringCodec>(
        NSFW_ENABLED_COOKIE,
        UseCookieOptions::default()
            .path("/")
            .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
            .same_site(leptos_use::SameSite::Lax),
    );

    let PostViewCtx { video_queue, .. } = expect_context();

    let initial_posts = Resource::new(params, move |params_map| {
        async move {
            // we already have videos and can therefore avoid loading more posts
            if video_queue.with_untracked(|q| !q.is_empty()) {
                return Ok(Default::default());
            }

            // Check query param first, then cookie, then show_nsfw_content
            let nsfw_from_query = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
            let nsfw_enabled = nsfw_from_query
                || nsfw_cookie_enabled.get_untracked().unwrap_or(false)
                || show_nsfw_content();
            leptos::logging::log!(
                "NSFW enabled: {nsfw_enabled} (query: {nsfw_from_query}, cookie: {:?})",
                nsfw_cookie_enabled.get_untracked()
            );

            let start = web_time::Instant::now();
            let posts = if nsfw_enabled {
                get_top_post_ids_global_nsfw_feed().await
            } else {
                get_top_post_ids_global_clean_feed().await
            }?;
            leptos::logging::log!(
                // TODO: switch to debug later
                "loaded {} posts from cache in {:?}",
                posts.len(),
                start.elapsed()
            );

            let posts = posts.into_iter().map(Into::into).collect();

            Ok::<_, ServerFnError>(posts)
        }
    });

    // Providing auth state is one of CtxProvider's job, but as a hack I am
    // setting it here because using CtxProvider here was causing this page to
    // be loaded twice

    view! {
        <Title text="YRAL - Home" />
        <Suspense fallback=FullScreenSpinner>
            {move || {
                Suspend::new(async move {
                    let initial_posts = try_or_redirect_opt!(initial_posts.await);

                    Some(view! {
                        <PostViewWithUpdatesMLFeed initial_posts />
                    }.into_any())
                })
            }}
        </Suspense>
    }
    .into_any()
}
