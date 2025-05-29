use crate::post_view::PostDetailsCacheCtx;
use crate::pumpdump::PumpNDump;
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
use utils::{
    host::{show_cdao_page, show_pnd_page},
    ml_feed::{get_ml_feed_coldstart_clean, get_ml_feed_coldstart_nsfw},
};
use yral_types::post::PostItem;

#[server]
async fn get_top_post_id_global_clean_feed() -> Result<Option<PostItem>, ServerFnError> {
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
async fn get_top_post_id_global_nsfw_feed() -> Result<Option<PostItem>, ServerFnError> {
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
pub fn CreatorDaoRootPage() -> impl IntoView {
    view! {
        <Redirect path="/board".to_string() />
    }
}

#[component]
pub fn YralRootPage() -> impl IntoView {
    let params = use_query_map();

    // let home_page_event = Action::new_unsync(move |_: &()| {
    //     let auth = auth_state();
    //     let base = unauth_canisters();
    //     async move {
    //         let cans = match auth.auth_cans(base).await {
    //             Ok(c) => c,
    //             Err(e) => return Err(e.to_string()),
    //         };
    //         let global = MixpanelGlobalProps::try_get(&cans, false);
    //         MixPanelEvent::track_home_page_viewed(MixpanelHomePageViewedProps {
    //             user_id: global.user_id,
    //             visitor_id: global.visitor_id,
    //             is_logged_in: global.is_logged_in,
    //             canister_id: global.canister_id,
    //             is_nsfw_enabled: global.is_nsfw_enabled,
    //         });
    //         Ok(())
    //     }
    // });

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
        // home_page_event.dispatch(());
    });

    let target_post = Resource::new_blocking(params, move |params_map| async move {
        let nsfw_enabled = params_map.get("nsfw").map(|s| s == "true").unwrap_or(false);
        if nsfw_enabled || show_nsfw_content() {
            get_top_post_id_global_nsfw_feed().await
        } else {
            get_top_post_id_global_clean_feed().await
        }
    });
    let post_details_cache: PostDetailsCacheCtx = expect_context();

    // if let Some(global) = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx()) {
    //     MixPanelEvent::track_home_page_viewed(MixpanelHomePageViewedProps {
    //         user_id: global.user_id,
    //         visitor_id: global.visitor_id,
    //         is_logged_in: global.is_logged_in,
    //         canister_id: global.canister_id,
    //         is_nsfw_enabled: global.is_nsfw_enabled,
    //     });
    // }

    view! {
        <Title text="YRAL - Home" />
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let url = match target_post.await {
                    Ok(Some(post_item)) => {
                        let canister_id = post_item.canister_id;
                        let post_id = post_item.post_id;
                        post_details_cache.post_details.update(|post_details| {
                            post_details.insert((canister_id, post_id), post_item.clone());
                        });

                        format!("/hot-or-not/{canister_id}/{post_id}")
                    },
                    Ok(None) => "/error?err=No Posts Found".to_string(),
                    Err(e) => format!("/error?err={e}"),
                };
                view! { <Redirect path=url /> }
            })}
        </Suspense>
    }
    .into_any()
}

#[component]
pub fn RootPage() -> impl IntoView {
    if show_pnd_page() {
        view! { <PumpNDump /> }.into_any()
    } else if show_cdao_page() {
        view! { <CreatorDaoRootPage /> }.into_any()
    } else {
        view! { <YralRootPage /> }.into_any()
    }
}
