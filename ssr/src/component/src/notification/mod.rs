use crate::notification::provider::{NotificationError, NotificationProvider};
use crate::overlay::{ShadowOverlay, ShowOverlay};
use crate::{infinite_scroller::InfiniteScroller, title::TitleText};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
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
fn NotificationItem(notif: NotificationData) -> impl IntoView {
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

    let notif_clone = notif.clone();
    let auth = auth_state();
    let href_icon = auth.derive_resource(
        move || notif.clone(),
        move |cans, notif| async move {
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

    let set_read = Action::new(move |()| async move {
        let cans = send_wrap(auth.auth_cans(unauth_canisters()))
            .await
            .map_err(|e| NotificationError(e.to_string()))
            .unwrap();
        let agent = cans.authenticated_user().await.1;
        let client = NotificationStore(NOTIFICATION_STORE_ID, agent);
        send_wrap(client.mark_notification_as_read(notif_clone.notification_id))
            .await
            .map_err(|e| NotificationError(e.to_string()))
            .unwrap();
    });

    view! {
        <Suspense fallback=NotificationLoadingItem>
            {move || {
                match href_icon.get() {
                    Some(Ok((href_value, icon))) => {
                        let href_value_clone = href_value.clone();
                        let nav = use_navigate();

                        view! {
                            <div class="w-full p-4 border-b border-neutral-800">
                                <div class="flex items-start gap-3">
                                    <div class="relative mt-0.5">
                                        <div class="size-2 rounded-full bg-pink-700 absolute -left-1 top-1/2 -translate-y-1/2 z-10"></div>
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
                                            <NotificationActionButton on_click=move || {
                                                set_read.dispatch(());
                                                nav(&href_value_clone, NavigateOptions::default());
                                            }>View</NotificationActionButton>
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
fn NotificationActionButton(
    children: Children,
    on_click: impl Fn() + 'static,
    #[prop(optional)] secondary: bool,
) -> impl IntoView {
    let on_click = move |_| on_click();

    view! {
        <button
        on:click=on_click
        class=format!("border px-5 py-2 font-semibold rounded-lg {}", if secondary { "border-neutral-700 text-neutral-700" } else { "border-pink-700 text-pink-700" })>
            {children()}
        </button>
    }
}

#[component]
pub fn NotificationPage(close: RwSignal<bool>) -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Notifications";

    let auth = auth_state();
    let provider = NotificationProvider { auth };
    let is_desktop = use_media_query("(min-width: 1024px)");

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
                            <NotificationInfiniteScroller provider=provider />
                        </div>
                    </div>
                </ShadowOverlay>
            }.into_any()
        } else {
            // Mobile: current layout
            view! {
                <div class="flex flex-col items-center pt-4 pb-12 w-screen min-h-screen text-white bg-black h-full">
                    <div class="sticky top-0 z-10 w-full bg-black">
                        <TitleText justify_center=false>
                            <div class="relative flex items-center justify-center">
                                <button
                                    on:click=move |_| close.set(false)
                                    class="absolute left-3"
                                >
                                    <Icon icon=icondata::AiLeftOutlined attr:class="w-6 h-6" />
                                </button>
                                <span class="text-xl font-bold">Notifications</span>
                            </div>
                        </TitleText>
                    </div>
                    <NotificationInfiniteScroller provider=provider />
                </div>
            }.into_any()
        }}
    }
}

#[component]
fn NotificationInfiniteScroller(provider: NotificationProvider) -> impl IntoView {
    view! {
            <div class="flex overflow-hidden overflow-y-auto flex-col px-4 pb-32 mx-auto mt-2 w-full max-w-5xl h-full">
                <InfiniteScroller
                    provider
                    fetch_count=10
                    children=move |notifications, _ref| {
                        view! {
                            <div node_ref=_ref.unwrap_or_default()>
                                <NotificationItem notif=notifications.0 />
                            </div>
                        }
                    }
                />
            </div>
    }
}
