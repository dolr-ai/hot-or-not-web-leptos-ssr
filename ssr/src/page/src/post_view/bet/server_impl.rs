use candid::Principal;
use hon_worker_common::VoteRequest;
use leptos::prelude::*;
use yral_identity::Signature;

use crate::post_view::bet::VoteAPIRes;

#[server(endpoint = "vote", input = server_fn::codec::Json)]
pub async fn vote_with_cents_on_post(
    sender: Principal,
    req: VoteRequest,
    sig: Signature,
    prev_video_info: Option<(Principal, String)>,
) -> Result<VoteAPIRes, ServerFnError> {
    #[cfg(feature = "alloydb")]
    use alloydb::vote_with_cents_on_post;
    #[cfg(not(feature = "alloydb"))]
    use mock::vote_with_cents_on_post;

    // validate request against global_constants

    use global_constants::MAX_BET_AMOUNT_SATS;
    if req.vote_amount > MAX_BET_AMOUNT_SATS as u128 {
        return Err(ServerFnError::new(format!(
            "bet amount exceeds maximum allowed: {} > {}",
            req.vote_amount, MAX_BET_AMOUNT_SATS
        )));
    }

    vote_with_cents_on_post(sender, req, sig, prev_video_info).await
}

#[cfg(feature = "alloydb")]
mod alloydb {
    use crate::post_view::bet::{VideoComparisonResult, VoteAPIRes};

    use super::*;
    use hon_worker_common::WORKER_URL;
    use hon_worker_common::{HoNGameVoteReqV3, HotOrNot, VoteRequestV3, VoteResV2};
    pub async fn vote_with_cents_on_post(
        sender: Principal,
        req: VoteRequest,
        sig: Signature,
        prev_video_info: Option<(Principal, String)>,
    ) -> Result<VoteAPIRes, ServerFnError> {
        use state::alloydb::AlloyDbInstance;
        use state::server::HonWorkerJwt;
        use yral_canisters_common::Canisters;

        let cans: Canisters<false> = expect_context();
        let Some(post_info) = cans
            .get_post_details(req.post_canister, req.post_id.to_string())
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
            "select hot_or_not_evaluator.compare_videos_hot_or_not_v2('{}', {})",
            post_info.uid, prev_uid_formatted,
        );

        let alloydb: AlloyDbInstance = expect_context();
        let mut res = alloydb.execute_sql_raw(query).await?;
        let mut res = res
            .sql_results
            .pop()
            .expect("hot_or_not_evaluator.compare_videos_hot_or_not_v2 MUST return a result");
        let mut res = res
            .rows
            .pop()
            .expect("hot_or_not_evaluator.compare_videos_hot_or_not_v2 MUST return a row");
        let res = res
            .values
            .pop()
            .expect("hot_or_not_evaluator.compare_videos_hot_or_not_v2 MUST return a value");

        let video_comparison_result = match res.value {
            Some(val) => VideoComparisonResult::parse_video_comparison_result(&val)
                .map_err(ServerFnError::new)?,
            None => {
                return Err(ServerFnError::new(
                    "hot_or_not_evaluator.compare_videos_hot_or_not_v2 returned no value",
                ))
            }
        };
        let sentiment = match video_comparison_result.hot_or_not {
            true => HotOrNot::Hot,
            false => HotOrNot::Not,
        };

        // Convert VoteRequest to VoteRequestV3
        let req_v3 = VoteRequestV3 {
            publisher_principal: post_info.poster_principal,
            post_id: req.post_id,
            vote_amount: req.vote_amount,
            direction: req.direction,
        };

        let worker_req = HoNGameVoteReqV3 {
            request: req_v3,
            fetched_sentiment: sentiment,
            signature: sig,
            post_creator: Some(post_info.poster_principal),
        };

        let req_url = format!("{WORKER_URL}v3/vote/{sender}");
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

        Ok(VoteAPIRes {
            game_result: vote_res,
            video_comparison_result,
        })
    }
}

#[cfg(not(feature = "alloydb"))]
mod mock {
    use hon_worker_common::{GameResultV2, VoteResV2};
    use state::hn_bet_state::VideoComparisonResult;

    use super::*;

    #[allow(dead_code)]
    pub async fn vote_with_cents_on_post(
        _sender: Principal,
        _req: VoteRequest,
        _sig: Signature,
        _prev_video_info: Option<(Principal, String)>,
    ) -> Result<VoteAPIRes, ServerFnError> {
        let game_result = VoteResV2 {
            game_result: GameResultV2::Win {
                win_amt: 0u32.into(),
                updated_balance: 0u32.into(),
            },
        };
        Ok(VoteAPIRes {
            game_result,
            video_comparison_result: VideoComparisonResult {
                hot_or_not: true,
                current_video_score: 50.0,
                previous_video_score: 10.0,
            },
        })
    }
}
