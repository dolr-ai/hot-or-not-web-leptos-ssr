use candid::Principal;
use codee::string::FromToStringCodec;
use component::spinner::FullScreenSpinner;
use consts::NSFW_ENABLED_COOKIE;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_query_map;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use utils::host::show_nsfw_content;
use utils::ml_feed::{get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw};
use utils::try_or_redirect_opt;
use yral_types::post::PostItemV3;

use crate::post_view::{PostViewCtx, PostViewWithUpdatesMLFeed};

#[server]
async fn get_top_post_ids_global_clean_feed() -> Result<Vec<PostItemV3>, ServerFnError> {
    let posts = get_ml_feed_coldstart_clean(Principal::anonymous(), 15, vec![], None)
        .await
        .map_err(|e| {
            log::error!("Error getting top post id global clean feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;

    Ok(posts)
}

#[server]
async fn get_top_post_ids_global_nsfw_feed() -> Result<Vec<PostItemV3>, ServerFnError> {
    let posts = get_ml_feed_coldstart_nsfw(Principal::anonymous(), 15, vec![], None)
        .await
        .map_err(|e| {
            log::error!("Error getting top post id global nsfw feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;

    Ok(posts)
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
            leptos::logging::debug_warn!(
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
