use component::{back_btn::BackButton, infinite_scroller::InfiniteScroller, title::TitleText};
use leptos::prelude::*;
use leptos_meta::*;
use state::canisters::unauth_canisters;
use state::canisters::AuthState;
use state::{app_state::AppState, canisters::auth_state};
use yral_canisters_client::notification_store::{
    NotificationData, NotificationStore, NotificationType,
};
use yral_canisters_common::cursored_data::{CursoredDataProvider, KeyedData, PageEntry};

#[component]
fn NotificationLoadingItem() -> impl IntoView {
    view! {
        <div class="bg-black w-full p-4 border-b border-neutral-900">
             <div class="flex items-center gap-3">
                 <div class="size-11 rounded-full bg-neutral-900 animate-pulse" />
                 <div class="flex flex-col gap-1.5 w-1/2">
                    <div class="w-1/2 h-3.5 rounded-full bg-neutral-800 animate-pulse" />
                    <div class="w-full h-3.5 rounded-full bg-neutral-900 animate-pulse" />
                 </div>
             </div>
        </div>
    }
}

#[component]
fn NotificationItem(notif: NotificationData) -> impl IntoView {
    let img_src = "/img/common/error-logo.svg";

    let (title, description) = match &notif.payload {
        NotificationType::VideoUpload(v) => (
            "Video Uploaded Sucessfully!".to_string(),
            format!("{} uploaded a video", v.video_uid),
        ),
        NotificationType::Liked(v) => (
            format!("Video Liked by {}", v.by_user_principal),
            format!("{} liked your video", v.by_user_principal),
        ),
    };

    let auth = auth_state();
    let href = auth.derive_resource(
        move || notif.clone(),
        move |cans, notif| async move {
            let path = match notif.payload {
                NotificationType::VideoUpload(v) => {
                    format!("hot-or-not/{}/{}", cans.user_canister(), v.video_uid)
                }
                NotificationType::Liked(v) => {
                    format!("hot-or-not/{}/{}", cans.user_canister(), v.post_id)
                }
            };
            Ok(path)
        },
    );

    view! {
        <Suspense fallback=NotificationLoadingItem>
            {move || {
                let href_value = match href.get() {
                    Some(Ok(path)) => path,
                    _ => "/wallet".to_string(),
                };

                view! {
                    <a href=href_value class="bg-black w-full p-4 border-b border-neutral-900">
                        <div class="flex items-center gap-3">
                            <div class="size-11 rounded-full bg-neutral-800 relative">
                                <img src={img_src} class="size-11 rounded-full object-cover" />
                                <div class="size-2 rounded-full bg-pink-700 absolute -left-4 top-5"></div>
                            </div>
                            <div class="flex flex-col gap-1">
                                <div class="text-neutral-50 font-semibold">
                                    {title.clone()}
                                </div>
                                <div class="text-neutral-500 font-semibold line-clamp-2">
                                    {description.clone()}
                                </div>
                                <div class="flex items-center gap-2 pt-1">
                                    <NotificationActionButton on_click=move || {}>View</NotificationActionButton>
                                    <NotificationActionButton on_click=move || {}>Accept</NotificationActionButton>
                                    <NotificationActionButton on_click=move || {} secondary=true>Reject</NotificationActionButton>
                                </div>
                                <div class="flex items-center gap-4 flex-wrap pt-1">
                                    <NotificationItemStatus status="accepted".to_string() />
                                    <NotificationItemStatus status="pending".to_string() />
                                    <NotificationItemStatus status="rejected".to_string() />
                                </div>
                            </div>
                        </div>
                    </a>
                }
            }}
        </Suspense>
    }
}

#[component]
fn NotificationItemStatus(status: String) -> impl IntoView {
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

#[derive(Clone)]
pub struct NotificationDataKeyed(NotificationData);

impl KeyedData for NotificationDataKeyed {
    type Key = String;
    fn key(&self) -> String {
        self.0.notification_id.to_string()
    }
}

// Custom error type that implements StdError
#[derive(Debug, Clone)]
pub struct NotificationError(String);

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for NotificationError {}

#[derive(Clone, Copy)]
pub struct NotificationProvider {
    auth: AuthState,
}

impl CursoredDataProvider for NotificationProvider {
    type Data = NotificationDataKeyed;
    type Error = NotificationError;

    async fn get_by_cursor_inner(
        &self,
        start: usize,
        end: usize,
    ) -> Result<PageEntry<Self::Data>, Self::Error> {
        // Get authenticated canisters and build a NotificationStore on-demand
        let cans = self
            .auth
            .auth_cans(unauth_canisters())
            .await
            .map_err(|e| NotificationError(e.to_string()))?;

        let agent = cans.authenticated_user().await.1;
        let principal = agent
            .get_principal()
            .map_err(|e| NotificationError(e.to_string()))?;

        let client = NotificationStore(principal, agent);

        let notifications = client
            .get_notifications((end - start + 1) as u64, start as u64)
            .await
            .map_err(|e| NotificationError(e.to_string()))?;

        let list_end = notifications.len() < (end - start);
        Ok(PageEntry {
            data: notifications
                .into_iter()
                .map(NotificationDataKeyed)
                .collect(),
            end: list_end,
        })
    }
}

#[component]
pub fn NotificationPage() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Notifications";

    let auth = auth_state();
    let provider = NotificationProvider { auth };
    view! {
        <Title text=page_title />
        <div class="flex flex-col items-center pt-4 pb-12 w-screen min-h-screen text-white bg-black">
            <div class="sticky top-0 z-10 w-full bg-black">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between">
                        <BackButton fallback="/wallet".to_string() />
                        <div>
                            <span class="text-xl font-bold">Notifications</span>
                        </div>
                        <div></div>
                    </div>
                </TitleText>
            </div>

            <NotificationInfiniteScroller provider=provider />
        </div>
    }
}

#[component]
fn NotificationInfiniteScroller(provider: NotificationProvider) -> impl IntoView {
    view! {
            <div class="flex overflow-hidden overflow-y-auto flex-col px-8 mx-auto mt-2 w-full max-w-5xl h-full md:px-16">
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
                    custom_loader=move || {
                        view! { <NotificationLoadingItem /> }
                    }
                />
            </div>
    }
}
