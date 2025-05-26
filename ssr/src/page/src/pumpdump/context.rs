use std::sync::Arc;

use candid::Principal;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use state::canisters::AuthCansResource;
use utils::send_wrap;
use yral_pump_n_dump_common::{
    ws::{GameResult as RawGameResult, WsError, WsMessage, WsRequest, WsResp},
    GameDirection,
};

use crate::icpump::ProcessedTokenListResponse;

use super::{
    convert_e8s_to_cents,
    model::{GameRunningData, PlayerData},
    GameResult, GameState,
};

#[derive(Copy, Clone, Debug)]
pub(super) struct ShowOnboarding(
    pub(super) Signal<Option<bool>>,
    pub(super) WriteSignal<Option<bool>>,
);

impl ShowOnboarding {
    #[inline]
    pub(super) fn show(&self) {
        self.1.set(Some(true));
    }

    #[inline]
    pub(super) fn hide(&self) {
        self.1.set(Some(false));
    }

    #[inline]
    pub(super) fn should_show(&self) -> bool {
        self.0.get().unwrap_or(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ShowSelectedCard(pub(super) bool);

pub(super) type ShowSelectedCardSignal = RwSignal<ShowSelectedCard>;
/// This is a local resource, you don't need <Suspense> with this
pub(super) type RunningGameRes = LocalResource<Result<RunningGameCtx, ServerFnError>>;

type SendFn = Arc<dyn Fn(&WsRequest) + Send + Sync>;

#[derive(Clone, Copy)]
pub(super) enum PlayerDataUpdate {
    IncrementGameCount {
        increase_wallet_amount_by: Option<u128>,
    },
    DecrementWalletBalance,
}

#[derive(Clone)]
pub(super) struct PlayerDataRes {
    /// this is a local resource
    pub read: Resource<Result<RwSignal<PlayerData>, ServerFnError>>,
    update: Action<PlayerDataUpdate, ()>,
}

impl PlayerDataRes {
    pub fn derive(auth_cans: AuthCansResource) -> Self {
        let read = Resource::new(
            || (),
            move |_| {
                send_wrap(async move {
                    let cans_wire = auth_cans.await;
                    let data = PlayerData::load(cans_wire?.user_canister).await?;
                    Ok::<_, ServerFnError>(RwSignal::new(data))
                })
            },
        );

        let read_c = read;
        let update = Action::new(move |&update_kind| {
            let read = read_c;
            async move {
                let Ok(pd) = read.await else {
                    return;
                };
                match update_kind {
                    PlayerDataUpdate::IncrementGameCount {
                        increase_wallet_amount_by,
                    } => pd.update(|data| {
                        data.games_count += 1;
                        if let Some(amt) = increase_wallet_amount_by {
                            data.wallet_balance += amt;
                        }
                    }),
                    PlayerDataUpdate::DecrementWalletBalance => {
                        pd.update(|d| d.wallet_balance = d.wallet_balance.saturating_sub(1));
                    }
                }
            }
        });

        Self { read, update }
    }
}

// TODO: use the WsResponse from pnd common crate
#[derive(Serialize, Deserialize, Clone)]
pub(super) struct WsResponse {
    pub(super) request_id: uuid::Uuid,
    pub(super) response: WsResp,
}

#[derive(Clone)]
pub(super) struct RunningGameCtx {
    sendfn: SendFn,
    pub reload_running_data: Action<(), ()>,
    player_data: PlayerDataRes,
    running_data: RwSignal<Option<GameRunningData>>,
    current_round: StoredValue<Option<u64>>,
}

impl RunningGameCtx {
    fn compute_reward(running_data: GameRunningData, raw_result: &RawGameResult) -> u128 {
        let user_bet_count = match raw_result.direction {
            GameDirection::Pump => running_data.pumps,
            GameDirection::Dump => running_data.dumps,
        };

        // backend already deducts 10% fees, so fe dont need to
        let reward = (user_bet_count * raw_result.reward_pool.clone()) / raw_result.bet_count;

        convert_e8s_to_cents(reward)
    }

    fn compute_game_result(
        running_data: GameRunningData,
        raw_result: &RawGameResult,
    ) -> GameResult {
        if running_data.pumps == running_data.dumps {
            return GameResult::Win { amount: 0 };
        }

        let spent = running_data.pumps as u128 + running_data.dumps as u128;
        let reward = Self::compute_reward(running_data, raw_result);

        if spent > reward {
            GameResult::Loss {
                // net loss
                amount: spent - reward,
            }
        } else {
            GameResult::Win {
                // net profit or zero
                amount: reward - spent,
            }
        }
    }

    pub fn derive(
        user_canister: Principal,
        player_data: PlayerDataRes,
        game_state: RwSignal<Option<GameState>>,
        ws_message: Signal<Option<WsResponse>>,
        sendfn: SendFn,
        token: ProcessedTokenListResponse,
    ) -> Self {
        let running_data = RwSignal::new(None);
        let current_round = StoredValue::new(None);

        let token_owner_canister = token.token_owner.as_ref().map(|o| o.canister_id).unwrap();
        let reload_running_data = Action::new_local(move |_| async move {
            let data = GameRunningData::load(token_owner_canister, token.root, user_canister)
                .await
                .inspect_err(|err| {
                    log::error!("couldn't load running data: {err}");
                })
                .ok();

            if data.is_some() {
                game_state.set(Some(GameState::Playing));
            }

            running_data.set(data);
        });

        Effect::new(move |_| {
            let msg = ws_message.get()?;
            match msg.response {
                WsResp::WelcomeEvent {
                    round,
                    pool,
                    player_count,
                    user_bets,
                } => {
                    running_data.set(Some(GameRunningData::new(
                        user_bets.pumps,
                        user_bets.dumps,
                        player_count,
                        Some(pool),
                    )));
                    current_round.set_value(Some(round));
                }
                WsResp::BetSuccesful { .. } => (),
                WsResp::Error(e) => {
                    log::error!("ws: received error: {e:?}");
                    if let WsError::BetFailure { direction, .. } = e {
                        running_data.update(move |data| {
                            if let Some(data) = data {
                                match direction {
                                    GameDirection::Pump => data.pumps -= 1,
                                    GameDirection::Dump => data.dumps -= 1,
                                }
                            }
                        });
                    }
                }
                WsResp::GameResultEvent(res) => {
                    let running_data = running_data.get_untracked()?;
                    current_round.set_value(Some(res.new_round));
                    let result = Self::compute_game_result(running_data, &res);
                    let reward = Self::compute_reward(running_data, &res);
                    game_state.set(Some(GameState::ResultDeclared(result)));

                    match result {
                        GameResult::Win { .. } => {
                            player_data
                                .update
                                .dispatch(PlayerDataUpdate::IncrementGameCount {
                                    increase_wallet_amount_by: Some(reward),
                                });
                        }
                        GameResult::Loss { .. } => {
                            player_data
                                .update
                                .dispatch(PlayerDataUpdate::IncrementGameCount {
                                    increase_wallet_amount_by: Some(reward),
                                });
                        }
                    }
                }
                WsResp::WinningPoolEvent { new_pool, .. } => {
                    running_data.update(|d| {
                        let Some(data) = d.as_mut() else {
                            return;
                        };
                        data.winning_pot = Some(new_pool);
                    });
                }
            }
            Some(())
        });

        Self {
            sendfn,
            reload_running_data,
            running_data,
            current_round,
            player_data,
        }
    }

    /// returns a signal which is true when
    /// running data is loading
    pub fn loading_data(&self) -> Memo<bool> {
        self.reload_running_data.pending()
    }

    // create a method to avoid having to use parantheses around the field
    #[inline(always)]
    fn send(&self, message: &WsRequest) {
        (self.sendfn)(message);
    }

    pub fn with_running_data<T>(&self, f: impl FnOnce(&GameRunningData) -> T) -> Option<T> {
        self.running_data.with(|d| d.as_ref().map(f))
    }

    pub fn send_bet(&self, direction: GameDirection) {
        let Some(round) = self.current_round.get_value() else {
            log::debug!("trying to bet when game is not ready");
            return;
        };
        self.send(&WsRequest {
            request_id: uuid::Uuid::new_v4(),
            msg: WsMessage::Bet { direction, round },
        });

        self.player_data
            .update
            .dispatch(PlayerDataUpdate::DecrementWalletBalance);
        self.running_data.update(|value| {
            let Some(value) = value else {
                return;
            };
            match direction {
                GameDirection::Dump => {
                    value.dumps += 1;
                }
                GameDirection::Pump => {
                    value.pumps += 1;
                }
            }
        });
    }
}
