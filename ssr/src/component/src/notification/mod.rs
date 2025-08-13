use crate::infinite_scroller::InfiniteScroller;
use crate::notification::provider::{NotificationError, NotificationProvider};
use crate::overlay::{ShadowOverlay, ShowOverlay};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_use::use_media_query;
use state::canisters::unauth_canisters;
use state::{app_state::AppState, canisters::auth_state};
use utils::send_wrap;
use yral_canisters_client::ic::NOTIFICATION_STORE_ID;
use yral_canisters_client::notification_store::NotificationStore;
use yral_canisters_client::notification_store::{NotificationData, NotificationType};
pub mod provider;

#[component]
fn NotificationLoadingItem() -> impl IntoView {
    view! {
        <div class="w-full p-4 border-b border-neutral-800">
             <div class="flex items-center gap-3">
                 <div class="size-11 rounded-full bg-neutral-800 animate-pulse" />
                 <div class="flex flex-col gap-1.5 w-1/2">
                    <div class="w-1/2 h-3.5 rounded-full bg-neutral-700 animate-pulse" />
                    <div class="w-full h-3.5 rounded-full bg-neutral-800 animate-pulse" />
                 </div>
             </div>
        </div>
    }
}

#[component]
fn NotificationItem(notif: NotificationData, is_read: bool) -> impl IntoView {
    let (title, description) = match &notif.payload {
        NotificationType::VideoUpload(_v) => (
            "Video Uploaded Sucessfully!".to_string(),
            "Your video has been uploaded sucessfully".to_string(),
        ),
        NotificationType::Liked(v) => (
            "Someone Liked your video!".to_string(),
            format!("{} liked your video", v.by_user_principal),
        ),
    };

    let notif = StoredValue::new(notif.clone());

    let auth = auth_state();
    let href_icon = auth.derive_resource(
        move || notif,
        move |cans, notif| async move {
            let notif = notif.get_value();
            let path = match notif.payload.clone() {
                NotificationType::VideoUpload(v) => {
                    let icon = cans.profile_details().profile_pic_or_random();

                    (
                        format!("/hot-or-not/{}/{}", cans.user_canister(), v.video_uid),
                        icon,
                    )
                }
                NotificationType::Liked(v) => {
                    let user_details =
                        match send_wrap(cans.get_profile_details(v.by_user_principal.to_string()))
                            .await
                        {
                            Ok(Some(details)) => details,
                            Ok(None) => {
                                return Err(ServerFnError::new("No profile details found"));
                            }
                            Err(e) => {
                                return Err(ServerFnError::new(format!(
                                    "Failed to get profile details: {e}"
                                )));
                            }
                        };

                    (
                        format!("/hot-or-not/{}/{}", cans.user_canister(), v.post_id),
                        user_details.profile_pic_or_random(),
                    )
                }
            };
            Ok(path)
        },
    );

    view! {
        <Suspense fallback=NotificationLoadingItem>
            {move || {
                match href_icon.get() {
                    Some(Ok((href_value, icon))) => {
                        let href_value_clone = href_value.clone();

                        view! {
                            <div class="w-full p-4 border-b border-neutral-800">
                                <div class="flex items-start gap-3">
                                    <div class="relative mt-0.5">
                                        <Show when=move || is_read>
                                            <div class="size-2 rounded-full bg-pink-700 absolute -left-1 top-1/2 -translate-y-1/2 -translate-x-2 z-10"></div>
                                        </Show>
                                        <div class="size-11 rounded-full bg-neutral-800 overflow-hidden">
                                            <img src={icon.clone()} class="size-11 rounded-full object-cover" />
                                        </div>
                                    </div>
                                    <div class="flex flex-col gap-1 flex-1">
                                        <div class="text-neutral-50 font-semibold">
                                            {title.clone()}
                                        </div>
                                        <div class="text-neutral-500 text-sm line-clamp-2">
                                            {description.clone()}
                                        </div>
                                        <div class="flex items-center gap-2 pt-2">
                                            <NotificationTarget href=href_value_clone>View</NotificationTarget>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                    Some(Err(_)) => {
                        view! {
                            <Redirect path="/wallet" />
                        }.into_any()
                    }
                    None => {
                        view! {
                            <NotificationLoadingItem />
                        }.into_any()
                    }
                }
            }}
        </Suspense>
    }
}

// Will be required later
#[component]
pub fn NotificationItemStatus(status: String) -> impl IntoView {
    view! {
        <span class=format!("text-sm py-1 px-2 rounded-full font-medium capitalize {}", match status.clone().as_str() {
            "accepted" => "text-green-500 bg-green-950/80",
            "pending" => "text-yellow-500 bg-yellow-950/80",
            "rejected" => "text-red-500 bg-red-950/80",
            _ => "text-gray-500 bg-gray-950/80",
        })>
            {format!("Status: {status}")}
        </span>
    }
}

#[component]
fn NotificationTarget(
    children: Children,
    href: String,
    #[prop(optional)] secondary: bool,
) -> impl IntoView {
    view! {
        <a
         href=href
        class=format!("border px-5 py-2 font-semibold rounded-lg transition-all {}",
            if secondary {
                "border-neutral-700 text-neutral-700 hover:border-neutral-500 hover:text-neutral-500"
            } else {
                "border-pink-700 text-pink-700 hover:bg-pink-700 hover:text-white"
            })>
            {children()}
        </a>
    }
}

#[component]
pub fn NotificationPage(close: RwSignal<bool>) -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Notifications";

    let is_desktop = use_media_query("(min-width: 1024px)");

    // Lock body scroll when popup is open on mobile
    Effect::new(move || {
        if !is_desktop.get() && close.get() {
            if let Some(window) = leptos::web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(body) = document.body() {
                        let _ = body.style().set_property("overflow", "hidden");
                    }
                }
            }
        } else if let Some(window) = leptos::web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    let _ = body.style().remove_property("overflow");
                }
            }
        }
    });

    let auth = auth_state();

    let set_last_viewed = Action::new(move |()| async move {
        let cans = send_wrap(auth.auth_cans())
            .await
            .map_err(|e| NotificationError(e.to_string()))
            .unwrap();
        let agent = cans.authenticated_user().await.1;
        let client = NotificationStore(NOTIFICATION_STORE_ID, agent);
        send_wrap(client.set_notification_panel_viewed())
            .await
            .map_err(|e| NotificationError(e.to_string()))
            .unwrap();
    });

    on_cleanup(move || {
        set_last_viewed.dispatch(());
    });

    let get_last_viewed = StoredValue::new(Resource::new(
        move || (),
        move |()| async move {
            let cans = send_wrap(auth.auth_cans())
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            let agent = cans.authenticated_user().await.1;
            let client = NotificationStore(NOTIFICATION_STORE_ID, agent);

            let res = send_wrap(client.get_notification_panel_viewed())
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            Ok::<_, ServerFnError>(res.map(|m| m.secs_since_epoch))
        },
    ));
    view! {
        <Title text=page_title />
        {move || if is_desktop.get() {
            // Desktop: right-side panel with ShadowOverlay
            view! {
                <ShadowOverlay show=ShowOverlay::Closable(close)>
                    <div class="fixed top-0 right-0 w-[552px] h-full bg-neutral-900 flex flex-col shadow-2xl border-l border-neutral-800">
                        <div class="sticky top-0 z-10 w-full bg-neutral-900 px-4 py-6">
                            <div class="flex flex-row justify-between items-center">
                                <h2 class="text-2xl font-semibold text-white">Notifications</h2>
                                <button
                                    on:click=move |_| close.set(false)
                                    class="text-white hover:text-gray-300 transition-colors"
                                >
                                    <Icon icon=icondata::AiCloseOutlined attr:class="w-6 h-6" />
                                </button>
                            </div>
                        </div>
                        <div class="flex-1 min-h-0 overflow-y-auto">
                        <Suspense>
                        {move || {
                            let Some(Ok(Some(res))) = get_last_viewed.get_value().get() else{
                                return view!{
                                    <Redirect path="/"/>
                                }.into_any()
                            };
                            view!{
                                <NotificationInfiniteScroller last_viewed_time=res/>
                            }.into_any()
                        }}
                        </Suspense>
                        </div>
                    </div>
                </ShadowOverlay>
            }.into_any()
        } else {
            // Mobile: current layout
            view! {
                    <Show when=close>
                    <div class="fixed z-50 inset-0 w-screen min-h-screen h-full overflow-y-auto bg-black">
                        <div class="flex flex-col items-center pt-4 pb-12 text-white min-h-full">
                            <div class="w-full bg-black p-4">
                                <div class="flex items-center">
                                    <button
                                        on:click=move |_| close.set(false)
                                        class="mr-4"
                                    >
                                        <Icon icon=icondata::AiLeftOutlined attr:class="w-6 h-6" />
                                    </button>
                                    <span class="text-xl font-bold flex-1 text-center mr-10">Notifications</span>
                                </div>
                            </div>

                            <Suspense>
                            {move || {
                                let Some(Ok(res)) = get_last_viewed.get_value().get() else{
                                    return view!{
                                        <Redirect path="/"/>
                                    }.into_any()
                                };
                                view!{
                                    <NotificationInfiniteScroller last_viewed_time=res.unwrap_or(web_time::SystemTime::now().duration_since(web_time::SystemTime::UNIX_EPOCH).unwrap().as_secs())/>
                                }.into_any()
                            }}
                            </Suspense>
                        </div>
                    </div>
                    </Show>
            }.into_any()
        }}
    }
}

#[component]
fn EmptyNotifications() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center h-full w-full px-4">
            <div class="flex flex-col items-center gap-12">
                <img
                    src="/img/yral/empty-notification-bell.png"
                    alt="No notifications"
                    class="w-32 h-32"
                />
                <div class="text-center max-w-xs">
                    <h3 class="text-xl font-semibold text-white">No Notifications Found!</h3>
                    <p class="text-base text-neutral-300 mt-2">"We'll notify you when there's something new or exciting."</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn NotificationInfiniteScroller(last_viewed_time: u64) -> impl IntoView {
    let auth = auth_state();
    let cans = unauth_canisters();
    let provider = NotificationProvider {
        auth,
        canisters: cans.clone(),
        last_viewed_time,
    };
    view! {
        <div class="flex flex-col px-4 pb-32 mx-auto w-full max-w-5xl h-full">
                <InfiniteScroller
                    provider
                    fetch_count=10
                    children=move |notifications, _ref| {
                        view! {
                            <div node_ref=_ref.unwrap_or_default()>
                                <NotificationItem notif=notifications.0.0 is_read=notifications.0.1 />
                            </div>
                        }
                    }
                    empty_content= move || view! { <EmptyNotifications /> }
                />
            </div>
    }
}
