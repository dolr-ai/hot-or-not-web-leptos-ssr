pub mod airdrop;
pub mod tokens;
pub mod transactions;
pub mod txn;

use candid::Principal;
use codee::string::FromToStringCodec;
use component::connect::ConnectLogin;
use component::icons::notification_icon::NotificationIcon;
use component::share_popup::ShareButtonWithFallbackPopup;
use component::toggle::Toggle;
use consts::NOTIFICATIONS_ENABLED_STORE;
use leptos::either::Either;
use leptos::html::Input;
use leptos::web_sys::{Notification, NotificationPermission};
use leptos::{ev, prelude::*};
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_params;
use leptos_router::params::{Params, ParamsError};
use leptos_use::storage::use_local_storage;
use leptos_use::use_event_listener;
use state::app_state::AppState;
use state::canisters::{auth_state, unauth_canisters};
use tokens::TokenList;
use utils::notifications::get_device_registeration_token;
use utils::send_wrap;
use yral_canisters_common::utils::profile::ProfileDetails;
use yral_metadata_client::MetadataClient;

/// Controller for the login modal, passed through context
/// under wallet
#[derive(Debug, Clone, Copy)]
pub struct ShowLoginSignal(RwSignal<bool>);

#[component]
fn ProfileCard(
    details: ProfileDetails,
    is_own_account: bool,
    is_connected: Signal<bool>,
) -> impl IntoView {
    let ShowLoginSignal(show_login) = expect_context();
    view! {
        <div class="w-full flex flex-col bg-neutral-900 rounded-lg p-4 gap-4">
            <div class="flex items-center gap-4">
                <img
                    src=details.profile_pic_or_random()
                    alt="Profile picture"
                    class="w-12 h-12 rounded-full object-cover shrink-0"
                />
                <span class="line-clamp-1 text-lg font-kumbh font-semibold select-all text-neutral-50">
                    // TEMP: Workaround for hydration bug until leptos 0.7
                    // class=("md:w-5/12", move || !is_connected())
                    {details.display_name_or_fallback()}
                </span>
            </div>

            <Show when=move || !is_connected.get() && is_own_account>
                <ConnectLogin
                    show_login
                    login_text="Login to claim your Bitcoin (SATS)"
                    cta_location="wallet"
                    redirect_to="/wallet"
                />
            </Show>
        </div>
    }
}

#[component]
fn ProfileCardLoading() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col bg-neutral-900 rounded-lg p-4 gap-4">
            <div class="flex items-center gap-4">
                <div
                    class="w-12 h-12 rounded-full bg-loading shrink-0"
                />
                <div class="flex-1 bg-loading rounded-lg h-7">
                </div>
            </div>
        </div>
    }
}

#[component]
fn Header(details: ProfileDetails, is_own_account: bool) -> impl IntoView {
    let share_link = {
        let principal = details.principal();
        format!("/wallet/{principal}")
    };
    let app_state = use_context::<AppState>();
    let message = format!(
        "Hey there 👋! Here's my wallet link on {}: {}",
        app_state.unwrap().name,
        share_link
    );

    view! {
        <div class="w-full flex items-center justify-between px-4 py-3 gap-10 ">
            <div class="text-white font-kumbh text-xl font-bold">My Wallet</div>
            <div class="flex items-center gap-8">
                <ShareButtonWithFallbackPopup share_link message />
                <Show when=move || is_own_account>
                    <a href="/wallet/notifications">
                        <NotificationIcon show_dot=false class="w-6 h-6 text-neutral-300" />
                    </a>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn HeaderLoading() -> impl IntoView {
    view! {
        <div class="w-full flex items-center justify-between px-4 py-3 gap-10 ">
            <div class="text-white font-kumbh text-xl font-bold">My Wallet</div>
            <div class="flex items-center gap-8">
                <div class="w-6 h-6 rounded-full bg-loading"></div>
                <div class="w-6 h-6 rounded-full bg-loading"></div>
            </div>
        </div>
    }
}

#[component]
fn FallbackGreeter() -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <span class="text-white/50 text-md">Welcome!</span>
            <div class="py-2 w-3/4 rounded-full animate-pulse bg-white/40"></div>
        </div>
        <div class="justify-self-end w-16 rounded-full animate-pulse aspect-square overflow-clip bg-white/40"></div>
    }
}

#[component]
fn BalanceFallback() -> impl IntoView {
    view! { <div class="py-3 mt-1 w-1/4 rounded-full animate-pulse bg-white/30"></div> }
}

#[derive(Params, PartialEq, Clone)]
struct WalletParams {
    id: Option<String>,
}
#[component]
pub fn Wallet() -> impl IntoView {
    let params = use_params::<WalletParams>();
    let param_principal = move || {
        let WalletParams { id } = params.get()?;
        Ok::<_, ParamsError>(id.and_then(|p| Principal::from_text(p).ok()))
    };

    view! {
        {move || {
            match param_principal() {
                Ok(Some(principal)) => Some(view! { <WalletImpl principal /> }.into_any()),
                Ok(None) => {
                    let auth = auth_state();
                    Some(view! {
                        <Suspense>
                            {move || auth.user_principal.get().map(|res| match res {
                                Ok(user_principal) => view! {
                                    <Redirect path=format!("/wallet/{user_principal}") />
                                },
                                Err(e) => view! {
                                    <Redirect path=format!("/error?err={e}") />
                                }
                            })}
                        </Suspense>
                    }.into_any())
                }
                Err(_) => None
            }
        }}
    }
}

#[component]
pub fn WalletImpl(principal: Principal) -> impl IntoView {
    let show_login = RwSignal::new(false);

    provide_context(ShowLoginSignal(show_login));

    let cans = unauth_canisters();

    let cans2 = cans.clone();
    let canister_id = OnceResource::new(send_wrap(async move {
        let canisters = cans2;
        let user_canister = canisters
            .get_individual_canister_by_user_principal(principal)
            .await?
            .ok_or_else(|| ServerFnError::new("Failed to get user canister"))?;
        Ok::<_, ServerFnError>(user_canister)
    }));

    let profile_info_res = OnceResource::new(send_wrap(async move {
        let user_canister = canister_id.await?;
        let user = cans.individual_user(user_canister).await;
        let user_details = user.get_profile_details().await?;
        Ok::<ProfileDetails, ServerFnError>(user_details.into())
    }));

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Wallet";

    view! {
        <div class="flex flex-col gap-4 pt-4 pb-12 bg-black min-h-dvh font-kumbh mx-auto max-w-md">
             <Title text=page_title />
             <Suspense fallback=move || view! { <HeaderLoading/> }>
                {move || Suspend::new(async move {
                    let profile_details = profile_info_res.await;
                    let logged_in_user = auth.user_principal.await;

                    match profile_details.and_then(|c| Ok((c, logged_in_user?))) {
                        Ok((profile_details, logged_in_user)) => {
                            let is_own_account = logged_in_user == principal;
                            Either::Left(view! {
                                <Header details=profile_details is_own_account/>
                            })
                        }
                        Err(e) => {
                            Either::Right(view! {
                                <Redirect path=format!("/error?err={e}") />
                            })
                        }
                    }
                })}
            </Suspense>
            <div class="flex h-full w-full flex-col items-center justify-center max-w-md mx-auto px-4 gap-4">
                <Suspense fallback=ProfileCardLoading>
                    {move || Suspend::new(async move {
                        let profile_details = profile_info_res.await;
                        let logged_in_user = auth.user_principal.await;

                        match profile_details.and_then(|c| Ok((c, logged_in_user?))) {
                            Ok((profile_details, logged_in_user)) => {
                                let is_own_account = logged_in_user == principal;
                                Either::Left(view! {
                                    <ProfileCard details=profile_details is_connected is_own_account />
                                })
                            }
                            Err(e) => {
                                Either::Right(view! {
                                    <Redirect path=format!("/error?err={e}") />
                                })
                            }
                        }
                    })}
                </Suspense>
                <Suspense>
                    {move || Suspend::new(async move {
                        let canister_id = canister_id.await;
                        match canister_id {
                            Ok(canister_id) => Either::Left(view! {
                                <div class="font-kumbh self-start pt-3 font-bold text-lg text-white">
                                    My tokens
                                </div>
                                <TokenList user_principal=principal user_canister=canister_id />
                            }),
                            Err(e) => {
                                Either::Right(view! {
                                    <Redirect path=format!("/error?err={e}") />
                                })
                            }
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }.into_any()
}

#[component]
pub fn NotificationWallet() -> impl IntoView {
    view! {
        <NotificationWalletImpl />
    }
    .into_any()
}

#[component]
pub fn NotificationWalletImpl() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Wallet";

    // Placeholder data for notifications
    let notifications = vec![];
    let toggle_ref = NodeRef::<Input>::new();

    let auth = auth_state();

    let (notifs_enabled, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let notifs_enabled_der = Signal::derive(move || {
        notifs_enabled.get()
            && matches!(Notification::permission(), NotificationPermission::Granted)
    });

    let on_token_click: Action<(), ()> = Action::new_unsync(move |()| async move {
        let metaclient: MetadataClient<false> = MetadataClient::default();

        let cans = auth.auth_cans(expect_context()).await.unwrap();

        let token = get_device_registeration_token().await.unwrap();

        log::info!("Notif enabled:{}", notifs_enabled.get_untracked());
        if notifs_enabled.get_untracked() {
            metaclient
                .unregister_device(cans.identity(), token)
                .await
                .unwrap();
            log::info!("Device unregistered successfully");
            set_notifs_enabled(false)
        } else {
            metaclient
                .register_device(cans.identity(), token)
                .await
                .unwrap();
            log::info!("Device registered sucessfully");
            set_notifs_enabled(true)
        }
    });

    _ = use_event_listener(toggle_ref, ev::change, move |_| {
        on_token_click.dispatch(());
    });

    view! {
        <div class="flex flex-col pt-4 pb-12 bg-black min-h-dvh overflow-x-hidden font-kumbh mx-auto max-w-md text-white">
            <Title text=page_title />
            <div class="sticky top-0 z-10 bg-black px-4 py-3 flex items-center justify-between">
                <a href="/wallet" class="text-white"> // Assuming back navigates to general wallet
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
                    </svg>
                </a>
                <h1 class="text-xl font-bold text-center flex-grow">Notification</h1>
                <div class="w-6"></div> // Spacer to balance the back button
            </div>

            <div class="flex flex-col gap-0"> // Changed gap-4 to gap-0 for tighter packing of elements
                <div class="flex items-center justify-between p-2 bg-neutral-900 my-4 rounded-lg">
                    <div class="flex items-center gap-3">
                        <NotificationIcon show_dot=false class="w-5 h-5 text-neutral-300" />
                        <span class="text-neutral-50">Allow Notifications</span>
                    </div>
                    <Toggle checked=notifs_enabled_der node_ref=toggle_ref />
                </div>

                <div class="flex flex-col">
                    {notifications
                        .into_iter()
                        .map(|notif_props| {
                            view! { <NotificationCard data=notif_props /> }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
    .into_any()
}

#[derive(Clone, PartialEq)]
struct NotificationCardData {
    title: String,
    message: String,
    image_src: String,
    is_read: bool,
}

#[component]
fn NotificationCard(data: NotificationCardData) -> impl IntoView {
    let title = data.title.clone();
    view! {
        <div class="flex items-center py-6 px-2 gap-4 border-b border-neutral-800 hover:bg-neutral-800 cursor-pointer">
            <Show when=move || !data.is_read >
                <div class="w-2 h-2 bg-pink-500 rounded-full self-start mt-2 shrink-0"></div>
            </Show>
            <Show when=move || data.is_read >
                 <div class="w-2 h-2 shrink-0"></div> // Placeholder for alignment when read
            </Show>
            <img src=data.image_src.clone() alt="Notification Icon" class="w-10 h-10 rounded-full object-cover shrink-0"/>
            <div class="flex flex-col">
                <Show when=move || !title.is_empty()>
                    <span class="font-semibold text-neutral-50">{data.title.clone()}</span>
                </Show>
                <span class="text-neutral-300 text-sm">{data.message.clone()}</span>
            </div>
        </div>
    }.into_any()
}
