use candid::Principal;
use codee::string::FromToStringCodec;
use component::spinner::FullScreenSpinner;
use consts::NSFW_ENABLED_COOKIE;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_query_map;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use utils::host::show_nsfw_content;
use utils::ml_feed::{get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw};
use yral_types::post::PostItemV2;

#[server]
async fn get_top_post_id_global_clean_feed() -> Result<Option<PostItemV2>, ServerFnError> {
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
async fn get_top_post_id_global_nsfw_feed() -> Result<Option<PostItemV2>, ServerFnError> {
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

    let full_info = Resource::new_blocking(params, move |params_map| async move {
        // Check query param first, then cookie, then show_nsfw_content
        let nsfw_from_query = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
        let nsfw_enabled = nsfw_from_query
            || nsfw_cookie_enabled.get_untracked().unwrap_or(false)
            || show_nsfw_content();
        leptos::logging::log!(
            "NSFW enabled: {nsfw_enabled} (query: {nsfw_from_query}, cookie: {:?})",
            nsfw_cookie_enabled.get_untracked()
        );
        let post = if nsfw_enabled {
            get_top_post_id_global_nsfw_feed().await
        } else {
            get_top_post_id_global_clean_feed().await
        }?;

        let utm = params_map.to_query_string();
        let utm = if utm.contains("utm") {
            Some(utm.replace("?", ""))
        } else {
            None
        };
        let user_refer = params_map.get("user_refer").map(|s| s.to_string());
        Ok::<_, ServerFnError>((post, utm, user_refer))
    });

    view! {
        <Title text="YRAL - Home" />
        <Suspense fallback=FullScreenSpinner>
            {move || {
                Suspend::new(async move {
                    let url = match full_info.await {
                        Ok((Some(post_item), utms, user_refer)) => {
                            let canister_id = post_item.canister_id.clone();
                            let post_id = post_item.post_id;

                            let mut url = format!("/hot-or-not/{canister_id}/{post_id}");
                            if let Some(user_refer) = user_refer {
                                url.push_str(&format!("?user_refer={user_refer}"));
                                if let Some(utms) = utms {
                                    url.push_str(&format!("&{utms}"));
                                }
                            } else if let Some(utms) = utms {
                                url.push_str(&format!("?{utms}"));
                            }
                            url
                        },
                        Ok((None, _, _)) => "/error?err=No Posts Found".to_string(),
                        Err(e) => format!("/error?err={e}"),
                    };

                    view! { <Redirect path=url /> }
                })
            }}
        </Suspense>
    }
    .into_any()
}
