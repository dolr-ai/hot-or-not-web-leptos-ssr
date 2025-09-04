use candid::Principal;
use codee::string::FromToStringCodec;
use component::spinner::FullScreenSpinner;
use consts::NSFW_ENABLED_COOKIE;
use global_constants::{DEFAULT_BET_COIN_FOR_LOGGED_IN, DEFAULT_BET_COIN_FOR_LOGGED_OUT};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_query_map;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use state::canisters::AuthState;
use utils::host::show_nsfw_content;
use utils::ml_feed::{get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw};
use utils::try_or_redirect_opt;
use yral_types::post::PostItemV3;

use crate::post_view::PostViewWithUpdatesMLFeed;

#[server]
async fn get_top_post_id_global_clean_feed() -> Result<Option<PostItemV3>, ServerFnError> {
    use utils::client_ip::get_client_ip;

    let ip_address = get_client_ip().await;

    let posts = get_ml_feed_coldstart_clean(Principal::anonymous(), 1, vec![], ip_address)
        .await
        .map_err(|e| {
            log::error!("Error getting top post id global clean feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;
    if !posts.is_empty() {
        return Ok(Some(posts[0].clone()));
    }

    Ok(None)
}

#[server]
async fn get_top_post_id_global_nsfw_feed() -> Result<Option<PostItemV3>, ServerFnError> {
    use utils::client_ip::get_client_ip;

    let ip_address = get_client_ip().await;

    let posts = get_ml_feed_coldstart_nsfw(Principal::anonymous(), 1, vec![], ip_address)
        .await
        .map_err(|e| {
            log::error!("Error getting top post id global nsfw feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;
    if !posts.is_empty() {
        return Ok(Some(posts[0].clone()));
    }

    Ok(None)
}

#[server]
async fn get_top_post_ids_global_clean_feed() -> Result<Vec<PostItemV3>, ServerFnError> {
    use utils::client_ip::get_client_ip;

    let ip_address = get_client_ip().await;

    let posts = get_ml_feed_coldstart_clean(Principal::anonymous(), 15, vec![], ip_address)
        .await
        .map_err(|e| {
            log::error!("Error getting top post id global clean feed: {e:?}");
            ServerFnError::new(e.to_string())
        })?;

    Ok(posts)
}

#[server]
async fn get_top_post_ids_global_nsfw_feed() -> Result<Vec<PostItemV3>, ServerFnError> {
    use utils::client_ip::get_client_ip;

    let ip_address = get_client_ip().await;

    let posts = get_ml_feed_coldstart_nsfw(Principal::anonymous(), 15, vec![], ip_address)
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

    provide_context(RwSignal::new(
        // TODO: check auth state
        if false {
            DEFAULT_BET_COIN_FOR_LOGGED_IN
        } else {
            DEFAULT_BET_COIN_FOR_LOGGED_OUT
        },
    ));

    // let full_info = Resource::new_blocking(params, move |params_map| async move {
    //     // Check query param first, then cookie, then show_nsfw_content
    //     let nsfw_from_query = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
    //     let nsfw_enabled = nsfw_from_query
    //         || nsfw_cookie_enabled.get_untracked().unwrap_or(false)
    //         || show_nsfw_content();
    //     leptos::logging::log!(
    //         "NSFW enabled: {nsfw_enabled} (query: {nsfw_from_query}, cookie: {:?})",
    //         nsfw_cookie_enabled.get_untracked()
    //     );
    //     let post = if nsfw_enabled {
    //         get_top_post_id_global_nsfw_feed().await
    //     } else {
    //         get_top_post_id_global_clean_feed().await
    //     }?;

    //     let utm = params_map.to_query_string();
    //     let utm = if utm.contains("utm") {
    //         Some(utm.replace("?", ""))
    //     } else {
    //         None
    //     };
    //     let user_refer = params_map.get("user_refer").map(|s| s.to_string());
    //     Ok::<_, ServerFnError>((post, utm, user_refer))
    // });

    // load videos from cache with num_results=10
    // for each video, load details in batch of N
    // provide context for coninstate
    // hand over to mlfeed whatever component as initial_posts

    // once done, move loading to frontend in a client side suspense
    // bring back event signals, discuss with bhavna if any is changing is changing

    let initial_posts = Resource::new_blocking(params, move |params_map| {
        async move {
            // Check query param first, then cookie, then show_nsfw_content
            let nsfw_from_query = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
            let nsfw_enabled = nsfw_from_query
                || nsfw_cookie_enabled.get_untracked().unwrap_or(false)
                || show_nsfw_content();
            leptos::logging::log!(
                "NSFW enabled: {nsfw_enabled} (query: {nsfw_from_query}, cookie: {:?})",
                nsfw_cookie_enabled.get_untracked()
            );

            leptos::logging::debug_warn!("loading posts from cache");
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

    let auth_state = AuthState::default();
    provide_context(auth_state);

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
