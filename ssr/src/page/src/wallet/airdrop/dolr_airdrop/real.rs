use anyhow::ensure;
use candid::Nat;
use candid::Principal;
use dolr_airdrop::db::DolrAirdrop;
use leptos::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sea_orm::sqlx::types::chrono::Utc;
use sea_orm::ActiveValue;
use sea_orm::DatabaseConnection;
use sea_orm::IntoActiveModel;
use sea_orm::QuerySelect;
use sea_orm::TransactionTrait;
use yral_canisters_client::individual_user_template::{Result7, SessionType};
use yral_canisters_common::Canisters;

use crate::wallet::airdrop::AirdropStatus;
use dolr_airdrop::entities::dolr_airdrop_data;
use sea_orm::prelude::*;

const DOLR_AIRDROP_LIMIT_DURATION: web_time::Duration = web_time::Duration::from_secs(24 * 3600);
/// in e0s
const DOLR_AIRDROP_AMOUNT_RANGE: std::ops::Range<u64> = 5..10;

/// returns either ok, or the how long after which airdrop will be available
async fn is_dolr_airdrop_available(
    _user_canister: Principal,
    user_principal: Principal,
    now: DateTimeUtc,
) -> anyhow::Result<Result<(), web_time::Duration>> {
    let DolrAirdrop(db) = expect_context();

    let Some(airdrop_data) = dolr_airdrop_data::Entity::find_by_id(user_principal.to_text())
        .lock_shared()
        .one(&db)
        .await?
    else {
        return Ok(Ok(()));
    };

    leptos::logging::debug_warn!("dolr airdrop data fetched: {airdrop_data:#?}");

    let next_airdrop_available_after =
        airdrop_data.last_airdrop_at.and_utc() + DOLR_AIRDROP_LIMIT_DURATION;

    if now < next_airdrop_available_after {
        let delta = next_airdrop_available_after.signed_duration_since(now);
        let secs = delta.num_seconds() as u64;
        let nanos = delta.subsec_nanos() as u32;
        let duration = web_time::Duration::new(secs, nanos);
        return Ok(Err(duration));
    }

    Ok(Ok(()))
}

#[server(input = server_fn::codec::Json)]
pub async fn is_user_eligible_for_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<AirdropStatus, ServerFnError> {
    let res = is_dolr_airdrop_available(user_canister, user_principal, Utc::now())
        .await
        .map_err(ServerFnError::new)?;

    match res {
        Ok(_) => Ok(AirdropStatus::Available),
        Err(duration) => Ok(AirdropStatus::WaitFor(duration)),
    }
}

#[cfg(not(feature = "backend-admin"))]
pub async fn send_airdrop_to_user(
    _user_principal: Principal,
    _amount: Nat,
) -> Result<(), ServerFnError> {
    log::error!("trying to send dolr but no backend admin is available");

    Err(ServerFnError::new("backend admin not available"))
}

#[cfg(feature = "backend-admin")]
pub async fn send_airdrop_to_user(
    user_principal: Principal,
    amount: Nat,
) -> Result<(), ServerFnError> {
    use consts::DOLR_AI_LEDGER_CANISTER;
    use state::admin_canisters::AdminCanisters;
    use yral_canisters_client::sns_ledger::{Account, SnsLedger, TransferResult};
    let admin: AdminCanisters = expect_context();

    let ledger = SnsLedger(
        DOLR_AI_LEDGER_CANISTER.parse().unwrap(),
        admin.get_agent().await,
    );

    let res = ledger
        .icrc_1_transfer(yral_canisters_client::sns_ledger::TransferArg {
            to: Account {
                owner: user_principal,
                subaccount: None,
            },
            fee: None,
            memo: Some(Vec::from("dolr airdrop").into()),
            from_subaccount: None,
            created_at_time: None,
            amount,
        })
        .await?;

    if let TransferResult::Err(err) = res {
        return Err(ServerFnError::new(format!("transfer failed: {err:?}")));
    }

    Ok(())
}

async fn mark_airdrop_claimed(
    db: &DatabaseConnection,
    user_principal: Principal,
    now: ChronoDateTimeUtc,
) -> anyhow::Result<()> {
    db.transaction::<_, _, anyhow::Error>(|txn| {
        Box::pin(async move {
            let Some(airdrop_data) =
                dolr_airdrop_data::Entity::find_by_id(user_principal.to_text())
                    .lock_with_behavior(
                        sea_orm::sea_query::LockType::Update,
                        sea_orm::sea_query::LockBehavior::Nowait,
                    )
                    .one(txn)
                    .await?
            else {
                let airdrop_data = dolr_airdrop_data::ActiveModel {
                    user_principal: ActiveValue::Set(user_principal.to_text()),
                    last_airdrop_at: ActiveValue::Set(now.naive_utc()),
                };

                dolr_airdrop_data::Entity::insert(airdrop_data)
                    .exec_without_returning(txn)
                    .await?;

                return Ok(());
            };

            let next_airdrop_available_after =
                airdrop_data.last_airdrop_at.and_utc() + DOLR_AIRDROP_LIMIT_DURATION;

            ensure!(
                now >= next_airdrop_available_after,
                "Airdrop is not allowed yet"
            );

            let mut airdrop_data = airdrop_data.into_active_model();

            airdrop_data
                .last_airdrop_at
                .set_if_not_equals(now.naive_utc());

            dolr_airdrop_data::Entity::update(airdrop_data)
                .exec(txn)
                .await?;

            Ok(())
        })
    })
    .await?;

    Ok(())
}

#[server(input = server_fn::codec::Json)]
pub async fn claim_dolr_airdrop(
    user_canister: Principal,
    user_principal: Principal,
) -> Result<u64, ServerFnError> {
    let cans: Canisters<false> = expect_context();
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

    let sess = user.get_session_type().await?;
    if !matches!(sess, Result7::Ok(SessionType::RegisteredSession)) {
        return Err(ServerFnError::new("Not allowed to claim: not logged in"));
    }

    let now = Utc::now();
    if is_dolr_airdrop_available(user_canister, user_principal, now)
        .await
        .map_err(ServerFnError::new)?
        .is_err()
    {
        return Err(ServerFnError::new(
            "Not allowed to claim: max claims reached within allowed duration",
        ));
    }

    let DolrAirdrop(db) = expect_context();
    mark_airdrop_claimed(&db, user_principal, now)
        .await
        .map_err(ServerFnError::new)?;

    let mut rng = SmallRng::from_os_rng();
    let amount = rng.random_range(DOLR_AIRDROP_AMOUNT_RANGE);
    let e8s_amount: Nat = Nat::from(amount) * (1e8 as usize);
    // sending money _after_ marking claim with reasoning "a couple unhappy users
    // are better than company losing money"
    send_airdrop_to_user(user_principal, e8s_amount).await?;

    Ok(amount)
}
