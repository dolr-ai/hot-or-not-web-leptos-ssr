#[cfg(feature = "ssr")]
mod server_impl;
#[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
pub mod yral;

use candid::Principal;
use codee::string::JsonSerdeCodec;
use consts::auth::REFRESH_MAX_AGE;
use consts::AUTH_JOURNEY_PAGE;
use global_constants::{NEW_USER_SIGNUP_REWARD_SATS, REFERRAL_REWARD_SATS};
use hon_worker_common::sign_referral_request;
use hon_worker_common::ReferralReqWithSignature;
use leptos::logging;
use leptos::prelude::ServerFnError;
use leptos::{ev, prelude::*, reactive::wrappers::write::SignalSetter};
use leptos_icons::Icon;
use leptos_router::hooks::use_location;
use leptos_router::hooks::use_navigate;
use leptos_use::use_cookie_with_options;
use leptos_use::UseCookieOptions;
use state::canisters::auth_state;
use utils::event_streaming::events::CentsAdded;
use utils::event_streaming::events::EventCtx;
use utils::event_streaming::events::{LoginMethodSelected, LoginSuccessful, ProviderKind};
use utils::mixpanel::mixpanel_events::BottomNavigationCategory;
use utils::mixpanel::mixpanel_events::MixPanelEvent;
use utils::mixpanel::mixpanel_events::MixpanelGlobalProps;
use utils::send_wrap;
use utils::types::NewIdentity;
use yral_canisters_common::Canisters;
use yral_metadata_client::MetadataClient;

#[server]
async fn issue_referral_rewards(worker_req: ReferralReqWithSignature) -> Result<(), ServerFnError> {
    server_impl::issue_referral_rewards(worker_req).await
}

#[server]
async fn mark_user_registered(user_principal: Principal) -> Result<bool, ServerFnError> {
    server_impl::mark_user_registered(user_principal).await
}

pub async fn handle_user_login(
    canisters: Canisters<true>,
    event_ctx: EventCtx,
    referrer: Option<Principal>,
    page_name: Option<BottomNavigationCategory>,
    email: Option<String>,
) -> Result<(), ServerFnError> {
    let user_principal = canisters.user_principal();
    let first_time_login = mark_user_registered(user_principal).await?;

    let auth_journey = MixpanelGlobalProps::get_auth_journey();
    // TODO: Move for first_time_login only
    let metadata_client: MetadataClient<false> = MetadataClient::default();

    if let Some(email) = email {
        let identity = canisters.identity();
        let _ = metadata_client
            .set_user_email(identity, email, !first_time_login)
            .await;
    }
    let _ = metadata_client
        .set_signup_datetime(user_principal, !first_time_login)
        .await;

    let page_name = page_name.unwrap_or_default();

    if first_time_login {
        CentsAdded.send_event(event_ctx, "signup".to_string(), NEW_USER_SIGNUP_REWARD_SATS);
        let global = MixpanelGlobalProps::try_get(&canisters, true);
        MixPanelEvent::track_signup_success_async(
            global,
            referrer.is_some(),
            referrer.map(|f| f.to_text()),
            auth_journey,
            page_name,
        )
        .await;
    } else {
        let global = MixpanelGlobalProps::try_get(&canisters, true);
        MixPanelEvent::track_login_success_async(global, auth_journey, page_name).await;
    }

    match referrer {
        Some(referrer_principal) if first_time_login => {
            let req = hon_worker_common::ReferralReq {
                referrer: referrer_principal,
                referee: user_principal,
                referee_canister: canisters.user_canister(),
                amount: REFERRAL_REWARD_SATS,
            };
            let sig = sign_referral_request(canisters.identity(), req.clone())?;
            issue_referral_rewards(ReferralReqWithSignature {
                request: req,
                signature: sig,
            })
            .await?;
            CentsAdded.send_event(event_ctx, "referral".to_string(), REFERRAL_REWARD_SATS);
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
    pub login_complete: SignalSetter<NewIdentity>,
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
    #[prop(optional, into)] reload_window: bool,
) -> impl IntoView {
    let auth = auth_state();

    let processing = RwSignal::new(None);

    let event_ctx = auth.event_ctx();

    let loc = use_location();

    let nav = use_navigate();

    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(event_ctx) {
        let page_name = global.page_name();
        MixPanelEvent::track_auth_screen_viewed(global, page_name);
    }

    let (_, set_auth_journey_page) =
        use_cookie_with_options::<BottomNavigationCategory, JsonSerdeCodec>(
            AUTH_JOURNEY_PAGE,
            UseCookieOptions::default()
                .path("/")
                .max_age(REFRESH_MAX_AGE.as_millis() as i64),
        );

    let login_action = Action::new(move |new_id: &NewIdentity| {
        // Clone the necessary parts
        let new_id = new_id.clone();
        let redirect_to = redirect_to.clone();
        let path = loc.pathname.get();
        let page_name = BottomNavigationCategory::try_from(path.clone()).ok();

        let nav = nav.clone();
        // let start = start.clone();
        // Capture the context signal setter
        send_wrap(async move {
            let referrer = auth.referrer_store.get_untracked();

            let mut canisters = auth
                .set_new_identity_and_wait_for_authentication(new_id.clone(), true)
                .await?;

            // HACK: leptos can panic sometimes and reach an undefined state
            // while the panic is not fixed, we use this workaround
            if canisters.user_principal()
                != Principal::self_authenticating(&new_id.id_wire.from_key)
            {
                canisters = Canisters::authenticate_with_network(new_id.id_wire).await?;
            }

            if let Err(e) = handle_user_login(
                canisters.clone(),
                auth.event_ctx(),
                referrer,
                page_name,
                new_id.email,
            )
            .await
            {
                log::warn!("failed to handle user login, err {e}. skipping");
            }

            let _ = LoginSuccessful.send_event(canisters.clone());

            if reload_window {
                let res = window().location().reload();
                logging::log!("Reloading window after login: {:#?}", res);
            }
            set_auth_journey_page.set(None);
            show_modal.set(false);

            if let Some(redir_loc) = redirect_to {
                nav(&redir_loc, Default::default());
            }

            Ok::<_, ServerFnError>(())
        })
    });

    let ctx = LoginProvCtx {
        processing: processing.read_only(),
        set_processing: SignalSetter::map(move |val: Option<ProviderKind>| {
            lock_closing.set(val.is_some());
            processing.set(val);
        }),
        login_complete: SignalSetter::map(move |val: NewIdentity| {
            // Dispatch just the DelegatedIdentityWire
            logging::log!("email: {:?}", val.email);
            login_action.dispatch(val);
        }),
    };
    provide_context(ctx);

    view! {
        <div class="flex justify-center items-center py-6 px-4 w-full h-full cursor-auto">
            <div class="overflow-hidden relative items-center w-full max-w-md rounded-md cursor-auto h-fit bg-neutral-950">
                <img
                    src="/img/common/refer-bg.webp"
                    class="object-cover absolute inset-0 z-0 w-full h-full opacity-40"
                />
                <div
                    style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                    class="absolute z-[1] size-[50rem] -left-[75%] -top-[50%]"
                ></div>
                <button
                    on:click=move |_| show_modal.set(false)
                    class="flex absolute top-4 right-4 justify-center items-center text-lg text-center text-white rounded-full md:text-xl size-6 bg-neutral-600 z-[3]"
                >
                    <Icon icon=icondata::ChCross />
                </button>
                <div class="flex relative flex-col gap-8 justify-center items-center py-10 px-12 text-white z-[2]">
                    <img src="/img/common/join-yral.webp" class="object-contain h-52" />
                    <div class="text-base font-bold text-center">
                        "Login in to watch, play & earn Bitcoin."
                    </div>
                    <div class="flex flex-col gap-4 items-center w-full">
                        {
                            #[cfg(any(feature = "oauth-ssr", feature = "oauth-hydrate"))]
                            view! { <yral::YralAuthProvider /> }
                        }
                    </div>
                    <div class="flex flex-col items-center text-center text-md">
                        <div>"By signing up, you agree to our"</div>
                        <a class="font-bold text-pink-300" target="_blank" href="/terms-of-service">
                            "Terms of Service"
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}
