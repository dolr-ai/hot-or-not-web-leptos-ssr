pub mod airdrop;
pub mod tokens;
pub mod transactions;
pub mod txn;

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
use utils::{send_wrap, UsernameOrPrincipal};
use yral_canisters_common::utils::profile::ProfileDetails;
use yral_metadata_client::MetadataClient;

use component::notification::NotificationPage;

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
        <div class="flex flex-col gap-4 p-4 w-full rounded-lg bg-neutral-900">
            <div class="flex gap-4 items-center">
                <img
                    src=details.profile_pic_or_random()
                    alt="Profile picture"
                    class="object-cover w-12 h-12 rounded-full shrink-0"
                />
                <div class="flex flex-col gap-2">
                    <span class="text-lg font-semibold select-all line-clamp-1 font-kumbh text-neutral-50">
                        @{details.username_or_fallback()}
                    </span>
                    <span class="text-xs text-neutral-400 line-clamp-1 select-all">
                        {details.principal()}
                    </span>
                </div>
            </div>

            <Show when=move || !is_connected.get() && is_own_account>
                <ConnectLogin
                    show_login
                    login_text="Login to claim YRAL"
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
        <div class="flex flex-col gap-4 p-4 w-full rounded-lg bg-neutral-900">
            <div class="flex gap-4 items-center">
                <div class="w-12 h-12 rounded-full bg-loading shrink-0" />
                <div class="flex flex-col gap-2 w-full animate-pulse">
                    <div class="w-full h-7 rounded-full bg-white/20"></div>
                    <div class="w-full h-4 rounded-full bg-white/20"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Header(details: ProfileDetails, is_own_account: bool) -> impl IntoView {
    let share_link = {
        let id = details.username_or_principal();
        format!("/wallet/{id}")
    };
    let app_state = use_context::<AppState>();
    let message = format!(
        "Hey there 👋! Here's my wallet link on {}: {}",
        app_state.unwrap().name,
        share_link
    );

    let notification_panel = RwSignal::new(false);

    view! {
        <NotificationPage close=notification_panel />
        <div class="flex gap-10 justify-between items-center py-3 px-4 w-full">
            <div class="text-xl font-bold text-white font-kumbh">My Wallet</div>
            <div class="flex gap-8 items-center">
                <ShareButtonWithFallbackPopup share_link message />
                <Show when=move || is_own_account>
                    <button on:click=move |_| notification_panel.set(true)>
                        <NotificationIcon show_dot=false class="w-6 h-6 text-neutral-300" />
                    </button>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn HeaderLoading() -> impl IntoView {
    view! {
        <div class="flex gap-10 justify-between items-center py-3 px-4 w-full">
            <div class="text-xl font-bold text-white font-kumbh">My Wallet</div>
            <div class="flex gap-8 items-center">
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
    id: Option<UsernameOrPrincipal>,
}
#[component]
pub fn Wallet() -> impl IntoView {
    let params = use_params::<WalletParams>();
    let param_id = move || {
        let WalletParams { id } = params.get()?;
        Ok::<_, ParamsError>(id)
    };

    view! {
        {move || {
            match param_id() {
                Ok(Some(id)) => Some(view! { <WalletImpl id /> }.into_any()),
                Ok(None) => {
                    let auth = auth_state();
                    Some(
                        view! {
                            <Suspense>
                                {move || {
                                    auth.user_principal
                                        .get()
                                        .map(|res| match res {
                                            Ok(user_principal) => {
                                                view! {
                                                    <Redirect path=format!("/wallet/{user_principal}") />
                                                }
                                            }
                                            Err(e) => {
                                                view! { <Redirect path=format!("/error?err={e}") /> }
                                            }
                                        })
                                }}
                            </Suspense>
                        }
                            .into_any(),
                    )
                }
                Err(_) => None,
            }
        }}
    }
}

#[component]
pub fn WalletImpl(id: UsernameOrPrincipal) -> impl IntoView {
    let show_login = RwSignal::new(false);

    provide_context(ShowLoginSignal(show_login));

    let cans = unauth_canisters();

    let id = StoredValue::new(id);

    let cans2 = cans.clone();
    let auth = auth_state();

    let query_user_metadata_result = OnceResource::new(send_wrap(async move {
        let canisters = cans2;
        let user_canister = canisters
            .get_user_metadata(id.with_value(|id| id.to_string()))
            .await?;
        Ok::<_, ServerFnError>(user_canister)
    }));

    let profile_info_res = OnceResource::new(send_wrap(async move {
        let profile_details = cans
            .get_profile_details(format!("{}", id.get_value()))
            .await
            .inspect_err(|e| log::warn!("{e}"))?;
        Ok::<_, ServerFnError>(profile_details)
    }));

    // Edge Case: unauthenticated user navigates to wallet page
    let query_user_principal_and_canister_res = Callback::new(move |()| async move {
        if let Some(meta) = query_user_metadata_result.await? {
            return Ok::<_, ServerFnError>((meta.user_principal, meta.user_canister_id));
        }
        let logged_in_user = auth.auth_cans().await?;
        let is_own_account = match id.get_value() {
            UsernameOrPrincipal::Principal(p) => p == logged_in_user.user_principal(),
            UsernameOrPrincipal::Username(u) => {
                Some(u) == logged_in_user.profile_details().username
            }
        };
        if is_own_account {
            Ok((
                logged_in_user.user_principal(),
                logged_in_user.user_canister(),
            ))
        } else {
            Err(ServerFnError::new("User canister not found"))
        }
    });

    // Edge Case: unauthenticated user navigates to wallet page
    let profile_details_and_is_owner_result = Callback::new(move |()| async move {
        let authenticated_canister_resource = auth.auth_cans().await?;
        let is_own_account = match id.get_value() {
            UsernameOrPrincipal::Principal(p) => {
                p == authenticated_canister_resource.user_principal()
            }
            UsernameOrPrincipal::Username(u) => {
                Some(u) == authenticated_canister_resource.profile_details().username
            }
        };
        let profile_details = if is_own_account {
            authenticated_canister_resource.profile_details()
        } else {
            let profile_details = profile_info_res.await?;
            profile_details.ok_or_else(|| ServerFnError::new("User canister not found"))?
        };
        Ok::<_, ServerFnError>((profile_details, is_own_account))
    });

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Wallet";

    view! {
        <div class="flex flex-col gap-4 pt-4 pb-12 mx-auto max-w-md bg-black min-h-dvh font-kumbh overflow-x-hidden">
            <Title text=page_title />
            <Suspense fallback=move || {
                view! { <HeaderLoading /> }
            }>
                {move || Suspend::new(async move {
                    match profile_details_and_is_owner_result.run(()).await {
                        Ok((profile_details, is_own_account)) => {
                            Either::Left(
                                view! { <Header details=profile_details is_own_account /> },
                            )
                        },
                        Err(e) => {
                            Either::Right(view! { <Redirect path=format!("/error?err={e}") /> })
                        }
                    }
                })}
            </Suspense>
            <div class="flex flex-col gap-4 justify-center items-center px-4 mx-auto w-full max-w-md h-full">
                <Suspense fallback=ProfileCardLoading>
                    {move || Suspend::new(async move {
                        match profile_details_and_is_owner_result.run(()).await {
                            Ok((profile_details, is_own_account)) => {
                                Either::Left(
                                    view! {
                                        <ProfileCard
                                            details=profile_details
                                            is_connected
                                            is_own_account
                                        />
                                    },
                                )
                            }
                            Err(e) => {
                                Either::Right(view! { <Redirect path=format!("/error?err={e}") /> })
                            }
                        }
                    })}
                </Suspense>
                <Suspense>
                    {move || Suspend::new(async move {
                        let meta = query_user_principal_and_canister_res.run(()).await;
                        match meta {
                            Ok((user_principal, user_canister)) => {
                                Either::Left(
                                    view! {
                                        <div class="self-start pt-3 text-lg font-bold text-white font-kumbh">
                                            My tokens
                                        </div>
                                        <TokenList
                                            user_principal
                                            user_canister
                                        />
                                    },
                                )
                            }
                            Err(e) => {
                                Either::Right(view! { <Redirect path=format!("/error?err={e}") /> })
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
    view! { <NotificationWalletImpl /> }.into_any()
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

        let cans = auth.auth_cans().await.unwrap();

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
        <div class="flex overflow-x-hidden flex-col pt-4 pb-12 mx-auto max-w-md text-white bg-black min-h-dvh font-kumbh">
            <Title text=page_title />
            <div class="flex sticky top-0 z-10 justify-between items-center py-3 px-4 bg-black">
                // Assuming back navigates to general wallet
                <a href="/wallet" class="text-white">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-6 h-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M15.75 19.5L8.25 12l7.5-7.5"
                        />
                    </svg>
                </a>
                <h1 class="text-xl font-bold text-center grow">Notification</h1>
                // Spacer to balance the back button
                <div class="w-6"></div>
            </div>

            // Changed gap-4 to gap-0 for tighter packing of elements
            <div class="flex flex-col gap-0">
                <div class="flex justify-between items-center p-2 my-4 rounded-lg bg-neutral-900">
                    <div class="flex gap-3 items-center">
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
        <div class="flex gap-4 items-center py-6 px-2 border-b cursor-pointer border-neutral-800 hover:bg-neutral-800">
            <Show when=move || !data.is_read>
                <div class="self-start mt-2 w-2 h-2 bg-pink-500 rounded-full shrink-0"></div>
            </Show>
            <Show when=move || data.is_read>
                // Placeholder for alignment when read
                <div class="w-2 h-2 shrink-0"></div>
            </Show>
            <img
                src=data.image_src.clone()
                alt="Notification Icon"
                class="object-cover w-10 h-10 rounded-full shrink-0"
            />
            <div class="flex flex-col">
                <Show when=move || !title.is_empty()>
                    <span class="font-semibold text-neutral-50">{data.title.clone()}</span>
                </Show>
                <span class="text-sm text-neutral-300">{data.message.clone()}</span>
            </div>
        </div>
    }.into_any()
}
