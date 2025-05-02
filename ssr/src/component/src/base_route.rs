use candid::Principal;
use ic_agent::identity::Secp256k1Identity;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_query;
use leptos_use::use_cookie;

use auth::delegate_identity;
use auth::{
    extract_identity, generate_anonymous_identity_if_required, set_anonymous_identity_cookie,
};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use consts::{ACCOUNT_CONNECTED_STORE, USER_CANISTER_ID_STORE, USER_PRINCIPAL_STORE};
use leptos_router::params::Params;
use leptos_use::storage::use_local_storage;
use state::{
    auth::AuthState,
    canisters::{do_canister_auth, AuthCansResource},
    local_storage::use_referrer_store,
};
use utils::event_streaming::events::PageVisit;
use utils::send_wrap;
use utils::{try_or_redirect, MockPartialEq};
use yral_canisters_common::Canisters;

#[derive(Params, PartialEq, Clone)]
struct Referrer {
    user_refer: String,
}

#[component]
fn CtxProvider(children: ChildrenFn) -> impl IntoView {
    let auth = AuthState::default();
    provide_context(auth);

    let canisters_store = RwSignal::new(None::<Canisters<true>>);
    provide_context(canisters_store);

    let temp_identity_res = OnceResource::new(async move {
        generate_anonymous_identity_if_required()
            .await
            .expect("Failed to generate anonymous identity?!")
    });

    let temp_identity_c = temp_identity_res;
    LocalResource::new(move || async move {
        let temp_identity = temp_identity_c.await;
        let Some(id) = temp_identity else {
            return;
        };
        if let Err(e) = set_anonymous_identity_cookie(id).await {
            log::error!("Failed to set anonymous identity as cookie?! err {e}");
        }
    });

    let referrer_query = use_query::<Referrer>();
    let referrer_principal = Signal::derive(move || {
        referrer_query.get()
            .ok()
            .and_then(|r| Principal::from_text(r.user_refer).ok())
    });
    let (referrer_store, set_referrer_store, _) = use_referrer_store();
    Effect::new(move |_| {
        if referrer_store.get().is_some() {
            return;
        }
        set_referrer_store(referrer_principal.get())
    });

    // We need to perform this cleanup in case the user's cookie expired
    let (_, set_logged_in, _) =
        use_local_storage::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
    let (_, set_user_canister_id, _) =
        use_local_storage::<Option<Principal>, JsonSerdeCodec>(USER_CANISTER_ID_STORE);
    let (_, set_user_principal) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);

    let temp_identity_c = temp_identity_res;
    let canisters_res: AuthCansResource = Resource::new(
        move || MockPartialEq(auth()),
        move |auth_id| {
            send_wrap(async move {
                let temp_identity = temp_identity_c.await;
                let ref_principal = referrer_principal.get_untracked();

                if let Some(id_wire) = auth_id.0 {
                    return do_canister_auth(id_wire, ref_principal).await;
                }

                let Some(jwk_key) = temp_identity else {
                    let id_wire = extract_identity().await?.expect("No refresh cookie set?!");
                    return do_canister_auth(id_wire, ref_principal).await;
                };

                let key = k256::SecretKey::from_jwk(&jwk_key)?;
                let id = Secp256k1Identity::from_private_key(key);
                let id_wire = delegate_identity(&id);

                do_canister_auth(id_wire, ref_principal).await
            })
        },
    );
    provide_context::<AuthCansResource>(canisters_res);

    let location = leptos_router::hooks::use_location();

    view! {
        {children()}
        <Suspense>
            {move || Suspend::new(async move {
                let temp_id = temp_identity_res.await;
                let res = canisters_res.await;
                let cans_wire = try_or_redirect!(res);
                let maybe_cans = Canisters::from_wire(cans_wire, expect_context());
                let cans = try_or_redirect!(maybe_cans);
                let user_canister = cans.user_canister();
                let user_principal = cans.user_principal();

                Effect::new(move |_| {
                    if temp_id.is_some() {
                        set_logged_in(false);
                        let _ = js_sys::eval(
                            r#"
                            window.Sentry &&
                                        Sentry.onLoad(function () {{
                                               Sentry.setUser(null);
                                               Sentry.setTag("user_canister", null);
                                        }});
                            "#,
                        );
                    }
                    set_user_canister_id(Some(user_canister));
                    set_user_principal(Some(user_principal));
                    let user_principal_str = user_principal.to_string();
                    let user_canister_str = user_canister.to_string();
                    let _ = js_sys::eval(&format!(
                        r#"
                        window.Sentry &&
                                    Sentry.onLoad(function () {{
                                           Sentry.setUser({{
                                                id: "{user_principal_str}",
                                           }});

                                           Sentry.setTag("user_canister", "{user_canister_str}");
                                    }});
                        "#,
                    ));
                });

                // We need to perform this cleanup in case the user's cookie expired
                // Cleanup doent work, it sets it but simulatanously removes it lols
                // Effect::new(move |_| {
                //     if temp_id.is_some() {
                //         log::debug!("Removing user principal");
                //         set_logged_in(false);
                //         set_user_canister_id(None);
                //         set_user_principal(None);
                //     }
                // });

                canisters_store.set(Some(cans.clone()));
                Effect::new(move |_| {
                    let pathname = location.pathname.get();
                    let cans = cans.clone();
                    PageVisit.send_event(cans, pathname);
                });
            })}

        </Suspense>
    }
}

#[component]
pub fn BaseRoute() -> impl IntoView {
    view! {
        <CtxProvider>
            <Outlet/>
        </CtxProvider>
    }
}
