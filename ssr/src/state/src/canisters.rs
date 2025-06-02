use std::future::{Future, IntoFuture};

use auth::{
    delegate_identity, extract_identity, generate_anonymous_identity_if_required,
    set_anonymous_identity_cookie,
};
use candid::Principal;
use codee::string::FromToStringCodec;
use consts::{
    auth::REFRESH_MAX_AGE, ACCOUNT_CONNECTED_STORE, AUTH_UTIL_COOKIES_MAX_AGE_MS, REFERRER_COOKIE,
    USER_CANISTER_ID_STORE, USER_PRINCIPAL_STORE,
};
use futures::FutureExt;
use ic_agent::identity::Secp256k1Identity;
use k256::elliptic_curve::JwkEcKey;
use leptos::prelude::*;
use leptos_router::{hooks::use_query, params::Params};
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use serde::{Deserialize, Serialize};
use yral_canisters_common::{Canisters, CanistersAuthWire};

use utils::{
    event_streaming::events::{EventCtx, EventUserDetails},
    send_wrap, MockPartialEq,
};
use yral_types::delegated_identity::DelegatedIdentityWire;

pub fn unauth_canisters() -> Canisters<false> {
    expect_context()
}

async fn do_canister_auth(
    auth: DelegatedIdentityWire,
    referrer: Option<Principal>,
) -> Result<CanistersAuthWire, ServerFnError> {
    let auth_fut = Canisters::authenticate_with_network(auth, referrer);
    let canisters = send_wrap(auth_fut).await?;
    Ok(canisters.into())
}
type AuthCansResource = Resource<Result<CanistersAuthWire, ServerFnError>>;

/// The Authenticated Canisters helper resource
/// prefer using helpers from [crate::component::canisters_prov]
/// instead
pub fn auth_state() -> AuthState {
    expect_context()
}

#[derive(Params, PartialEq, Clone)]
struct Referrer {
    user_refer: String,
}

#[derive(Copy, Clone)]
pub struct AuthState {
    _temp_identity_resource: OnceResource<Option<JwkEcKey>>,
    _temp_id_cookie_resource: LocalResource<()>,
    pub referrer_store: Signal<Option<Principal>>,
    is_logged_in_with_oauth: (Signal<Option<bool>>, WriteSignal<Option<bool>>),
    new_identity_setter: RwSignal<Option<DelegatedIdentityWire>>,
    canisters_resource: AuthCansResource,
    pub user_canister: Resource<Result<Principal, ServerFnError>>,
    user_canister_id_cookie: (Signal<Option<Principal>>, WriteSignal<Option<Principal>>),
    pub user_principal: Resource<Result<Principal, ServerFnError>>,
    user_principal_cookie: (Signal<Option<Principal>>, WriteSignal<Option<Principal>>),
    event_ctx: EventCtx,
}

impl Default for AuthState {
    fn default() -> Self {
        // Super complex, don't mess with this.

        let temp_identity_resource = OnceResource::new(async move {
            generate_anonymous_identity_if_required()
                .await
                .expect("Failed to generate anonymous identity?!")
        });
        let temp_id_cookie_resource = LocalResource::new(move || async move {
            let temp_identity = temp_identity_resource.await;
            let Some(id) = temp_identity else {
                return;
            };
            if let Err(e) = set_anonymous_identity_cookie(id).await {
                log::error!("Failed to set anonymous identity as cookie?! err {e}");
            }
        });

        let (referrer_cookie, set_referrer_cookie) =
            use_cookie_with_options::<Principal, FromToStringCodec>(
                REFERRER_COOKIE,
                UseCookieOptions::default()
                    .path("/")
                    .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
            );
        let referrer_query = use_query::<Referrer>();
        let referrer_principal = Signal::derive(move || {
            let referrer_query_val = referrer_query()
                .ok()
                .and_then(|r| Principal::from_text(r.user_refer).ok());

            let referrer_cookie_val = referrer_cookie.get_untracked();
            if let Some(ref_princ) = referrer_query_val {
                set_referrer_cookie(Some(ref_princ));
                Some(ref_princ)
            } else {
                referrer_cookie_val
            }
        });

        let is_logged_in_with_oauth = use_cookie_with_options::<bool, FromToStringCodec>(
            ACCOUNT_CONNECTED_STORE,
            UseCookieOptions::default()
                .path("/")
                .max_age(REFRESH_MAX_AGE.as_millis() as i64),
        );

        let new_identity_setter = RwSignal::new(None::<DelegatedIdentityWire>);

        let user_identity_resource = Resource::new(
            move || MockPartialEq(new_identity_setter()),
            move |auth_id| async move {
                let temp_identity = temp_identity_resource.await;

                if let Some(id_wire) = auth_id.0 {
                    return Ok::<_, ServerFnError>(id_wire);
                }

                let Some(jwk_key) = temp_identity else {
                    let id_wire = extract_identity()
                        .await?
                        .ok_or_else(|| ServerFnError::new("No refresh cookie set?!"))?;
                    return Ok(id_wire);
                };

                let key = k256::SecretKey::from_jwk(&jwk_key)?;
                let id = Secp256k1Identity::from_private_key(key);
                let id_wire = delegate_identity(&id);

                Ok(id_wire)
            },
        );

        let canisters_resource: AuthCansResource = Resource::new(
            move || {
                user_identity_resource.track();
                MockPartialEq(())
            },
            move |_| {
                send_wrap(async move {
                    let id_wire = user_identity_resource.await?;
                    let ref_principal = referrer_principal.get_untracked();

                    let res = do_canister_auth(id_wire, ref_principal).await?;

                    Ok::<_, ServerFnError>(res)
                })
            },
        );

        let user_principal_cookie = use_cookie_with_options::<Principal, FromToStringCodec>(
            USER_PRINCIPAL_STORE,
            UseCookieOptions::default()
                .path("/")
                .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
        );
        let user_principal = Resource::new(
            move || {
                user_identity_resource.track();
                MockPartialEq(())
            },
            move |_| async move {
                if let Some(princ) = user_principal_cookie.0.get_untracked() {
                    return Ok(princ);
                }

                let id_wire = user_identity_resource.await?;
                let princ = Principal::self_authenticating(&id_wire.from_key);
                user_principal_cookie.1.set(Some(princ));

                Ok(princ)
            },
        );

        let user_canister_id_cookie = use_cookie_with_options::<Principal, FromToStringCodec>(
            USER_CANISTER_ID_STORE,
            UseCookieOptions::default()
                .path("/")
                .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
        );
        let user_canister = Resource::new(
            move || {
                canisters_resource.track();
                MockPartialEq(())
            },
            move |_| async move {
                if let Some(canister_id) = user_canister_id_cookie.0.get_untracked() {
                    return Ok(canister_id);
                }

                let cans_wire = canisters_resource.await?;

                let canister_id = cans_wire.user_canister;
                user_canister_id_cookie.1.set(Some(canister_id));

                Ok(canister_id)
            },
        );

        let event_ctx = EventCtx {
            is_connected: StoredValue::new(Box::new(move || {
                is_logged_in_with_oauth
                    .0
                    .get_untracked()
                    .unwrap_or_default()
            })),
            user_details: StoredValue::new(Box::new(move || {
                canisters_resource
                    .into_future()
                    .now_or_never()
                    .and_then(|c| {
                        let cans_wire = c.ok()?;
                        Some(EventUserDetails {
                            details: cans_wire.profile_details.clone(),
                            canister_id: cans_wire.user_canister,
                        })
                    })
            })),
        };

        Self {
            _temp_identity_resource: temp_identity_resource,
            _temp_id_cookie_resource: temp_id_cookie_resource,
            referrer_store: referrer_principal,
            is_logged_in_with_oauth,
            new_identity_setter,
            canisters_resource,
            user_principal,
            user_principal_cookie,
            user_canister,
            user_canister_id_cookie,
            event_ctx,
        }
    }
}

impl AuthState {
    pub fn is_logged_in_with_oauth(&self) -> Signal<bool> {
        let logged_in = self.is_logged_in_with_oauth.0;
        Signal::derive(move || logged_in.get().unwrap_or_default())
    }

    pub fn set_new_identity(
        &self,
        new_identity: DelegatedIdentityWire,
        is_logged_in_with_oauth: bool,
    ) {
        self.is_logged_in_with_oauth
            .1
            .set(Some(is_logged_in_with_oauth));

        self.user_canister_id_cookie.1.set(None);
        self.user_principal_cookie
            .1
            .set(Some(Principal::self_authenticating(&new_identity.from_key)));
        self.new_identity_setter.set(Some(new_identity));
    }

    /// WARN: This function MUST be used with `<Suspense>`, if used inside view! {}
    /// this also tracks any changes made to user's identity, if used with <Suspend>
    pub async fn auth_cans(
        &self,
        base: Canisters<false>,
    ) -> Result<Canisters<true>, ServerFnError> {
        let cans_wire = self.canisters_resource.await?;
        let cans = Canisters::from_wire(cans_wire, base)?;
        Ok(cans)
    }

    /// WARN: This function MUST be used with `<Suspense>`, if used inside view! {}
    /// this also tracks any changes made to user's identity, if used with <Suspend>
    pub async fn cans_wire(&self) -> Result<CanistersAuthWire, ServerFnError> {
        let cans_wire = self.canisters_resource.await?;
        Ok(cans_wire)
    }

    /// Get the user principal if loaded
    /// does not have any tracking
    /// NOT RECOMMENDED TO BE USED IN DOM
    pub fn user_principal_if_available(&self) -> Option<Principal> {
        self.user_principal_cookie.0.get_untracked()
    }

    /// Get the user canister if loaded
    /// does not have any tracking
    /// NOT RECOMMENDED TO BE USED IN DOM
    pub fn user_canister_if_available(&self) -> Option<Principal> {
        self.user_canister_id_cookie.0.get_untracked()
    }

    /// WARN: Only use this for analytics
    // TODO: I really want to refactor events as a whole
    pub fn event_ctx(&self) -> EventCtx {
        self.event_ctx
    }

    /// Derive a new resource which uses the current user's canister
    /// WARN: The signals in tracker are not memoized
    pub fn derive_resource<
        S: Clone + Send + Sync + 'static,
        D: Send + Sync + Serialize + for<'x> Deserialize<'x>,
        DFut: Future<Output = Result<D, ServerFnError>> + 'static + Send,
    >(
        &self,
        tracker: impl Fn() -> S + Send + Sync + 'static,
        fetcher: impl Fn(Canisters<true>, S) -> DFut + Send + Sync + 'static + Clone,
    ) -> Resource<Result<D, ServerFnError>> {
        let cans = self.canisters_resource;
        let base = unauth_canisters();
        Resource::new(
            move || {
                // MockPartialEq is necessary
                // See: https://github.com/leptos-rs/leptos/issues/2661
                cans.track();
                MockPartialEq(tracker())
            },
            move |s| {
                let base = base.clone();
                let fetcher = fetcher.clone();
                async move {
                    let cans_wire = cans.await?;
                    let cans = Canisters::from_wire(cans_wire, base)?;
                    fetcher(cans, s.0).await
                }
            },
        )
    }

    /// WARN: Use this very carefully, this function only exists for very fine-tuned optimizations
    /// for critical pages
    /// this definitely must not be used in DOM
    pub fn auth_cans_if_available(&self, base: Canisters<false>) -> Option<Canisters<true>> {
        self.canisters_resource
            .into_future()
            .now_or_never()
            .and_then(|c| {
                let cans_wire = c.ok()?;
                Canisters::from_wire(cans_wire, base).ok()
            })
    }
}
