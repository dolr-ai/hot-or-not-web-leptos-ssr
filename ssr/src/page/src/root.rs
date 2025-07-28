use crate::post_view::PostDetailsCacheCtx;
use candid::Principal;
use codee::string::FromToStringCodec;
use component::spinner::FullScreenSpinner;
use consts::USER_INTERNAL_STORE;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_query_map;
use leptos_use::storage::use_local_storage;
use utils::host::show_nsfw_content;
use utils::ml_feed::{get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw};
use yral_types::post::PostItemV2;

#[server]
async fn get_top_post_id_global_clean_feed() -> Result<Option<PostItemV2>, ServerFnError> {
    let posts = get_ml_feed_coldstart_clean(Principal::anonymous(), 1, vec![])
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
    let posts = get_ml_feed_coldstart_nsfw(Principal::anonymous(), 1, vec![])
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

    Effect::new(move |_| {
        let params_map = params.get();
        let utm_source = params_map
            .get("utm_source")
            .unwrap_or("external".to_string());

        let (_, set_is_internal_user, _) =
            use_local_storage::<bool, FromToStringCodec>(USER_INTERNAL_STORE);
        if utm_source == "internal" {
            set_is_internal_user(true);
        } else if utm_source == "internaloff" {
            set_is_internal_user(false);
        }
    });

    let target_post = Resource::new_blocking(params, move |params_map| async move {
        let nsfw_enabled = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
        if nsfw_enabled || show_nsfw_content() {
            get_top_post_id_global_nsfw_feed().await
        } else {
            get_top_post_id_global_clean_feed().await
        }
    });

    let store_utms = Resource::new_blocking(
        || (),
        move |_| async move {
            let utm = params.get_untracked().to_query_string();
            if utm.contains("utm") {
                Some(utm.replace("?", ""))
            } else {
                None
            }
        },
    );

    let post_details_cache: PostDetailsCacheCtx = expect_context();

    view! {
        <Title text="YRAL - Home" />
        <Suspense fallback=FullScreenSpinner>
            {move || {
                let user_refer = params.get().get("user_refer").map(|s| s.to_string());
                let fresh = params.get().get("fresh").map(|s| s == "true").unwrap_or(false);
                Suspend::new(async move {
                    let utms = store_utms.await;
                    let mut url = match target_post.await {
                        Ok(Some(post_item)) => {
                            let publisher_user_id = post_item.publisher_user_id.clone();
                            let canister_id = post_item.canister_id.clone();
                            let post_id = post_item.post_id;
                            post_details_cache
                                .post_details
                                .update(|post_details| {
                                    if fresh {
                                        post_details.clear();
                                    }
                                    post_details.insert((Principal::from_text(publisher_user_id).unwrap(), post_id), post_item.clone());
                                });
                            format!("/hot-or-not/{canister_id}/{post_id}")
                        }
                        Ok(None) => "/error?err=No Posts Found".to_string(),
                        Err(e) => format!("/error?err={e}"),
                    };
                    if let Some(user_refer) = user_refer {
                        url.push_str(&format!("?user_refer={user_refer}"));
                        if let Some(utms) = utms {
                            url.push_str(&format!("&{utms}"));
                        }
                    } else if let Some(utms) = utms {
                        url.push_str(&format!("?{utms}"));
                    }
                    view! { <Redirect path=url /> }
                })
            }}
        </Suspense>
    }
    .into_any()
}
