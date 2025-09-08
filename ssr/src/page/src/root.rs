use candid::Principal;
use codee::string::FromToStringCodec;
use component::leaderboard::api::fetch_user_rank_from_api;
use component::leaderboard::{RankUpdateCounter, UserRank};
use component::spinner::FullScreenSpinner;
use consts::NSFW_ENABLED_COOKIE;
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

    let initial_posts = Resource::new(params, move |params_map| {
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

    let rank_update_count = use_context::<RwSignal<RankUpdateCounter>>()
        .expect("RankUpdateCounter should be provided globally");
    let global_rank =
        use_context::<RwSignal<UserRank>>().expect("UserRank should be provided globally");

    let global_rank_resource = auth_state.derive_resource(
        move || rank_update_count.get().0,
        move |cans, counter| {
            let global_rank = global_rank;
            async move {
                // If we already have a rank and counter is 0, return cached value
                if counter == 0 {
                    let cached = global_rank.get_untracked();
                    if cached.rank.is_some() {
                        return Ok(cached);
                    }
                }

                // Get user principal from canisters
                let principal = cans.user_principal();

                leptos::logging::log!(
                    "PostView: Fetching rank for principal: {} (counter: {})",
                    principal,
                    counter
                );

                // Fetch rank and tournament status from API
                match fetch_user_rank_from_api(principal).await {
                    Ok(Some((rank, status))) => {
                        leptos::logging::log!(
                            "PostView: Fetched rank: {}, status: {}",
                            rank,
                            status
                        );
                        // Update global rank value
                        let user_rank = UserRank {
                            rank: Some(rank),
                            tournament_status: Some(status),
                        };
                        global_rank.set(user_rank.clone());
                        Ok(user_rank)
                    }
                    Ok(None) => {
                        leptos::logging::log!("PostView: No rank found for user");
                        Ok(UserRank {
                            rank: None,
                            tournament_status: None,
                        })
                    }
                    Err(e) => {
                        leptos::logging::error!("PostView: Failed to fetch user rank: {}", e);
                        Ok(UserRank {
                            rank: None,
                            tournament_status: None,
                        })
                    }
                }
            }
        },
    );
    provide_context(global_rank_resource);
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
