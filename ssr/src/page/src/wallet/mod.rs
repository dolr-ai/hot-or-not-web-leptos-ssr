pub mod airdrop;
pub mod tokens;
pub mod transactions;
pub mod txn;

use candid::Principal;
use codee::string::FromToStringCodec;
use component::icons::notification_icon::NotificationIcon;
use component::share_popup::ShareButtonWithFallbackPopup;
use component::toggle::Toggle;
use component::{canisters_prov::AuthCansProvider, connect::ConnectLogin};
use consts::NOTIFICATIONS_ENABLED_STORE;
use leptos::html::Input;
use leptos::{ev, prelude::*};
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use leptos_use::storage::use_local_storage;
use leptos_use::use_event_listener;
use state::canisters::unauth_canisters;
use state::{app_state::AppState, canisters::authenticated_canisters};
use tokens::TokenList;
use utils::event_streaming::events::account_connected_reader;
use utils::notifications::get_device_registeration_token;
use utils::send_wrap;
use utils::try_or_redirect_opt;
use yral_canisters_common::utils::profile::ProfileDetails;
use yral_canisters_common::Canisters;
use yral_metadata_client::MetadataClient;

/// Controller for the login modal, passed through context
/// under wallet
#[derive(Debug, Clone, Copy)]
pub struct ShowLoginSignal(RwSignal<bool>);

#[component]
fn ProfileCard(details: ProfileDetails, is_own_account: bool, is_connected: bool) -> impl IntoView {
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

            <Show when=move || !is_connected && is_own_account>
                <ConnectLogin
                    show_login
                    login_text="Login to claim your Cents"
                    cta_location="wallet"
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

#[derive(Params, PartialEq)]
struct WalletParams {
    id: String,
}
#[component]
pub fn Wallet() -> impl IntoView {
    let params = use_params::<WalletParams>();
    let param_principal = move || {
        params.with(|p| {
            let WalletParams { id, .. } = p.as_ref().ok()?;
            Principal::from_text(id).ok()
        })
    };

    view! {
        {move || {
            match param_principal() {
                Some(principal) => view! { <WalletImpl principal /> }.into_any(),
                None => {
                    view! {
                        <AuthCansProvider let:cans>
                            {move || {
                                view! {
                                    <Redirect path=format!("/wallet/{}", cans.user_principal()) />
                                }
                            }}
                        </AuthCansProvider>
                    }.into_any()
                }
            }
        }}
    }
}

#[component]
pub fn WalletImpl(principal: Principal) -> impl IntoView {
    let (is_connected, _) = account_connected_reader();
    let show_login = RwSignal::new(false);

    provide_context(ShowLoginSignal(show_login));

    let auth_cans = authenticated_canisters();

    let profile_info_res = Resource::new(
        move || principal,
        move |principal| {
            send_wrap(async move {
                let cans_wire = auth_cans.await;
                let cans_wire = cans_wire?;
                let canisters = Canisters::from_wire(cans_wire, expect_context())?;

                let Some(user_canister) = canisters
                    .get_individual_canister_by_user_principal(principal)
                    .await?
                else {
                    return Err(ServerFnError::new("Failed to get user canister"));
                };
                let user = canisters.individual_user(user_canister).await;
                let user_details = user.get_profile_details().await?;
                Ok::<ProfileDetails, ServerFnError>(user_details.into())
            })
        },
    );

    let is_own_account = Resource::new(
        move || principal,
        move |principal| async move {
            let cans_wire = auth_cans.await?;
            let canisters = Canisters::from_wire(cans_wire, expect_context())?;
            Ok::<_, ServerFnError>(canisters.user_principal() == principal)
        },
    );

    let canister_id = Resource::new(
        move || principal,
        move |principal| {
            send_wrap(async move {
                let canisters = unauth_canisters();
                let Some(user_canister) = canisters
                    .get_individual_canister_by_user_principal(principal)
                    .await?
                else {
                    return Err(ServerFnError::new("Failed to get user canister"));
                };
                Ok((user_canister, principal))
            })
        },
    );

    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Wallet";
    view! {
        <div class="flex flex-col gap-4 pt-4 pb-12 bg-black min-h-dvh font-kumbh mx-auto max-w-md">
             <Title text=page_title />
             <Suspense fallback=move || view! { <HeaderLoading/> }>
                {move || {
                    let profile_details = try_or_redirect_opt!(profile_info_res.get()?);
                    let is_own_account = try_or_redirect_opt!(is_own_account.get()?);
                    Some(
                        view! {
                            <Header details=profile_details is_own_account=is_own_account/>
                        },
                    )
                }}
            </Suspense>
            <div class="flex h-full w-full flex-col items-center justify-center max-w-md mx-auto px-4 gap-4">
                <Suspense fallback=move || view! { <ProfileCardLoading/> }>
                    {move || {
                        let profile_details = try_or_redirect_opt!(profile_info_res.get()?);
                        let is_own_account = try_or_redirect_opt!(is_own_account.get()?);
                        Some(
                            view! { <ProfileCard details=profile_details is_connected=is_connected() is_own_account=is_own_account /> },
                        )
                    }}
                </Suspense>
                <Suspense>
                    {move || {
                        let canister_id = try_or_redirect_opt!(canister_id.get() ?);
                        Some(
                            view! {
                                <div class="font-kumbh self-start pt-3 font-bold text-lg text-white">
                                    My tokens
                                </div>
                                <TokenList user_principal=canister_id.1 user_canister=canister_id.0 />
                            },
                        )
                    }}
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
    let auth_cans = authenticated_canisters();
    let (notifs_enabled, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let on_token_click: Action<(), (), LocalStorage> = Action::new_unsync(move |()| async move {
        let metaclient: MetadataClient<false> = MetadataClient::default();

        let cans = Canisters::from_wire(auth_cans.await.unwrap(), expect_context()).unwrap();

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
                    <Toggle checked=notifs_enabled node_ref=toggle_ref />
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
