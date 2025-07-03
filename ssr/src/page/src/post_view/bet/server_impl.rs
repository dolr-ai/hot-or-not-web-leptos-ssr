use candid::Principal;
use hon_worker_common::{VoteRequest, VoteResV2};
use leptos::prelude::*;
use yral_identity::Signature;

#[server(endpoint = "vote", input = server_fn::codec::Json)]
pub async fn vote_with_cents_on_post(
    sender: Principal,
    req: VoteRequest,
    sig: Signature,
    prev_video_info: Option<(Principal, u64)>,
) -> Result<VoteResV2, ServerFnError> {
    #[cfg(feature = "alloydb")]
    use alloydb::vote_with_cents_on_post;
    #[cfg(not(feature = "alloydb"))]
    use mock::vote_with_cents_on_post;

    // validate request against limits

    use limits::MAX_BET_AMOUNT;
    if req.vote_amount > MAX_BET_AMOUNT as u128 {
        return Err(ServerFnError::new(format!(
            "bet amount exceeds maximum allowed: {} > {}",
            req.vote_amount, MAX_BET_AMOUNT
        )));
    }

    vote_with_cents_on_post(sender, req, sig, prev_video_info).await
}

#[cfg(feature = "alloydb")]
mod alloydb {
    use super::*;
    use hon_worker_common::WORKER_URL;
    use hon_worker_common::{HoNGameVoteReq, HotOrNot, VoteRequest, VoteResV2};

    pub async fn vote_with_cents_on_post(
        sender: Principal,
        req: VoteRequest,
        sig: Signature,
        prev_video_info: Option<(Principal, u64)>,
    ) -> Result<VoteResV2, ServerFnError> {
        use state::alloydb::AlloyDbInstance;
        use state::server::HonWorkerJwt;
        use yral_canisters_common::Canisters;

        let cans: Canisters<false> = expect_context();
        let Some(post_info) = cans
            .get_post_details(req.post_canister, req.post_id)
            .await?
        else {
            return Err(ServerFnError::new("post not found"));
        };
        let prev_uid_formatted = if let Some((canister_id, post_id)) = prev_video_info {
            let details = cans
                .get_post_details(canister_id, post_id)
                .await?
                .ok_or_else(|| ServerFnError::new("previous post not found"))?;
            format!("'{}'", details.uid)
        } else {
            "NULL".to_string()
        };

        // sanitization is not required here, as get_post_details verifies that the post is valid
        // and exists on cloudflare
        let query = format!(
            "select hot_or_not_evaluator.compare_videos_hot_or_not('{}', {})",
            post_info.uid, prev_uid_formatted,
        );

        let alloydb: AlloyDbInstance = expect_context();
        let mut res = alloydb.execute_sql_raw(query).await?;
        let mut res = res
            .sql_results
            .pop()
            .expect("hot_or_not_evaluator.compare_videos_hot_or_not MUST return a result");
        let mut res = res
            .rows
            .pop()
            .expect("hot_or_not_evaluator.compare_videos_hot_or_not MUST return a row");
        let res = res
            .values
            .pop()
            .expect("hot_or_not_evaluator.compare_videos_hot_or_not MUST return a value");

        let res = res.value.clone().map(|v| v.to_uppercase());
        let sentiment = match res.as_deref() {
            Some("TRUE") => HotOrNot::Hot,
            Some("FALSE") => HotOrNot::Not,
            None => HotOrNot::Not,
            _ => {
                return Err(ServerFnError::new(
                    "hot_or_not_evaluator.compare_videos_hot_or_not MUST return a boolean",
                ));
            }
        };

        let worker_req = HoNGameVoteReq {
            request: req,
            fetched_sentiment: sentiment,
            signature: sig,
            post_creator: Some(post_info.poster_principal),
        };

        let req_url = format!("{WORKER_URL}vote_v2/{sender}");
        let client = reqwest::Client::new();
        let jwt = expect_context::<HonWorkerJwt>();
        let res = client
            .post(&req_url)
            .json(&worker_req)
            .header("Authorization", format!("Bearer {}", jwt.0))
            .send()
            .await?;

        if res.status() != reqwest::StatusCode::OK {
            return Err(ServerFnError::new(format!(
                "worker error: {}",
                res.text().await?
            )));
        }

        let vote_res: VoteResV2 = res.json().await?;

        Ok(vote_res)
    }
}

#[cfg(not(feature = "alloydb"))]
mod mock {
    use hon_worker_common::GameResultV2;

    use super::*;

    #[allow(dead_code)]
    pub async fn vote_with_cents_on_post(
        _sender: Principal,
        _req: VoteRequest,
        _sig: Signature,
        _prev_video_info: Option<(Principal, u64)>,
    ) -> Result<VoteResV2, ServerFnError> {
        Ok(VoteResV2 {
            game_result: GameResultV2::Win {
                win_amt: 0u32.into(),
                updated_balance: 0u32.into(),
            },
        })
    }
}
