use candid::Principal;
use codee::string::FromToStringCodec;
use component::back_btn::BackButton;
use component::title::TitleText;
use consts::USER_PRINCIPAL_STORE;
use leptos::prelude::*;
use leptos_use::use_cookie;
use state::app_state::AppState;
use utils::notifications::get_notitfication;

#[component]
fn NotificationLoadingItem() -> impl IntoView {
    view! {
        <div class="bg-black w-full p-4 border-b border-neutral-900 animate-pulse h-28">
        </div>
    }
}

#[component]
fn NotificationItem() -> impl IntoView {
    view! {
        <a href="#target" class="bg-black w-full p-4 border-b border-neutral-900 animate-pulse">
            <div class="flex items-center gap-3">
                <div class="w-12 h-12 rounded-full bg-neutral-800 relative">
                    <img src="" class="size-11 rounded-full object-cover" />
                    <div class="size-2 rounded-full bg-pink-300 absolute -left-3 top-3"></div>
                </div>
                <div class="flex flex-col gap-2">
                    <div class="text-white font-bold text-sm">
                        Title
                    </div>
                    <div class="text-neutral-500 text-sm font-semibold line-clamp-2">
                        Description
                    </div>
                    <div class="flex items-center gap-2">
                        <button class="border border-pink-400">
                            View
                        </button>
                        <button class="border border-neutral-500">
                            Reject
                        </button>
                    </div>
                    <div class="bg-green-950/80 text-green-500">
                        Status: Accepted
                    </div>

                    <div class="bg-amber-950/80 text-amber-500">
                        Status: Pending
                    </div>

                    <div class="bg-red-950/80 text-red-500">
                        Status: Rejected
                    </div>
                </div>
            </div>
        </a>
    }
}

#[component]
pub fn NotificaitonPage() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let _page_title = app_state.unwrap().name.to_owned() + " - Notifications";
    let notifications = RwSignal::new(None);

    let (user_principal_cookie, _) =
        use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);

    let ctx: state::stdb_dolr_airdrop::WrappedContext = expect_context();
    Effect::new(move |_| {
        notifications.set(get_notitfication(user_principal_cookie.get().unwrap(), &*ctx.conn).ok())
    });

    view! {
        // <Title text=page_title />
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

                {
                    move || {
                        format!("{:?}", notifications.get())
                    }
                }
                // <NotificationLoadingItem />
                // <NotificationLoadingItem />
                // <NotifcaitonItem />
                // <NotifcaitonItem />
            </div>
        </div>
    }
}
