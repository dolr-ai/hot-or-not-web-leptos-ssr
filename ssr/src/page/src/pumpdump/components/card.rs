pub(super) mod result;
pub(super) mod skeleton;
use ic_agent::identity::DelegatedIdentity;
use leptos_router::hooks::use_query;
pub use result::*;
pub use skeleton::*;

use codee::string::JsonSerdeCodec;
use consts::PUMP_AND_DUMP_WORKER_URL;
use leptos::{either::Either, prelude::*};
use leptos_use::{use_websocket, UseWebSocketReturn};
use state::canisters::auth_state;
use yral_pump_n_dump_common::ws::{websocket_connection_url, WsRequest};

use crate::{
    icpump::ProcessedTokenListResponse,
    pumpdump::{
        CardQuery, GameState, PlayerDataRes, RunningGameCtx, RunningGameRes, ShowSelectedCard,
        ShowSelectedCardSignal, WsResponse,
    },
};

#[component]
pub fn GameCard(token: ProcessedTokenListResponse) -> impl IntoView {
    let show_selected_card: ShowSelectedCardSignal = expect_context();
    let card_query = use_query::<CardQuery>().get().ok();

    provide_context(token.clone());

    let game_state = RwSignal::new(None);
    Effect::new(move |_| {
        match (
            show_selected_card.get(),
            card_query.as_ref().map(|c| c.details()),
        ) {
            (ShowSelectedCard(true), Some((root, Some(result)))) if root == token.root => {
                show_selected_card.set(ShowSelectedCard(false));
                game_state.set(Some(GameState::ResultDeclared(result)))
            }
            _ => game_state.set(Some(GameState::Playing)),
        }
    });

    let auth = auth_state();
    let player_data: PlayerDataRes = expect_context();
    let running_game_res = LocalResource::new(move || {
        let player_data = player_data.clone();
        let token = token.clone();
        async move {
            let mut ws_url = PUMP_AND_DUMP_WORKER_URL.clone();
            let scheme = {
                #[cfg(not(feature = "local-lib"))]
                {
                    "wss"
                }

                #[cfg(feature = "local-lib")]
                {
                    "ws"
                }
            };
            ws_url.set_scheme(scheme).expect("scheme to be valid");

            let cans = auth.cans_wire().await?;
            let id: DelegatedIdentity = cans.id.try_into()?;
            let websocket_url = websocket_connection_url(
                ws_url,
                &id,
                token.token_owner.as_ref().unwrap().canister_id,
                token.root,
            )
            .map_err(ServerFnError::new)?;

            let UseWebSocketReturn {
                message,
                send: sendfn,
                ..
            } = use_websocket::<WsRequest, WsResponse, JsonSerdeCodec>(websocket_url.as_str());

            let ctx = RunningGameCtx::derive(
                cans.user_canister,
                player_data,
                game_state,
                message,
                std::sync::Arc::new(sendfn),
                token,
            );

            Ok::<_, ServerFnError>(ctx)
        }
    });
    provide_context::<RunningGameRes>(running_game_res);

    view! {
        {move || game_state.get().map(move |game_state| view! {
            <div
                style="perspective: 500px; transition: transform 0.4s; transform-style: preserve-3d;"
                class="relative w-full min-h-[31rem] snap-start snap-always"
            >
                {match game_state {
                    GameState::Playing => Either::Left(view! {
                        <PlayingCard/>
                    }.into_any()),
                    GameState::ResultDeclared(result) => Either::Right(view! {
                        <ResultDeclaredCard result/>
                    }.into_any())
                }}
            </div>
        })}
    }
}
