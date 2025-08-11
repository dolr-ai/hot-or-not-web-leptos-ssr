use candid::Principal;
use component::spinner::FullScreenSpinner;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_query_map;
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

    // This will never get trigerred...
    // efects only run on client (by default)
    // TODO: figure out the proper logic to trigger this
    // Effect::new(move |_| {
    //     let params_map = params.get();
    //     let utm_source = params_map
    //         .get("utm_source")
    //         .unwrap_or("external".to_string());

    //     let (_, set_is_internal_user, _) =
    //         use_local_storage::<bool, FromToStringCodec>(USER_INTERNAL_STORE);
    //     if utm_source == "internal" {
    //         set_is_internal_user(true);
    //     } else if utm_source == "internaloff" {
    //         set_is_internal_user(false);
    //     }
    // });

    let full_info = Resource::new_blocking(params, move |params_map| async move {
        let nsfw_enabled = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
        let post = if nsfw_enabled || show_nsfw_content() {
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
