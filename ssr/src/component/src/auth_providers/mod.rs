#[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
pub mod google;
#[cfg(feature = "local-auth")]
pub mod local_storage;
use candid::Principal;
use codee::string::FromToStringCodec;
use consts::ACCOUNT_CONNECTED_STORE;
use consts::NEW_USER_SIGNUP_REWARD;
use consts::REFERRAL_REWARD;
use ic_agent::Identity;
use leptos::prelude::ServerFnError;
use leptos::{ev, prelude::*, reactive::wrappers::write::SignalSetter};
use leptos_use::storage::use_local_storage;
use state::{auth::auth_state, local_storage::use_referrer_store};
use utils::event_streaming::events::CentsAdded;
use utils::event_streaming::events::{LoginMethodSelected, LoginSuccessful, ProviderKind};
use yral_canisters_common::Canisters;
use yral_types::delegated_identity::DelegatedIdentityWire;

#[server]
async fn issue_referral_rewards(referee_canister: Principal) -> Result<(), ServerFnError> {
    use self::server_fn_impl::issue_referral_rewards_impl;
    issue_referral_rewards_impl(referee_canister).await
}

#[server]
async fn mark_user_registered(user_principal: Principal) -> Result<bool, ServerFnError> {
    use self::server_fn_impl::mark_user_registered_impl;
    use state::canisters::unauth_canisters;

    // TODO: verify that user principal is registered
    let canisters = unauth_canisters();
    let user_canister = canisters
        .get_individual_canister_by_user_principal(user_principal)
        .await?
        .ok_or_else(|| ServerFnError::new("User not found"))?;
    mark_user_registered_impl(user_canister).await
}

pub async fn handle_user_login(
    canisters: Canisters<true>,
    referrer: Option<Principal>,
) -> Result<(), ServerFnError> {
    let user_principal = canisters.identity().sender().unwrap();
    let first_time_login = mark_user_registered(user_principal).await?;

    if first_time_login {
        CentsAdded.send_event("signup".to_string(), NEW_USER_SIGNUP_REWARD);
    }

    match referrer {
        Some(_referee_principal) if first_time_login => {
            issue_referral_rewards(canisters.user_canister()).await?;
            CentsAdded.send_event("referral".to_string(), REFERRAL_REWARD);
            Ok(())
        }
        _ => Ok(()),
    }
}

#[derive(Clone, Copy)]
pub struct LoginProvCtx {
    /// Setting processing should only be done on login cancellation
    /// and inside [LoginProvButton]
    /// stores the current provider handling the login
    pub processing: ReadSignal<Option<ProviderKind>>,
    pub set_processing: SignalSetter<Option<ProviderKind>>,
    pub login_complete: SignalSetter<DelegatedIdentityWire>,
}

/// Login providers must use this button to trigger the login action
/// automatically sets the processing state to true
#[component]
fn LoginProvButton<Cb: Fn(ev::MouseEvent) + 'static>(
    prov: ProviderKind,
    #[prop(into)] class: Oco<'static, str>,
    on_click: Cb,
    #[prop(optional, into)] disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();

    let click_action = Action::new(move |()| async move {
        LoginMethodSelected.send_event(prov);
    });

    view! {
        <button
            disabled=move || ctx.processing.get().is_some() || disabled()
            class=class
            on:click=move |ev| {
                ctx.set_processing.set(Some(prov));
                on_click(ev);
                click_action.dispatch(());
            }
        >

            {children()}
        </button>
    }
}

#[component]
pub fn LoginProviders(show_modal: RwSignal<bool>, lock_closing: RwSignal<bool>) -> impl IntoView {
    let (_, write_account_connected, _) =
        use_local_storage::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
    let auth = auth_state();

    let processing = RwSignal::new(None);
    let (referrer_store, _, _) = use_referrer_store();

    let login_action = Action::new_local(move |id: &DelegatedIdentityWire| {
        let id = id.clone();
        async move {
            let referrer = referrer_store.get_untracked();

            // This is some redundant work, but saves us 100+ lines of resource handling
            let canisters = Canisters::authenticate_with_network(id, referrer).await?;

            if let Err(e) = handle_user_login(canisters.clone(), referrer).await {
                log::warn!("failed to handle user login, err {e}. skipping");
            }

            let _ = LoginSuccessful.send_event(canisters);

            Ok::<_, ServerFnError>(())
        }
    });

    let ctx = LoginProvCtx {
        processing: processing.read_only(),
        set_processing: SignalSetter::map(move |val: Option<ProviderKind>| {
            lock_closing.set(val.is_some());
            processing.set(val);
        }),
        login_complete: SignalSetter::map(move |val: DelegatedIdentityWire| {
            login_action.dispatch(val.clone());
            write_account_connected(true);
            auth.set(Some(val));
            show_modal.set(false);
        }),
    };
    provide_context(ctx);

    view! {
        <div class="flex flex-col py-12 px-16 items-center gap-2 bg-neutral-900 text-white cursor-auto">
        <h1 class="text-xl">Login to Yral</h1>
        <img class="h-32 w-32 object-contain my-8" src="/img/yral/logo.webp" />
        <span class="text-md">Continue with</span>
        <div class="flex flex-col w-full gap-4 items-center">

            {
                #[cfg(feature = "local-auth")]
                view! {
                    <local_storage::LocalStorageProvider></local_storage::LocalStorageProvider>
                }
            }
            {
                #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
                view! { <google::GoogleAuthProvider></google::GoogleAuthProvider> }
            }
            <div id="tnc" class="text-white text-center">
                By continuing you agree to our <a class="text-primary-600 underline" href="/terms-of-service">Terms of Service</a>
            </div>
        </div>
    </div>
    }
}

#[cfg(feature = "ssr")]
mod server_fn_impl {
    #[cfg(feature = "backend-admin")]
    pub use backend_admin::*;
    #[cfg(not(feature = "backend-admin"))]
    pub use mock::*;

    #[cfg(feature = "backend-admin")]
    mod backend_admin {
        use candid::Principal;
        use leptos::prelude::*;

        use state::canisters::unauth_canisters;
        use yral_canisters_client::individual_user_template::KnownPrincipalType;

        pub async fn issue_referral_rewards_impl(
            referee_canister: Principal,
        ) -> Result<(), ServerFnError> {
            let canisters = unauth_canisters();
            let user = canisters.individual_user(referee_canister).await;
            let referrer_details = user
                .get_profile_details()
                .await?
                .referrer_details
                .ok_or(ServerFnError::new("Referrer details not found"))?;

            let referrer = canisters
                .individual_user(referrer_details.user_canister_id)
                .await;

            let user_details = user.get_profile_details().await?;

            let referrer_index_principal = referrer
                .get_well_known_principal_value(KnownPrincipalType::CanisterIdUserIndex)
                .await?
                .ok_or_else(|| ServerFnError::new("User index not present in referrer"))?;
            let user_index_principal = user
                .get_well_known_principal_value(KnownPrincipalType::CanisterIdUserIndex)
                .await?
                .ok_or_else(|| ServerFnError::new("User index not present in referee"))?;

            issue_referral_reward_for(
                user_index_principal,
                referee_canister,
                referrer_details.profile_owner,
                user_details.principal_id,
            )
            .await?;
            issue_referral_reward_for(
                referrer_index_principal,
                referrer_details.user_canister_id,
                referrer_details.profile_owner,
                user_details.principal_id,
            )
            .await?;

            Ok(())
        }

        async fn issue_referral_reward_for(
            user_index: Principal,
            user_canister_id: Principal,
            referrer_principal_id: Principal,
            referee_principal_id: Principal,
        ) -> Result<(), ServerFnError> {
            use state::admin_canisters::admin_canisters;
            use yral_canisters_client::user_index::Result_;

            let admin_cans = admin_canisters();
            let user_idx = admin_cans.user_index_with(user_index).await;
            let res = user_idx
                .issue_rewards_for_referral(
                    user_canister_id,
                    referrer_principal_id,
                    referee_principal_id,
                )
                .await?;
            if let Result_::Err(e) = res {
                return Err(ServerFnError::new(format!(
                    "failed to issue referral reward {e}"
                )));
            }
            Ok(())
        }

        pub async fn mark_user_registered_impl(
            user_canister: Principal,
        ) -> Result<bool, ServerFnError> {
            use state::admin_canisters::admin_canisters;
            use yral_canisters_client::individual_user_template::{
                Result14, Result24, SessionType,
            };

            let admin_cans = admin_canisters();
            let user = admin_cans.individual_user_for(user_canister).await;
            if matches!(
                user.get_session_type().await?,
                Result14::Ok(SessionType::RegisteredSession)
            ) {
                return Ok(false);
            }
            user.update_session_type(SessionType::RegisteredSession)
                .await
                .map_err(ServerFnError::from)
                .and_then(|res| match res {
                    Result24::Ok(_) => Ok(()),
                    Result24::Err(e) => Err(ServerFnError::new(format!(
                        "failed to mark user as registered {e}"
                    ))),
                })?;
            Ok(true)
        }
    }

    #[cfg(not(feature = "backend-admin"))]
    mod mock {
        use candid::Principal;
        use leptos::prelude::ServerFnError;
        pub async fn issue_referral_rewards_impl(
            _referee_canister: Principal,
        ) -> Result<(), ServerFnError> {
            Ok(())
        }

        pub async fn mark_user_registered_impl(
            _user_canister: Principal,
        ) -> Result<bool, ServerFnError> {
            Ok(true)
        }
    }
}
