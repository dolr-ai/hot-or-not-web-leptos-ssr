#[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
mod google;
// #[cfg(feature = "local-auth")]
// mod local_storage;

use candid::Principal;
use codee::string::FromToStringCodec;
use ic_agent::Identity;
use leptos::*;
use leptos_use::storage::use_local_storage;
use yral_auth_client::types::{DelegatedIdentityWire, SignedRefreshTokenClaim};

use crate::{
    consts::ACCOUNT_CONNECTED_STORE,
    try_or_redirect,
    state::{
        auth::{AuthState, auth_state, auth_client},
        canisters::{do_canister_auth, Canisters},
        local_storage::use_referrer_store,
    },
    utils::{
        event_streaming::events::{LoginMethodSelected, LoginSuccessful},
        MockPartialEq,
    },
};



#[server]
async fn generate_claim_for_migration() -> Result<Option<SignedRefreshTokenClaim>, ServerFnError> {
    use crate::utils::current_epoch;
    use axum_extra::extract::cookie::{Key, SignedCookieJar};
    use candid::Principal;
    use hmac::{Hmac, Mac};
    use http::{header, HeaderMap};
    use leptos_axum::{extract, extract_with_state, ResponseOptions};
    use serde::Deserialize;
    use web_time::Duration;
    use yral_auth_client::types::RefreshTokenClaim;

    #[derive(Deserialize)]
    struct RefreshToken {
        principal: Principal,
        expiry_epoch_ms: u128,
    }

    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    let Some(cookie) = jar.get("user-identity") else {
        return Ok(None);
    };
    let headers: HeaderMap = extract().await?;
    let Some(host) = headers.get(header::HOST).map(|h| h.to_str()).transpose()? else {
        return Ok(None);
    };

    let token: RefreshToken = serde_json::from_str(cookie.value())?;
    if current_epoch().as_millis() > token.expiry_epoch_ms {
        return Ok(None);
    }

    let signing = key.signing();
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(signing)?;

    let claim = RefreshTokenClaim {
        namespace: "YRAL".into(),
        principal: token.principal,
        expiry_epoch: current_epoch() + Duration::from_secs(300),
        referrer_host: url::Host::parse(host)?,
    };
    let raw = serde_json::to_vec(&claim)?;
    mac.update(&raw);
    let digest = mac.finalize().into_bytes();

    let resp: ResponseOptions = expect_context();
    resp.insert_header(
        "Set-Cookie".parse()?,
        "user-identity=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=None".parse()?,
    );

    Ok(Some(SignedRefreshTokenClaim {
        claim,
        digest: digest.to_vec(),
    }))
}

#[server]
async fn issue_referral_rewards(referee_canister: Principal) -> Result<(), ServerFnError> {
    use self::server_fn_impl::issue_referral_rewards_impl;
    issue_referral_rewards_impl(referee_canister).await
}

#[server]
async fn mark_user_registered(user_principal: Principal) -> Result<bool, ServerFnError> {
    use self::server_fn_impl::mark_user_registered_impl;
    use crate::state::canisters::unauth_canisters;

    // TODO: verify that user principal is registered
    let canisters = unauth_canisters();
    let user_canister = canisters
        .get_individual_canister_by_user_principal(user_principal)
        .await?
        .ok_or_else(|| ServerFnError::new("User not found"))?;
    mark_user_registered_impl(user_canister).await
}

async fn handle_user_login(
    canisters: Canisters<true>,
    referrer: Option<Principal>,
) -> Result<(), ServerFnError> {
    let user_principal = canisters.identity().sender().unwrap();
    let first_time_login = mark_user_registered(user_principal).await?;

    match referrer {
        Some(_referee_principal) if first_time_login => {
            issue_referral_rewards(canisters.user_canister()).await?;
            Ok(())
        }
        _ => Ok(()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    #[cfg(feature = "local-auth")]
    LocalStorage,
    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
    Google,
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
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    let ctx: LoginProvCtx = expect_context();

    let click_action = create_action(move |()| async move {
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

    let new_identity = create_rw_signal::<Option<DelegatedIdentityWire>>(None);

    let processing = create_rw_signal(None);

    create_local_resource(
        move || MockPartialEq(new_identity()),
        move |identity| async move {
            let Some(identity) = identity.0 else {
                return Ok(());
            };

            let (referrer_store, _, _) = use_referrer_store();
            let referrer = referrer_store.get_untracked();

            // This is some redundant work, but saves us 100+ lines of resource handling
            let cans_wire = do_canister_auth(Some(identity), referrer).await?;
            let Some(cans_wire) = cans_wire else {
                return Ok(())
            };

            let canisters = cans_wire.canisters()?;

            if let Err(e) = handle_user_login(canisters.clone(), referrer).await {
                log::warn!("failed to handle user login, err {e}. skipping");
            }

            LoginSuccessful.send_event(canisters);

            Ok::<_, ServerFnError>(())
        },
    );

    let ctx = LoginProvCtx {
        processing: processing.read_only(),
        set_processing: SignalSetter::map(move |val: Option<ProviderKind>| {
            lock_closing.set(val.is_some());
            processing.set(val);
        }),
        login_complete: SignalSetter::map(move |val: DelegatedIdentityWire| {
            new_identity.set(Some(val.clone()));
            write_account_connected(true);
            auth.identity.set(Some(val));
            show_modal.set(false);
        }),
    };
    provide_context(ctx);

    view! {
        <div class="flex flex-col py-12 px-16 items-center gap-2 bg-neutral-900 text-white cursor-auto">
            <h1 class="text-xl">Login to Yral</h1>
            <img class="h-32 w-32 object-contain my-8" src="/img/logo.webp"/>
            <span class="text-md">Continue with</span>
            <div class="flex w-full gap-4">

                {
                    #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
                    view! { <google::GoogleAuthProvider></google::GoogleAuthProvider> }
                }

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
        use leptos::ServerFnError;

        use crate::{
            canister::individual_user_template::KnownPrincipalType,
            state::canisters::unauth_canisters,
        };

        pub async fn issue_referral_rewards_impl(
            referee_canister: Principal,
        ) -> Result<(), ServerFnError> {
            let canisters = unauth_canisters();
            let user = canisters.individual_user(referee_canister).await?;
            let referrer_details = user
                .get_profile_details()
                .await?
                .referrer_details
                .ok_or(ServerFnError::new("Referrer details not found"))?;

            let referrer = canisters
                .individual_user(referrer_details.user_canister_id)
                .await?;

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
            use crate::{canister::user_index::Result_, state::admin_canisters::admin_canisters};

            let admin_cans = admin_canisters();
            let user_idx = admin_cans.user_index_with(user_index).await?;
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
            use crate::{
                canister::individual_user_template::{Result6, Result9, SessionType},
                state::admin_canisters::admin_canisters,
            };

            let admin_cans = admin_canisters();
            let user = admin_cans.individual_user_for(user_canister).await?;
            if matches!(
                user.get_session_type().await?,
                Result6::Ok(SessionType::RegisteredSession)
            ) {
                return Ok(false);
            }
            user.update_session_type(SessionType::RegisteredSession)
                .await
                .map_err(ServerFnError::from)
                .and_then(|res| match res {
                    Result9::Ok(_) => Ok(()),
                    Result9::Err(e) => Err(ServerFnError::new(format!(
                        "failed to mark user as registered {e}"
                    ))),
                })?;
            Ok(true)
        }
    }

    #[cfg(not(feature = "backend-admin"))]
    mod mock {
        use candid::Principal;
        use leptos::ServerFnError;

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

#[component]
pub fn AuthFrame(auth: RwSignal<Option<DelegatedIdentityWire>>) -> impl IntoView {
    let auth_res = create_local_resource(
        || (),
        move |_| async move {
            let auth_c = auth_client();
            let id = if let Some(claim) = try_or_redirect!(generate_claim_for_migration().await) {
                try_or_redirect!(auth_c.upgrade_refresh_token_claim(claim).await)
            } else {
                try_or_redirect!(auth_c.extract_or_generate_identity().await)
            };
            auth.set(Some(id));
        },
    );

    view! { <Suspense>{move || auth_res.get().map(|_| ())}</Suspense> }
}

#[component]
pub fn AuthProvider() -> impl IntoView {
    let auth = auth_state().identity;
    view! {
        <div class="hidden">
            <Show when=move || auth.with(|a| a.is_none())>
                <AuthFrame auth/>
            </Show>
        </div>
    }
}