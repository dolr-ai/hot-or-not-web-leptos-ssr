use codee::string::FromToStringCodec;
use component::show_any::ShowAny;
use leptos::{html::Div, prelude::*};
use leptos_router::hooks::use_query;
use leptos_use::{use_cookie, use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use log;
use state::canisters::{auth_state, unauth_canisters};
use utils::token::icpump::{get_paginated_token_list_with_limit, IcpumpTokenInfo, TokenListItem};
use yral_canisters_common::{utils::token::RootType, Canisters};

use crate::icpump::{process_token_list_item, ProcessedTokenListResponse};

pub mod profile;
pub use profile::*;
pub mod test;
pub mod withdrawal;
pub use test::*;

pub(super) mod model;
use model::*;

mod context;
use context::*;

pub(super) mod components;
use components::header::Header;
use components::{
    card::{CardSkeleton, GameCard},
    onboarding::OnboardingPopup,
};

async fn load_selected_card(
    cans: &Canisters<true>,
    card_query: CardQuery,
) -> Result<ProcessedTokenListResponse, String> {
    let CardQuery { root, .. } = card_query;

    let meta = cans
        .token_metadata_by_root_type(
            &IcpumpTokenInfo,
            Some(cans.user_principal()),
            RootType::Other(root),
        )
        .await
        .map_err(|err| format!("Couldn't load token's meta info: {err}"))?
        .ok_or("The token doesn't exist".to_string())?;

    // REFACTOR: create struct with only the information that's actually needed
    Ok(ProcessedTokenListResponse {
        token_details: TokenListItem {
            user_id: "notneeded".into(),
            name: meta.name.clone(),
            token_name: meta.name,
            token_symbol: meta.symbol,
            logo: meta.logo_b64,
            description: meta.description,
            created_at: "notneeded".into(),
            formatted_created_at: "notneeded".into(),
            link: "notneeded".into(),
            is_nsfw: meta.is_nsfw,
        },
        root,
        token_owner: meta.token_owner,
        is_airdrop_claimed: false, // not needed
    })
}

#[component]
pub fn PumpNDump() -> impl IntoView {
    let card_query = use_query::<CardQuery>();
    let s: ShowSelectedCardSignal = RwSignal::new(ShowSelectedCard(
        card_query.with_untracked(|cq| cq.as_ref().is_ok_and(|q| q.is_valid())),
    ));
    provide_context(s);
    let card_query_fused = RwSignal::new(card_query.get_untracked().ok());

    let auth = auth_state();
    provide_context(PlayerDataRes::derive(auth));

    let (should_show, set_should_show) = use_cookie::<bool, FromToStringCodec>("show_onboarding");
    let show_onboarding = ShowOnboarding(should_show, set_should_show);
    provide_context(show_onboarding);

    let tokens = RwSignal::new(Vec::<ProcessedTokenListResponse>::new());

    let cans = unauth_canisters();
    let token_fetch_action = Action::new_local(move |page: &u32| {
        let cans = cans.clone();
        let page = *page;
        async move {
            let cans = auth.auth_cans(cans).await?;

            let selected_card = card_query_fused.get();
            card_query_fused.update(|item| *item = None);

            let selected_card = match selected_card {
                Some(q) => load_selected_card(&cans, q)
                    .await
                    .inspect_err(|err| {
                        log::error!("Couldn't load selected card: {err}");
                    })
                    .ok(),
                None => Default::default(),
            };

            let limit = 5;
            let more_tokens = get_paginated_token_list_with_limit(page, limit).await?;
            let list_end = more_tokens.is_empty();

            let mut processed_token: Vec<_> = selected_card.into_iter().collect();
            let mut more_processed_tokens =
                process_token_list_item(more_tokens, cans.user_principal()).await;

            processed_token.append(&mut more_processed_tokens);
            // ignore tokens with no owners
            processed_token.retain(|item| item.token_owner.is_some());

            tokens.update(|tokens| {
                tokens.append(&mut processed_token);
            });

            Ok::<_, ServerFnError>((page + 1, list_end))
        }
    });
    let token_fetching = token_fetch_action.pending();
    let prev_state = token_fetch_action.value();
    let fetch_more_tokens = move || {
        if token_fetching.get_untracked() {
            return;
        };
        let Ok((next_page, list_end)) = prev_state.get_untracked().unwrap_or(Ok((1, false))) else {
            return;
        };
        if list_end {
            return;
        }
        token_fetch_action.dispatch(next_page);
    };

    let scroll_container = NodeRef::<Div>::new();
    let loading = use_infinite_scroll_with_options(
        scroll_container,
        move |_| {
            fetch_more_tokens();
            std::future::ready(())
        },
        // start loading early, throttled at 3s per load
        UseInfiniteScrollOptions::default()
            .distance(400f64)
            .interval(2000f64),
    );

    view! {
        <div class="h-screen w-screen block text-white bg-black">
            <div class="max-w-md flex flex-col relative w-full mx-auto items-center h-full px-4 py-4">
                <Header />
                <div node_ref=scroll_container class="size-full overflow-scroll flex flex-col gap-4 snap-mandatory snap-y pb-[50vh]">
                    <For each=move || tokens.get() key=|item| (item.root, item.token_details.token_name.clone()) let:token>
                        <GameCard token />
                    </For>
                    <ShowAny when=move || loading.get() || tokens.with(|t| t.is_empty())>
                        <CardSkeleton />
                    </ShowAny>
                </div>
            </div>
            <ShowAny when=move || show_onboarding.should_show()>
                <OnboardingPopup />
            </ShowAny>
        </div>
    }
}
