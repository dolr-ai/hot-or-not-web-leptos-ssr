#[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
pub mod google;
#[cfg(feature = "local-auth")]
pub mod local_storage;
use candid::Principal;
use consts::NEW_USER_SIGNUP_REWARD;
use consts::REFERRAL_REWARD;
use hon_worker_common::sign_referral_request;
use hon_worker_common::ReferralReqWithSignature;
use ic_agent::Identity;
use leptos::prelude::ServerFnError;
use leptos::{ev, prelude::*, reactive::wrappers::write::SignalSetter};
use leptos_router::hooks::use_navigate;
use state::canisters::auth_state;
use utils::event_streaming::events::CentsAdded;
use utils::event_streaming::events::EventCtx;
use utils::event_streaming::events::{LoginMethodSelected, LoginSuccessful, ProviderKind};
use utils::mixpanel::mixpanel_events::MixPanelEvent;
use utils::mixpanel::mixpanel_events::MixpanelGlobalProps;
use utils::mixpanel::mixpanel_events::MixpanelLoginSuccessProps;
use utils::mixpanel::mixpanel_events::MixpanelSignupSuccessProps;
use utils::send_wrap;
use yral_canisters_common::Canisters;
use yral_types::delegated_identity::DelegatedIdentityWire;

#[server]
async fn issue_referral_rewards(worker_req: ReferralReqWithSignature) -> Result<(), ServerFnError> {
    use self::server_fn_impl::issue_referral_rewards_impl;
    issue_referral_rewards_impl(worker_req).await
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
    event_ctx: EventCtx,
    referrer: Option<Principal>,
) -> Result<(), ServerFnError> {
    let user_principal = canisters.identity().sender().unwrap();
    let first_time_login = mark_user_registered(user_principal).await?;

    if first_time_login {
        CentsAdded.send_event(event_ctx, "signup".to_string(), NEW_USER_SIGNUP_REWARD);
        let global = MixpanelGlobalProps::try_get(&canisters, true);
        MixPanelEvent::track_signup_success(MixpanelSignupSuccessProps {
            user_id: global.user_id,
            visitor_id: global.visitor_id,
            is_logged_in: global.is_logged_in,
            canister_id: global.canister_id,
            is_nsfw_enabled: global.is_nsfw_enabled,
            is_referral: referrer.is_some(),
            referrer_user_id: referrer.map(|f| f.to_text()),
        });
    } else {
        let global = MixpanelGlobalProps::try_get(&canisters, true);
        MixPanelEvent::track_login_success(MixpanelLoginSuccessProps {
            user_id: global.user_id,
            visitor_id: global.visitor_id,
            is_logged_in: global.is_logged_in,
            canister_id: global.canister_id,
            is_nsfw_enabled: global.is_nsfw_enabled,
        });
    }

    MixPanelEvent::identify_user(user_principal.to_text().as_str());

    match referrer {
        Some(referrer_principal) if first_time_login => {
            let req = hon_worker_common::ReferralReq {
                referrer: referrer_principal,
                referee: user_principal,
                referee_canister: canisters.user_canister(),
            };
            let sig = sign_referral_request(canisters.identity(), req.clone())?;
            issue_referral_rewards(ReferralReqWithSignature {
                request: req,
                signature: sig,
            })
            .await?;
            CentsAdded.send_event(event_ctx, "referral".to_string(), REFERRAL_REWARD);
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

/// on_resolve -> a callback that returns the new principal
#[component]
pub fn LoginProviders(
    show_modal: RwSignal<bool>,
    lock_closing: RwSignal<bool>,
    redirect_to: Option<String>,
) -> impl IntoView {
    let auth = auth_state();

    let processing = RwSignal::new(None);

    let login_action = Action::new(move |id: &DelegatedIdentityWire| {
        // Clone the necessary parts
        let id = id.clone();
        let redirect_to = redirect_to.clone();
        // Capture the context signal setter
        send_wrap(async move {
            let referrer = auth.referrer_store.get_untracked();
            leptos::logging::log!("LoginProviders referrer: {:?}", referrer);
            auth.set_new_identity(id.clone(), true);

            let canisters = Canisters::authenticate_with_network(id, referrer).await?;

            if let Err(e) = handle_user_login(canisters.clone(), auth.event_ctx(), referrer).await {
                log::warn!("failed to handle user login, err {e}. skipping");
            }

            let _ = LoginSuccessful.send_event(canisters);

            if let Some(redir_loc) = redirect_to {
                let nav = use_navigate();
                nav(&redir_loc, Default::default());
            }

            // Update the context signal instead of writing directly
            show_modal.set(false);

            Ok::<_, ServerFnError>(())
        })
    });

    let ctx = LoginProvCtx {
        processing: processing.read_only(),
        set_processing: SignalSetter::map(move |val: Option<ProviderKind>| {
            lock_closing.set(val.is_some());
            processing.set(val);
        }),
        login_complete: SignalSetter::map(move |val: DelegatedIdentityWire| {
            // Dispatch just the DelegatedIdentityWire
            login_action.dispatch(val);
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
        use hon_worker_common::ReferralReqWithSignature;
        use hon_worker_common::WORKER_URL;
        use leptos::prelude::*;
        use state::server::HonWorkerJwt;
        use yral_canisters_client::individual_user_template::{Result22, Result9};

        pub async fn issue_referral_rewards_impl(
            worker_req: ReferralReqWithSignature,
        ) -> Result<(), ServerFnError> {
            let req_url = format!("{WORKER_URL}referral_reward");
            let client = reqwest::Client::new();
            let jwt = expect_context::<HonWorkerJwt>();
            let res = client
                .post(&req_url)
                .json(&worker_req)
                .bearer_auth(jwt.0)
                .send()
                .await?;

            if res.status() != reqwest::StatusCode::OK {
                return Err(ServerFnError::new(format!(
                    "worker error: {}",
                    res.text().await?
                )));
            }

            Ok(())
        }

        pub async fn mark_user_registered_impl(
            user_canister: Principal,
        ) -> Result<bool, ServerFnError> {
            use state::admin_canisters::admin_canisters;
            use yral_canisters_client::individual_user_template::SessionType;

            let admin_cans = admin_canisters();
            let user = admin_cans.individual_user_for(user_canister).await;
            if matches!(
                user.get_session_type().await?,
                Result9::Ok(SessionType::RegisteredSession)
            ) {
                return Ok(false);
            }
            user.update_session_type(SessionType::RegisteredSession)
                .await
                .map_err(ServerFnError::from)
                .and_then(|res| match res {
                    Result22::Ok(_) => Ok(()),
                    Result22::Err(e) => Err(ServerFnError::new(format!(
                        "failed to mark user as registered {e}"
                    ))),
                })?;
            Ok(true)
        }
    }

    #[cfg(not(feature = "backend-admin"))]
    mod mock {
        use candid::Principal;
        use hon_worker_common::ReferralReqWithSignature;
        use leptos::prelude::ServerFnError;
        pub async fn issue_referral_rewards_impl(
            _worker_req: ReferralReqWithSignature,
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
