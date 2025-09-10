use std::future::{Future, IntoFuture};

use auth::{
    extract_identity, generate_anonymous_identity_if_required, set_anonymous_identity_cookie,
    AnonymousIdentity,
};
use candid::Principal;
use codee::string::FromToStringCodec;
use consts::{
    auth::REFRESH_MAX_AGE, ACCOUNT_CONNECTED_STORE, AUTH_UTIL_COOKIES_MAX_AGE_MS, REFERRER_COOKIE,
    USER_CANISTER_ID_STORE, USER_PRINCIPAL_STORE,
};
use futures::FutureExt;
use global_constants::USERNAME_MAX_LEN;
use leptos::prelude::*;
use leptos_router::{hooks::use_query, params::Params};
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use rand::{distr::Alphanumeric, rngs::SmallRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use yral_canisters_common::{utils::time::current_epoch, Canisters};

use utils::{
    event_streaming::events::{EventCtx, EventUserDetails},
    send_wrap,
    types::NewIdentity,
    MockPartialEq,
};
use yral_types::delegated_identity::DelegatedIdentityWire;

pub fn unauth_canisters() -> Canisters<false> {
    expect_context()
}

async fn set_fallback_username(cans: &mut Canisters<true>, mut username: String) {
    let mut rng: Option<SmallRng> = None;
    while let Err(e) = cans.set_username(username.clone()).await {
        use yral_canisters_common::Error::*;
        use yral_metadata_client::Error as MetadataError;
        use yral_metadata_types::error::ApiError;

        match e {
            Metadata(MetadataError::Api(ApiError::DuplicateUsername)) => {
                let remaining_chars = username.len().saturating_sub(USERNAME_MAX_LEN);
                if remaining_chars == 0 {
                    break;
                }
                let rng = rng.get_or_insert_with(|| {
                    let mut seed = [0u8; 32];
                    seed[..16].copy_from_slice(&current_epoch().as_nanos().to_be_bytes());

                    SmallRng::from_seed(seed)
                });
                // append random characters in chunks of 3
                let rand_chars = rng
                    .sample_iter(Alphanumeric)
                    .map(|c| c as char)
                    .take(remaining_chars.min(3));
                username.extend(rand_chars);
            }
            e => log::warn!("failed to set fallback username, err: {e}, ignoring"),
        }
    }
}

async fn do_canister_auth(
    auth: DelegatedIdentityWire,
    _referrer: Option<Principal>,
    fallback_username: Option<String>,
) -> Result<Canisters<true>, ServerFnError> {
    let auth_fut = Canisters::authenticate_with_network(auth);
    let mut canisters = send_wrap(auth_fut).await?;

    leptos::logging::log!(
        "registered new user with principal {}",
        canisters.user_principal().to_text()
    );

    if canisters.profile_details().username.is_some() {
        return Ok(canisters);
    }
    let Some(username) = fallback_username else {
        return Ok(canisters);
    };

    set_fallback_username(&mut canisters, username).await;

    Ok(canisters)
}
type AuthCansResource = LocalResource<Result<Canisters<true>, ServerFnError>>;

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
    _temp_identity_resource: OnceResource<Option<AnonymousIdentity>>,
    _temp_id_cookie_resource: LocalResource<()>,
    pub referrer_store: Signal<Option<Principal>>,
    is_logged_in_with_oauth: (Signal<Option<bool>>, WriteSignal<Option<bool>>),
    new_identity_setter: RwSignal<Option<NewIdentity>>,
    pub canisters_resource: AuthCansResource,
    user_canister_id_cookie: (Signal<Option<Principal>>, WriteSignal<Option<Principal>>),
    pub user_principal: Resource<Result<Principal, ServerFnError>>,
    user_principal_cookie: (Signal<Option<Principal>>, WriteSignal<Option<Principal>>),
    event_ctx: EventCtx,
    pub user_identity: Resource<Result<NewIdentity, ServerFnError>>,
    new_cans_setter: RwSignal<Option<Canisters<true>>>,
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
            let Some(temp_identity) = temp_identity_resource.await else {
                return;
            };
            if let Err(e) = set_anonymous_identity_cookie(temp_identity.refresh_token).await {
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

        let new_identity_setter = RwSignal::new(None::<NewIdentity>);

        let user_identity_resource = Resource::new(
            move || MockPartialEq(new_identity_setter()),
            move |auth_id| async move {
                let temp_identity = temp_identity_resource.await;

                if let Some(id_wire) = auth_id.0 {
                    return Ok::<_, ServerFnError>(id_wire);
                }

                let Some(id) = temp_identity else {
                    let id_wire = match extract_identity().await {
                        Ok(Some(identity)) => identity,
                        Ok(None) => return Err(ServerFnError::new("No refresh cookie set?!")),
                        Err(e) => {
                            return Err(ServerFnError::new(e.to_string()));
                        }
                    };
                    return Ok(NewIdentity {
                        id_wire,
                        fallback_username: None,
                        email: None,
                    });
                };

                Ok(NewIdentity {
                    id_wire: id.identity,
                    fallback_username: None,
                    email: None,
                })
            },
        );

        let new_cans_setter = RwSignal::new(None::<Canisters<true>>);

        let canisters_resource: AuthCansResource = LocalResource::new(move || {
            user_identity_resource.track();
            let new_cans = new_cans_setter();
            async move {
                let new_id = user_identity_resource.await?;
                match new_cans {
                    Some(cans)
                        if cans.user_principal()
                            == Principal::self_authenticating(&new_id.id_wire.from_key) =>
                    {
                        return Ok::<_, ServerFnError>(cans);
                    }
                    // this means that the user did the following:
                    // 1. Changed their username, then
                    // 2. Logged in with oauth (or logged out)
                    _ => {}
                };
                let ref_principal = referrer_principal.get_untracked();

                let res = do_canister_auth(new_id.id_wire, ref_principal, new_id.fallback_username)
                    .await?;

                Ok::<_, ServerFnError>(res)
            }
        });

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
                if let Some(princ) = user_principal_cookie.0.try_get_untracked().flatten() {
                    return Ok(princ);
                }

                let id_wire = user_identity_resource.await?;
                let princ = Principal::self_authenticating(&id_wire.id_wire.from_key);
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

        let event_ctx = EventCtx {
            is_connected: StoredValue::new(Box::new(move || {
                is_logged_in_with_oauth
                    .0
                    .get_untracked()
                    .unwrap_or_default()
            })),
            user_details: StoredValue::new(Box::new(move || {
                #[cfg(not(feature = "hydrate"))]
                {
                    None
                }

                #[cfg(feature = "hydrate")]
                canisters_resource
                    .into_future()
                    .now_or_never()
                    .and_then(|c| {
                        let cans = c.ok()?;
                        Some(EventUserDetails {
                            details: cans.profile_details(),
                            canister_id: cans.user_canister(),
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
            user_canister_id_cookie,
            event_ctx,
            user_identity: user_identity_resource,
            new_cans_setter,
        }
    }
}

impl AuthState {
    pub fn is_logged_in_with_oauth(&self) -> Signal<bool> {
        let logged_in = self.is_logged_in_with_oauth.0;
        Signal::derive(move || logged_in.get().unwrap_or_default())
    }

    /// Updates the identity
    /// fallback_username will be the username of this id
    /// if not already set
    pub fn set_new_identity(&self, new_identity: NewIdentity, is_logged_in_with_oauth: bool) {
        self.is_logged_in_with_oauth
            .1
            .set(Some(is_logged_in_with_oauth));

        self.user_canister_id_cookie.1.set(None);
        self.user_principal_cookie
            .1
            .set(Some(Principal::self_authenticating(
                &new_identity.id_wire.from_key,
            )));
        self.new_identity_setter.set(Some(new_identity));
    }

    pub async fn set_new_identity_and_wait_for_authentication(
        &self,
        new_identity: NewIdentity,
        is_logged_in_with_oauth: bool,
    ) -> Result<Canisters<true>, ServerFnError> {
        self.set_new_identity(new_identity, is_logged_in_with_oauth);
        self.canisters_resource.ready().await;

        self.auth_cans().await
    }

    /// WARN: This function MUST be used with `<Suspense>`, if used inside view! {}
    /// this also tracks any changes made to user's identity, if used with <Suspend>
    pub async fn auth_cans(&self) -> Result<Canisters<true>, ServerFnError> {
        self.canisters_resource.await
    }

    /// Get the user principal if loaded
    /// does not have any tracking
    /// NOT RECOMMENDED TO BE USED IN DOM
    pub fn user_principal_if_available(&self) -> Option<Principal> {
        self.user_principal_cookie.0.get_untracked()
    }

    /// WARN: This function MUST be used with `<Suspense>`, if used inside view! {}
    /// this also tracks any changes made to user's identity, if used with <Suspend>
    pub async fn user_canister(&self) -> Result<Principal, ServerFnError> {
        if let Some(canister_id) = self.user_canister_id_cookie.0.get_untracked() {
            return Ok(canister_id);
        }

        let cans_wire = self.canisters_resource.await?;

        let canister_id = cans_wire.user_canister();
        self.user_canister_id_cookie.1.set(Some(canister_id));

        Ok(canister_id)
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

    pub fn derive_resource<
        S: Clone + 'static,
        D: Serialize + for<'x> Deserialize<'x> + 'static,
        DFut: Future<Output = Result<D, ServerFnError>> + 'static,
    >(
        &self,
        tracker: impl Fn() -> S + 'static,
        fetcher: impl Fn(Canisters<true>, S) -> DFut + 'static + Clone,
    ) -> LocalResource<Result<D, ServerFnError>> {
        let cans = self.canisters_resource;
        LocalResource::new(move || {
            cans.track();
            let state = tracker();
            let fetcher = fetcher.clone();
            async move {
                let cans = cans.await?;
                fetcher(cans, state).await
            }
        })
    }

    /// WARN: Use this very carefully, this function only exists for very fine-tuned optimizations
    /// for critical pages
    /// this definitely must not be used in DOM
    /// this always be `None` for ssr
    pub fn auth_cans_if_available(&self) -> Option<Canisters<true>> {
        #[cfg(not(feature = "hydrate"))]
        {
            None
        }

        #[cfg(feature = "hydrate")]
        self.canisters_resource
            .into_future()
            .now_or_never()
            .and_then(|c| c.ok())
    }

    /// Update the username of the user
    /// WARN: all subscribers to the canisters resource will be notified
    pub async fn update_username(
        &self,
        mut cans: Canisters<true>,
        new_username: String,
    ) -> yral_canisters_common::Result<()> {
        cans.set_username(new_username).await?;
        self.new_cans_setter.set(Some(cans));

        Ok(())
    }
}
