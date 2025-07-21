use std::time::Duration;

use anyhow::ensure;
use candid::Principal;
use hon_worker_common::{ClaimRequest, SatsBalanceUpdateRequestV2, WORKER_URL};
use leptos::prelude::*;
use global_constants::{MAX_BET_AMOUNT_SATS, SATS_AIRDROP_LIMIT_RANGE_SATS};
use num_bigint::BigUint;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use reqwest::Url;
use sats_airdrop::{db::SatsAirdrop, entities::sats_airdrop_data};
use sea_orm::{
    prelude::DateTimeUtc, sqlx::types::chrono::Utc, ActiveValue, EntityTrait, IntoActiveModel,
    QuerySelect, TransactionTrait,
};
use state::server::HonWorkerJwt;
use yral_canisters_client::individual_user_template::{Result7, SessionType};
use yral_canisters_common::{utils::token::load_sats_balance, Canisters};
use yral_identity::Signature;

async fn is_airdrop_claimed(user_principal: Principal, now: DateTimeUtc) -> anyhow::Result<bool> {
    let SatsAirdrop(db) = expect_context();

    let Some(airdrop_data) = sats_airdrop_data::Entity::find_by_id(user_principal.to_text())
        .lock_shared()
        .one(&db)
        .await?
    else {
        return Ok(false);
    };

    let next_airdrop_available_after =
        airdrop_data.last_airdrop_at.and_utc() + Duration::from_secs(24 * 3600);

    Ok(now < next_airdrop_available_after)
}

async fn mark_airdrop_claimed(
    user_principal: Principal,
    amount: u32,
    now: DateTimeUtc,
) -> anyhow::Result<()> {
    let SatsAirdrop(db) = expect_context();
    let amount = amount as i64;
    db.transaction::<_, _, anyhow::Error>(|txn| {
        Box::pin(async move {
            let Some(airdrop_data) =
                sats_airdrop_data::Entity::find_by_id(user_principal.to_text())
                    .lock_with_behavior(
                        sea_orm::sea_query::LockType::Update,
                        sea_orm::sea_query::LockBehavior::Nowait,
                    )
                    .one(txn)
                    .await?
            else {
                let airdrop_data = sats_airdrop_data::ActiveModel {
                    user_principal: ActiveValue::Set(user_principal.to_text()),
                    last_airdrop_at: ActiveValue::Set(now.naive_utc()),
                    total_sats: ActiveValue::Set(amount),
                };

                sats_airdrop_data::Entity::insert(airdrop_data)
                    .exec_without_returning(txn)
                    .await?;

                return Ok(());
            };

            let next_airdrop_available_after =
                airdrop_data.last_airdrop_at.and_utc() + Duration::from_secs(24 * 3600);

            ensure!(
                now >= next_airdrop_available_after,
                "Airdrop is not allowed yet"
            );

            let mut airdrop_data = airdrop_data.into_active_model();

            airdrop_data
                .last_airdrop_at
                .set_if_not_equals(now.naive_utc());

            airdrop_data
                .total_sats
                .set_if_not_equals(amount + airdrop_data.total_sats.clone().unwrap());

            sats_airdrop_data::Entity::update(airdrop_data)
                .exec(txn)
                .await?;

            Ok(())
        })
    })
    .await?;

    Ok(())
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_sats_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<bool, ServerFnError> {
    let now = Utc::now();
    let balance = load_sats_balance(user_principal).await?.balance;
    let res = validate_sats_airdrop_eligibility(user_canister, user_principal, now, &balance).await;

    match res {
        Ok(_) => Ok(true),
        Err(ServerFnError::ServerError(..)) => Ok(false),
        Err(err) => Err(err),
    }
}

#[server(input = server_fn::codec::Json)]
pub async fn claim_sats_airdrop(
    user_canister: Principal,
    request: ClaimRequest,
    _signature: Signature,
) -> Result<u64, ServerFnError> {
    let now = Utc::now();
    let cans: Canisters<false> = expect_context();
    let user_principal = request.user_principal;
    let user = cans.individual_user(user_canister).await;
    let profile_owner = user.get_profile_details_v_2().await?;
    if profile_owner.principal_id != user_principal {
        // ideally should never happen unless its a hacking attempt
        println!(
            "Not allowed to claim due to principal mismatch: owner={} != receiver={user_principal}",
            profile_owner.principal_id,
        );
        return Err(ServerFnError::new(
            "Not allowed to claim: principal mismatch",
        ));
    }
    let balance = load_sats_balance(user_principal).await?.balance;
    validate_sats_airdrop_eligibility(user_canister, user_principal, now, &balance).await?;
    let mut rng = SmallRng::from_os_rng();
    let amount = rng.random_range(SATS_AIRDROP_LIMIT_RANGE_SATS);
    let worker_req = SatsBalanceUpdateRequestV2 {
        previous_balance: balance,
        delta: amount.into(),
        is_airdropped: true,
    };

    mark_airdrop_claimed(user_principal, amount as u32, now)
        .await
        .map_err(ServerFnError::new)?;

    let req_url: Url = WORKER_URL.parse().expect("url to be valid");
    let req_url = req_url
        .join(&format!("/v2/update_balance/{user_principal}"))
        .expect("url to be valid");
    let client = reqwest::Client::new();
    let jwt = expect_context::<HonWorkerJwt>();
    let res = client
        .post(req_url)
        .json(&worker_req)
        .header("Authorization", format!("Bearer {}", jwt.0))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "worker error[{}]: {}",
            res.status().as_u16(),
            res.text().await?
        )));
    }

    Ok(amount)
}

async fn validate_sats_airdrop_eligibility(
    user_canister: Principal,
    user_principal: Principal,
    now: DateTimeUtc,
    balance: &BigUint,
) -> Result<(), ServerFnError> {
    let cans = Canisters::default();
    let user = cans.individual_user(user_canister).await;

    if balance.ge(&MAX_BET_AMOUNT_SATS.into()) {
        return Err(ServerFnError::new(
            "Not allowed to claim: balance >= max bet amount",
        ));
    }
    let sess = user.get_session_type().await?;
    if !matches!(sess, Result7::Ok(SessionType::RegisteredSession)) {
        return Err(ServerFnError::new("Not allowed to claim: not logged in"));
    }
    let is_airdrop_claimed = is_airdrop_claimed(user_principal, now)
        .await
        .map_err(ServerFnError::new)?;
    if is_airdrop_claimed {
        return Err(ServerFnError::new("Not allowed to claim: already claimed"));
    }

    Ok(())
}
