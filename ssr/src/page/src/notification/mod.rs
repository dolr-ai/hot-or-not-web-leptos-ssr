use component::{back_btn::BackButton, title::TitleText};
use leptos::prelude::*;
use leptos_meta::*;
use state::app_state::AppState;

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
fn NotificationItem() -> impl IntoView {
    let img_src = "/img/common/error-logo.svg";

    view! {
        <a href="#target" class="bg-black w-full p-4 border-b border-neutral-900">
            <div class="flex items-center gap-3">
                <div class="size-11 rounded-full bg-neutral-800 relative">
                    <img src={img_src} class="size-11 rounded-full object-cover" />
                    <div class="size-2 rounded-full bg-pink-700 absolute -left-4 top-5"></div>
                </div>
                <div class="flex flex-col gap-1">
                    <div class="text-neutral-50 font-semibold">
                        Title
                    </div>
                    <div class="text-neutral-500 font-semibold line-clamp-2">
                        Description
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

#[component]
pub fn NotificaitonPage() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Notifications";
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

            <div class="flex overflow-hidden overflow-y-auto flex-col px-8 mx-auto mt-2 w-full max-w-5xl h-full md:px-16">

                <NotificationLoadingItem />
                <NotificationLoadingItem />
                <NotificationItem />
                <NotificationItem />
            </div>
        </div>
    }
}
