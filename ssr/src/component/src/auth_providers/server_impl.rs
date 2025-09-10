#[cfg(feature = "backend-admin")]
use backend_admin::*;
use candid::Principal;
use hon_worker_common::ReferralReqWithSignature;
use leptos::prelude::*;
#[cfg(not(feature = "backend-admin"))]
use no_op::*;
use state::canisters::unauth_canisters;

pub async fn issue_referral_rewards(
    worker_req: ReferralReqWithSignature,
) -> Result<(), ServerFnError> {
    ensure_user_logged_in_with_oauth(worker_req.request.referee).await?;

    issue_referral_rewards_impl(worker_req).await
}

pub async fn mark_user_registered(user_principal: Principal) -> Result<bool, ServerFnError> {
    ensure_user_logged_in_with_oauth(user_principal).await?;

    let unauth_canisters = unauth_canisters();

    let user_canister = unauth_canisters
        .get_individual_canister_v2(user_principal.to_text())
        .await?
        .ok_or(ServerFnError::new("user canister not found"))?;

    mark_user_registered_impl(user_principal, user_canister).await
}

async fn ensure_user_logged_in_with_oauth(user_principal: Principal) -> Result<(), ServerFnError> {
    #[cfg(feature = "oauth-ssr")]
    {
        use std::env;

        use auth::server_impl::yral::YralAuthRefreshTokenClaims;
        use axum_extra::extract::{cookie::Key, SignedCookieJar};
        use consts::{
            auth::REFRESH_TOKEN_COOKIE,
            yral_auth::{YRAL_AUTH_CLIENT_ID_ENV, YRAL_AUTH_ISSUER_URL, YRAL_AUTH_TRUSTED_KEY},
        };
        use jsonwebtoken::Validation;
        use leptos_axum::extract_with_state;

        let key: Key = expect_context();
        let jar: SignedCookieJar = extract_with_state(&key).await?;

        let Some(refresh_token) = jar.get(REFRESH_TOKEN_COOKIE) else {
            return Err(ServerFnError::new("not logged in"));
        };

        let client_id = env::var(YRAL_AUTH_CLIENT_ID_ENV).expect("expected to have client id");

        let mut token_validation = Validation::new(jsonwebtoken::Algorithm::ES256);
        token_validation.set_audience(&[client_id]);
        token_validation.set_issuer(&[YRAL_AUTH_ISSUER_URL]);

        let decoded = jsonwebtoken::decode::<YralAuthRefreshTokenClaims>(
            refresh_token.value(),
            &YRAL_AUTH_TRUSTED_KEY,
            &token_validation,
        )?;
        if decoded.claims.ext_is_anonymous || decoded.claims.sub != user_principal {
            Err(ServerFnError::new("not logged in"))
        } else {
            Ok(())
        }
    }
    #[cfg(not(feature = "oauth-ssr"))]
    {
        _ = user_principal;
        Err(ServerFnError::new("not logged in"))
    }
}

#[cfg(feature = "backend-admin")]
mod backend_admin {
    use candid::Principal;
    use hon_worker_common::ReferralReqWithSignature;
    use hon_worker_common::WORKER_URL;
    use leptos::prelude::*;
    use state::server::HonWorkerJwt;
    use yral_canisters_client::ic::USER_INFO_SERVICE_ID;
    use yral_canisters_client::individual_user_template::{Result15, Result7};
    use yral_canisters_client::user_info_service::Result2;
    use yral_canisters_client::user_info_service::Result_;

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
        user_principal: Principal,
        user_canister: Principal,
    ) -> Result<bool, ServerFnError> {
        use state::admin_canisters::admin_canisters;
        use yral_canisters_client::individual_user_template::SessionType;
        use yral_canisters_client::user_info_service::SessionType as UserServiceSessionType;

        let admin_cans = admin_canisters();

        if user_canister == USER_INFO_SERVICE_ID {
            let user_service = admin_cans.user_info_service().await;
            if matches!(
                user_service.get_user_session_type(user_principal).await?,
                Result2::Ok(UserServiceSessionType::RegisteredSession)
            ) {
                return Ok(false);
            }
            user_service
                .update_session_type(user_principal, UserServiceSessionType::RegisteredSession)
                .await
                .map_err(ServerFnError::from)
                .and_then(|res| match res {
                    Result_::Ok => Ok(()),
                    Result_::Err(e) => Err(ServerFnError::new(format!(
                        "failed to mark user as registered {e}"
                    ))),
                })?;

            Ok(true)
        } else {
            let user = admin_cans.individual_user_for(user_canister).await;
            if matches!(
                user.get_session_type().await?,
                Result7::Ok(SessionType::RegisteredSession)
            ) {
                return Ok(false);
            }
            user.update_session_type(SessionType::RegisteredSession)
                .await
                .map_err(ServerFnError::from)
                .and_then(|res| match res {
                    Result15::Ok(_) => Ok(()),
                    Result15::Err(e) => Err(ServerFnError::new(format!(
                        "failed to mark user as registered {e}"
                    ))),
                })?;
            Ok(true)
        }
    }
}

#[cfg(not(feature = "backend-admin"))]
mod no_op {
    use candid::Principal;
    use hon_worker_common::ReferralReqWithSignature;
    use leptos::prelude::ServerFnError;
    pub async fn issue_referral_rewards_impl(
        _worker_req: ReferralReqWithSignature,
    ) -> Result<(), ServerFnError> {
        Ok(())
    }

    pub async fn mark_user_registered_impl(
        _user_principal: Principal,
        _user_canister: Principal,
    ) -> Result<bool, ServerFnError> {
        Ok(true)
    }
}
