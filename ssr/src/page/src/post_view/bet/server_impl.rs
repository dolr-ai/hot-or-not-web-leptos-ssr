use candid::Principal;
use hon_worker_common::{ServerVoteRequest, VoteRequest};
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

#[server(endpoint = "v2/vote", input = server_fn::codec::Json)]
#[tracing::instrument(skip(sig), fields(sender = %sender, post_id = %req.post_id, vote_amount = %req.vote_amount))]
pub async fn vote_with_cents_post_v2(
    sender: Principal,
    req: ServerVoteRequest,
    sig: Signature,
    prev_video_info: Option<(Principal, String)>,
) -> Result<VoteAPIRes, ServerFnError> {
    #[cfg(feature = "alloydb")]
    use alloydb::vote_with_cents_on_post_v2;
    #[cfg(not(feature = "alloydb"))]
    use mock::vote_with_cents_on_post_v2;

    // validate request against global_constants

    use global_constants::MAX_BET_AMOUNT_SATS;
    if req.vote_amount > MAX_BET_AMOUNT_SATS as u128 {
        return Err(ServerFnError::new(format!(
            "bet amount exceeds maximum allowed: {} > {}",
            req.vote_amount, MAX_BET_AMOUNT_SATS
        )));
    }

    vote_with_cents_on_post_v2(sender, req, sig, prev_video_info).await
}

#[cfg(feature = "alloydb")]
mod alloydb {
    use crate::post_view::bet::{VideoComparisonResult, VoteAPIRes};

    use super::*;
    use hon_worker_common::HoNGameVoteReqV4;
    use hon_worker_common::VoteRequestV4;
    use hon_worker_common::WORKER_URL;
    use hon_worker_common::{GameResultV2, HotOrNot, VoteResV2};

    #[tracing::instrument(skip(sig))]
    pub async fn vote_with_cents_on_post(
        sender: Principal,
        req: VoteRequest,
        sig: Signature,
        prev_video_info: Option<(Principal, String)>,
    ) -> Result<VoteAPIRes, ServerFnError> {
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

        let vote_request = VoteRequestV4 {
            publisher_principal: post_info.poster_principal,
            post_id: req.post_id.to_string(),
            vote_amount: req.vote_amount,
            direction: req.direction,
        };

        compare_video_with_uid(
            sender,
            vote_request,
            sig,
            &post_info.uid,
            &prev_uid_formatted,
        )
        .await
    }

    #[tracing::instrument(skip(sig), fields(sender = %sender, post_id = %req.post_id, vote_amount = %req.vote_amount))]
    pub async fn vote_with_cents_on_post_v2(
        sender: Principal,
        req: ServerVoteRequest,
        sig: Signature,
        prev_video_info: Option<(Principal, String)>,
    ) -> Result<VoteAPIRes, ServerFnError> {
        use yral_canisters_common::Canisters;

        let cans: Canisters<false> = expect_context();

        tracing::info!("Fetching post details for post_id: {}", req.post_id);
        let post_details_span = tracing::info_span!("fetch_post_details");
        let Some(post_info) = post_details_span
            .in_scope(|| async {
                cans.get_post_details(req.post_canister, req.post_id.clone())
                    .await
            })
            .await?
        else {
            tracing::warn!("Post not found: {}", req.post_id);
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

        let vote_request = VoteRequestV4 {
            publisher_principal: post_info.poster_principal,
            post_id: req.post_id,
            vote_amount: req.vote_amount,
            direction: req.direction,
        };

        compare_video_with_uid(
            sender,
            vote_request,
            sig,
            &post_info.uid,
            &prev_uid_formatted,
        )
        .await
    }

    #[tracing::instrument(skip(request_signature), fields(sender = %sender, post_uid = %post_uid, previous_post_uid = %previous_post_uid, vote_amount = %vote_request.vote_amount))]
    async fn compare_video_with_uid(
        sender: Principal,
        vote_request: VoteRequestV4,
        request_signature: Signature,
        post_uid: &str,
        previous_post_uid: &str,
    ) -> Result<VoteAPIRes, ServerFnError> {
        use state::alloydb::AlloyDbInstance;
        use state::server::HonWorkerJwt;

        let query = format!(
            "select hot_or_not_evaluator.compare_videos_hot_or_not_v3('{post_uid}', {previous_post_uid})",
        );

        // TODO: figure out the overhead from this alloydb call in prod
        tracing::info!("Executing AlloyDB video comparison query");
        let alloydb_span = tracing::info_span!("alloydb_video_comparison");
        let alloydb: AlloyDbInstance = expect_context();
        let mut res = alloydb_span
            .in_scope(|| async { alloydb.execute_sql_raw(query).await })
            .await?;

        tracing::info!("Parsing AlloyDB response");
        let parse_span = tracing::info_span!("parse_alloydb_response");
        let res = parse_span.in_scope(|| {
            let mut sql_results = res.sql_results.pop().ok_or_else(|| {
                tracing::error!("No SQL results returned from compare_videos_hot_or_not_v3");
                ServerFnError::new(
                    "hot_or_not_evaluator.compare_videos_hot_or_not_v3 MUST return a result",
                )
            })?;

            let mut rows = sql_results.rows.pop().ok_or_else(|| {
                tracing::error!("No rows returned from compare_videos_hot_or_not_v3");
                ServerFnError::new(
                    "hot_or_not_evaluator.compare_videos_hot_or_not_v3 MUST return a row",
                )
            })?;

            let value = rows.values.pop().ok_or_else(|| {
                tracing::error!("No values returned from compare_videos_hot_or_not_v3");
                ServerFnError::new(
                    "hot_or_not_evaluator.compare_videos_hot_or_not_v3 MUST return a value",
                )
            })?;

            tracing::info!("Successfully parsed AlloyDB response");
            Ok::<_, ServerFnError>(value)
        })?;

        tracing::info!("Parsing video comparison result");
        let comparison_span = tracing::info_span!("parse_video_comparison");
        let (video_comparison_result, sentiment) = comparison_span.in_scope(|| {
            let video_comparison_result = match res.value {
                Some(val) => {
                    tracing::info!("Parsing video comparison value");
                    VideoComparisonResult::parse_video_comparison_result(&val).map_err(|e| {
                        tracing::error!("Failed to parse video comparison: {}", e);
                        ServerFnError::new(e)
                    })?
                }
                None => {
                    tracing::error!("No value in video comparison result");
                    return Err(ServerFnError::new(
                        "hot_or_not_evaluator.compare_videos_hot_or_not_v3 returned no value",
                    ));
                }
            };

            let sentiment = match video_comparison_result.hot_or_not {
                true => {
                    tracing::info!("Sentiment determined: Hot");
                    HotOrNot::Hot
                }
                false => {
                    tracing::info!("Sentiment determined: Not");
                    HotOrNot::Not
                }
            };

            Ok((video_comparison_result, sentiment))
        })?;

        tracing::info!("Preparing worker request");
        let request_prep_span = tracing::info_span!("prepare_worker_request");
        let worker_req = request_prep_span.in_scope(|| {
            let post_creator_principal = vote_request.publisher_principal;

            HoNGameVoteReqV4 {
                request: vote_request,
                fetched_sentiment: sentiment,
                signature: request_signature,
                post_creator: Some(post_creator_principal),
            }
        });

        let req_url = format!("{WORKER_URL}v4/vote/{sender}");
        let client = reqwest::Client::new();
        let jwt = expect_context::<HonWorkerJwt>();

        tracing::info!("Sending vote request to worker API");
        let worker_span = tracing::info_span!("worker_api_call", url = %req_url);
        let res = worker_span
            .in_scope(|| async {
                client
                    .post(&req_url)
                    .json(&worker_req)
                    .header("Authorization", format!("Bearer {}", jwt.0))
                    .send()
                    .await
            })
            .await?;

        tracing::info!("Processing worker response");
        let response_span = tracing::info_span!("process_worker_response");
        let vote_res = response_span
            .in_scope(|| async {
                let status = res.status();
                tracing::info!("Worker response status: {}", status);

                if status != reqwest::StatusCode::OK {
                    let error_text = res.text().await?;
                    tracing::error!("Worker returned error: {}", error_text);
                    return Err(ServerFnError::new(format!("worker error: {error_text}")));
                }

                tracing::info!("Deserializing vote response");
                let deserialize_span = tracing::info_span!("deserialize_vote_response");
                let vote_res: VoteResV2 = deserialize_span
                    .in_scope(|| async { res.json().await })
                    .await?;

                tracing::info!("Successfully processed worker response");
                Ok(vote_res)
            })
            .await?;

        // Update leaderboard - track games won
        // This is fire-and-forget: spawn a task so we don't block the response
        #[cfg(feature = "ssr")]
        if matches!(vote_res.game_result, GameResultV2::Win { .. }) {
            tracing::info!("User won game, updating leaderboard");
            tokio::spawn(async move {
                let leaderboard_span = tracing::info_span!("leaderboard_update", user = %sender);
                let _guard = leaderboard_span.enter();

                if let Err(e) = update_leaderboard_score(
                    sender,
                    1.0, // Increment games played by 1
                    "games_won",
                )
                .await
                {
                    tracing::error!("Failed to update leaderboard for user {}: {:?}", sender, e);
                } else {
                    tracing::info!("Successfully updated leaderboard for user {}", sender);
                }
            });
        }

        tracing::info!("Vote processed successfully for user {}", sender);
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
    pub async fn vote_with_cents_on_post_v2(
        _sender: Principal,
        _req: ServerVoteRequest,
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

    #[allow(dead_code)]
    #[tracing::instrument(skip(_sig))]
    pub async fn vote_with_cents_on_post(
        sender: Principal,
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
