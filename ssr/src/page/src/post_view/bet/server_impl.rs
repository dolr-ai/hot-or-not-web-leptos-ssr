use candid::Principal;
use hon_worker_common::VoteRequest;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use tracing;
use yral_identity::Signature;

use crate::post_view::bet::VoteAPIRes;

#[derive(Serialize, Deserialize, Debug)]
struct LeaderboardUpdateRequest {
    principal_id: String,
    metric_value: f64,
    metric_type: String,
    source: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LeaderboardUpdateResponse {
    success: bool,
    new_score: f64,
    operation: String,
}

/// Helper function to update leaderboard score for games played
#[cfg(feature = "ssr")]
async fn update_leaderboard_score(
    principal_id: Principal,
    metric_value: f64,
    metric_type: &str,
) -> Result<(), ServerFnError> {
    use consts::OFF_CHAIN_AGENT_URL;

    // Get auth token from environment
    let mut auth_token = std::env::var("GRPC_AUTH_TOKEN")
        .map_err(|_| ServerFnError::new("Missing auth token for leaderboard update"))?;
    auth_token.retain(|c| !c.is_whitespace());

    let url = OFF_CHAIN_AGENT_URL
        .join("api/v1/leaderboard/score/update")
        .map_err(|e| ServerFnError::new(format!("Failed to build URL: {e}")))?;

    let request = LeaderboardUpdateRequest {
        principal_id: principal_id.to_string(),
        metric_value,
        metric_type: metric_type.to_string(),
        source: "web_app".to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .bearer_auth(auth_token)
        .json(&request)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to send leaderboard update: {e}")))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Leaderboard update failed: {error_text}"
        )));
    }

    Ok(())
}

#[server(endpoint = "vote", input = server_fn::codec::Json)]
#[tracing::instrument(skip(sig))]
pub async fn vote_with_cents_on_post(
    sender: Principal,
    req: VoteRequest,
    sig: Signature,
    prev_video_info: Option<(Principal, u64)>,
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
    use futures::try_join;
    use hon_worker_common::WORKER_URL;
    use hon_worker_common::{GameResultV2, HoNGameVoteReqV3, HotOrNot, VoteRequestV3, VoteResV2};
    use yral_canisters_client::individual_user_template::PostDetailsForFrontend;
    use yral_canisters_common::Canisters;

    async fn get_poster_principal_and_video_uid(
        cans: Canisters<false>,
        user_principal: Principal,
        post_id: u64,
    ) -> Result<(Principal, String), ServerFnError> {
        let PostDetailsForFrontend {
            video_uid,
            created_by_user_principal_id,
            ..
        } = cans
            .individual_user(user_principal)
            .await
            .get_individual_post_details_by_id(post_id)
            .await?;
        Ok((created_by_user_principal_id, video_uid))
    }

    #[tracing::instrument(skip(sig))]
    pub async fn vote_with_cents_on_post(
        sender: Principal,
        req: VoteRequest,
        sig: Signature,
        prev_video_info: Option<(Principal, u64)>,
    ) -> Result<VoteAPIRes, ServerFnError> {
        use state::alloydb::AlloyDbInstance;
        use state::server::HonWorkerJwt;

        // loads post details, only needs video_id and poster's id
        // makes sense to call the canister ourselves, as that will only incur network overhead + very slight computation on canister
        let ((poster_principal, uid), prev_uid_formatted) = try_join!(
            get_poster_principal_and_video_uid(expect_context(), req.post_canister, req.post_id),
            async {
                let res = if let Some((canister_id, post_id)) = prev_video_info {
                    // only needs the uid
                    let (_, uid) =
                        get_poster_principal_and_video_uid(expect_context(), canister_id, post_id)
                            .await?;
                    format!("'{uid}'")
                } else {
                    "NULL".to_string()
                };

                Ok::<_, ServerFnError>(res)
            }
        )?;

        // the above two `get_post_details` call could be run in parallel

        // sanitization is not required here, as get_post_details verifies that the post is valid
        // and exists on cloudflare
        let query = format!(
            "select hot_or_not_evaluator.compare_videos_hot_or_not_v3('{uid}', {prev_uid_formatted})",
        );

        // TODO: figure out the overhead from this alloydb call in prod
        let alloydb: AlloyDbInstance = expect_context();
        let mut res = alloydb.execute_sql_raw(query).await?;
        let mut res = res.sql_results.pop().ok_or_else(|| {
            ServerFnError::new(
                "hot_or_not_evaluator.compare_videos_hot_or_not_v3 MUST return a result",
            )
        })?;
        let mut res = res.rows.pop().ok_or_else(|| {
            ServerFnError::new(
                "hot_or_not_evaluator.compare_videos_hot_or_not_v3 MUST return a row",
            )
        })?;
        let res = res.values.pop().ok_or_else(|| {
            ServerFnError::new(
                "hot_or_not_evaluator.compare_videos_hot_or_not_v3 MUST return a value",
            )
        })?;

        let video_comparison_result = match res.value {
            Some(val) => VideoComparisonResult::parse_video_comparison_result(&val)
                .map_err(ServerFnError::new)?,
            None => {
                return Err(ServerFnError::new(
                    "hot_or_not_evaluator.compare_videos_hot_or_not_v3 returned no value",
                ))
            }
        };
        let sentiment = match video_comparison_result.hot_or_not {
            true => HotOrNot::Hot,
            false => HotOrNot::Not,
        };

        // Convert VoteRequest to VoteRequestV3
        let req_v3 = VoteRequestV3 {
            publisher_principal: poster_principal,
            post_id: req.post_id,
            vote_amount: req.vote_amount,
            direction: req.direction,
        };

        let worker_req = HoNGameVoteReqV3 {
            request: req_v3,
            fetched_sentiment: sentiment,
            signature: sig,
            post_creator: Some(poster_principal),
        };

        // TODO: figure out the overhead from this worker call in prod
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

        // huh?
        let vote_res: VoteResV2 = res.json().await?;

        // Update leaderboard - track games won
        // This is fire-and-forget: spawn a task so we don't block the response
        #[cfg(feature = "ssr")]
        if matches!(vote_res.game_result, GameResultV2::Win { .. }) {
            tokio::spawn(async move {
                if let Err(e) = update_leaderboard_score(
                    sender,
                    1.0, // Increment games played by 1
                    "games_won",
                )
                .await
                {
                    leptos::logging::error!(
                        "Failed to update leaderboard for user {}: {:?}",
                        sender,
                        e
                    );
                } else {
                    leptos::logging::log!("Successfully updated leaderboard for user {}", sender);
                }
            });
        }

        Ok(VoteAPIRes {
            game_result: vote_res,
            video_comparison_result,
        })
    }
}

#[cfg(not(feature = "alloydb"))]
mod mock {
    use hon_worker_common::{GameResultV2, VoteResV2};
    use leptos::task::{spawn, spawn_local};
    use state::hn_bet_state::VideoComparisonResult;

    use super::*;

    #[allow(dead_code)]
    #[tracing::instrument(skip(_sig))]
    pub async fn vote_with_cents_on_post(
        sender: Principal,
        _req: VoteRequest,
        _sig: Signature,
        _prev_video_info: Option<(Principal, u64)>,
    ) -> Result<VoteAPIRes, ServerFnError> {
        let game_result = VoteResV2 {
            game_result: GameResultV2::Win {
                win_amt: 0u32.into(),
                updated_balance: 0u32.into(),
            },
        };

        // Update leaderboard in mock mode as well (for testing)
        // Making it synchronous for now
        #[cfg(feature = "ssr")]
        tokio::spawn(async move {
            if let Err(e) = super::update_leaderboard_score(
                sender,
                1.0, // Increment games played by 1
                "games_played",
            )
            .await
            {
                leptos::logging::error!(
                    "Failed to update leaderboard for user {} in mock mode: {:?}",
                    sender,
                    e
                );
            } else {
                leptos::logging::log!(
                    "Successfully updated leaderboard for user {} in mock mode",
                    sender
                );
            }
        });

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
