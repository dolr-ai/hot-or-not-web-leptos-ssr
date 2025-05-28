use codee::string::{FromToStringCodec, JsonSerdeCodec};
use consts::NOTIFICATIONS_ENABLED_STORE;
use ic_agent::identity::Secp256k1Identity;
use k256::elliptic_curve::JwkEcKey;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;

use yral_types::delegated_identity::DelegatedIdentityWire;

use super::{LoginProvButton, LoginProvCtx, ProviderKind};

const IDENTITY_JWK_STORE: &str = "id-jwk-insecure";

#[server]
async fn perform_local_storage_auth(
    secp256k1_key: Option<JwkEcKey>,
) -> Result<(DelegatedIdentityWire, JwkEcKey), ServerFnError> {
    use auth::server_impl::{
        store::KVStoreImpl, try_extract_identity, update_user_identity_and_delegate,
    };
    use axum_extra::extract::{cookie::Key, SignedCookieJar};
    use leptos_axum::{extract_with_state, ResponseOptions};

    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    let kv: KVStoreImpl = expect_context();
    let base_key = if let Some(id) = secp256k1_key.as_ref() {
        k256::SecretKey::from_jwk(id)?
    } else {
        try_extract_identity(&jar, &kv).await?.unwrap()
    };
    let jwk = secp256k1_key.unwrap_or_else(|| base_key.to_jwk());
    let base_identity = Secp256k1Identity::from_private_key(base_key);

    let resp: ResponseOptions = expect_context();
    let delegated = update_user_identity_and_delegate(&resp, jar, base_identity)?;
    Ok((delegated, jwk))
}

#[component]
pub fn LocalStorageProvider() -> impl IntoView {
    let (jwk_identity, set_jwk_identity, _) =
        use_local_storage::<Option<JwkEcKey>, JsonSerdeCodec>(IDENTITY_JWK_STORE);

    let ctx: LoginProvCtx = expect_context();
    let (_, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let do_login_action = Action::new(move |()| async move {
        let secp256k1_key = jwk_identity.get_untracked();
        let (delegation, jwk) = perform_local_storage_auth(secp256k1_key).await?;
        set_jwk_identity(Some(jwk));
        set_notifs_enabled(false);
        ctx.login_complete.set(delegation);
        Ok::<_, ServerFnError>(())
    });

    view! {
        <LoginProvButton
            prov=ProviderKind::LocalStorage
            class="rounded-full bg-neutral-700 p-4"
            on_click=move |ev| {
                ev.stop_propagation();
                do_login_action.dispatch(());
            }
        >

            <span class="text-white">Local Storage</span>
            <span class="text-red-600">(insecure)</span>
        </LoginProvButton>
    }
}
