use component::bullet_loader::BulletLoader;
use component::canisters_prov::AuthCansProvider;
use component::infinite_scroller::InfiniteScroller;
use history_provider::*;
use leptos::html;
use leptos::prelude::*;
use utils::time::get_day_month;
use yral_canisters_common::{
    cursored_data::ref_history::HistoryDetails, utils::profile::propic_from_principal, Canisters,
};

#[component]
fn HistoryItem(detail: HistoryDetails, _ref: NodeRef<html::Div>) -> impl IntoView {
    view! {
        <div node_ref=_ref class="px-2 grid grid-cols-4 grid-rows-1 items-center gap-2 w-full">
            <div class="flex flex-row col-span-3 items-center gap-4 justify-items-start">
                <img
                    class="aspect-square w-12 md:w-16 lg:w-24 rounded-full"
                    src=propic_from_principal(detail.referee)
                />
                <div class="grid grid-cols-1 grid-rows-2">
                    <span class="text-white text-lg truncate">{detail.referee.to_text()}</span>
                    <span class="text-white/50 text-sm md:text-md">
                        {get_day_month(detail.epoch_secs)}
                    </span>
                </div>
            </div>
            <span class="text-white text-md md:text-xl text-center justify-self-end">
                {detail.amount} Cents
            </span>
        </div>
    }
}

#[component]
fn AuthenticatedHistory(canisters: Canisters<true>) -> impl IntoView {
    let provider = get_history_provider(canisters);
    view! {
        <div class="flex flex-col justify-center w-full md:w-10/12 lg:w-8/12 gap-4 pb-16">
            <InfiniteScroller
                provider
                fetch_count=10
                children=|detail, _ref| {
                    view! { <HistoryItem detail _ref=_ref.unwrap_or_default() /> }
                }
            />

        </div>
    }
}

#[component]
pub fn HistoryView() -> impl IntoView {
    view! {
        <AuthCansProvider fallback=BulletLoader let:canisters>
            <AuthenticatedHistory canisters />
        </AuthCansProvider>
    }
}

mod history_provider {
    use yral_canisters_common::{
        cursored_data::{ref_history::HistoryDetails, CursoredDataProvider, PageEntry},
        Canisters,
    };

    pub fn get_history_provider(
        canisters: Canisters<true>,
    ) -> impl CursoredDataProvider<Data = HistoryDetails> + Clone {
        #[cfg(feature = "mock-referral-history")]
        {
            _ = canisters;
            mock::MockHistoryProvider
        }
        #[cfg(not(feature = "mock-referral-history"))]
        {
            use yral_canisters_common::cursored_data::ref_history::ReferralHistory;
            ReferralHistory(canisters)
        }
    }

    #[cfg(feature = "mock-referral-history")]
    mod mock {
        use std::convert::Infallible;

        use ic_agent::{identity::Secp256k1Identity, Identity};
        use k256::SecretKey;
        use rand_chacha::{
            rand_core::{RngCore, SeedableRng},
            ChaCha8Rng,
        };

        use yral_canisters_common::utils::time::current_epoch;

        use super::*;

        #[derive(Clone, Copy)]
        pub struct MockHistoryProvider;

        impl CursoredDataProvider for MockHistoryProvider {
            type Data = HistoryDetails;
            type Error = Infallible;

            async fn get_by_cursor_inner(
                &self,
                from: usize,
                end: usize,
            ) -> Result<PageEntry<HistoryDetails>, Infallible> {
                let mut rand_gen = ChaCha8Rng::seed_from_u64(current_epoch().as_nanos() as u64);
                let data = (from..end)
                    .map(|_| {
                        let sk = SecretKey::random(&mut rand_gen);
                        let epoch_secs = rand_gen.next_u32() as u64;
                        let identity = Secp256k1Identity::from_private_key(sk);
                        let amount = rand_gen.next_u64() % 500;
                        HistoryDetails {
                            epoch_secs,
                            referee: identity.sender().unwrap(),
                            amount,
                        }
                    })
                    .collect();
                Ok(PageEntry { data, end: false })
            }
        }
    }
}
