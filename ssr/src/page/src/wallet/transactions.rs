use leptos::prelude::*;
use yral_canisters_common::cursored_data::transaction::IndexOrLedger;

use super::txn::{provider::get_history_provider, TxnView};
use component::infinite_scroller::InfiniteScroller;
use state::canisters::unauth_canisters;

const FETCH_CNT: usize = 15;

#[component]
pub fn TransactionList(source: IndexOrLedger, symbol: String, decimals: u8) -> impl IntoView {
    let provider = get_history_provider(unauth_canisters(), source, decimals);
    view! {
        <div class="flex flex-col w-full justify-between items-stretch">
            <InfiniteScroller
                provider
                fetch_count=FETCH_CNT
                children=move |info, _ref| {
                    view! { <TxnView info _ref=_ref.unwrap_or_default() symbol=symbol.clone() /> }
                }
            />
        </div>
    }
}

#[component]
pub fn Transactions(source: IndexOrLedger, symbol: String, decimals: u8) -> impl IntoView {
    view! {
        <span class="text-xl w-full text-white font-bold">Transactions</span>

        <div class="flex items-center flex-col gap- pb-12 w-full">
            <div class="flex flex-col divide-y divide-white/10 w-full">
                <TransactionList source=source symbol decimals />
            </div>
        </div>
    }
}
