use crate::token::info::TokenKeyParam;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_params;
use utils::time::parse_ns_to_datetime;
use yral_canisters_common::utils::transaction::{TxnDirection, TxnInfoType, TxnInfoWallet};

fn direction_to_icon(direction: TxnDirection) -> &'static icondata_core::IconData {
    use TxnDirection::*;
    match direction {
        Transaction => icondata::LuArrowLeftRight,
        Added => icondata::FaArrowDownSolid,
        Deducted => icondata::FaArrowUpSolid,
    }
}

fn txn_info_to_icon(txn_info: TxnInfoType) -> &'static icondata_core::IconData {
    let direction = TxnDirection::from(txn_info);
    direction_to_icon(direction)
}

#[component]
pub fn TxnView(
    info: TxnInfoWallet,
    #[prop(optional)] _ref: NodeRef<html::Div>,
    symbol: String,
) -> impl IntoView {
    let params = use_params::<TokenKeyParam>();
    let direction = TxnDirection::from(info.tag);
    let bal_res = format!(
        "{}{}",
        match direction {
            TxnDirection::Added => "+",
            TxnDirection::Deducted => "-",
            TxnDirection::Transaction => "",
        },
        info.amount.humanize_float_truncate_to_dp(8)
    );

    view! {
        <div
            node_ref=_ref
            class="grid grid-cols-2 grid-rows-1 w-full py-3 border-b-2 border-white/10 justify-between"
        >
            <div class="flex flex-row gap-2">
                {match direction {
                    TxnDirection::Added => {
                        view! {
                            <div class="flex items-center justify-center w-7 h-7 lg:w-10 lg:h-10 rounded-md text-green-600 bg-green-600/5 text-lg lg:text-xl">
                                <Icon icon=txn_info_to_icon(info.tag) />
                            </div>
                        }
                            .into_any()
                    }
                    TxnDirection::Deducted => {
                        view! {
                            <div class="flex items-center justify-center w-7 h-7 lg:w-10 lg:h-10 rounded-md text-red-600 bg-red-600/5 text-lg lg:text-xl">
                                <Icon icon=txn_info_to_icon(info.tag) />
                            </div>
                        }
                            .into_any()
                    }
                    TxnDirection::Transaction => {
                        view! {
                            <div class="flex items-center justify-center w-7 h-7 lg:w-10 lg:h-10 rounded-md text-white bg-blue-600/5 text-lg lg:text-xl">
                                <Icon icon=txn_info_to_icon(info.tag) />
                            </div>
                        }
                            .into_any()
                    }
                }} <div class="flex flex-col">
                    <span class="text-md md:text-lg font-semibold text-white">
                        {info.tag.to_text()}
                    </span>
                    {move || {
                        match info.tag {
                            TxnInfoType::Mint { to } => {
                                match params.get() {
                                    Ok(_) => None,
                                    Err(_) => {
                                        Some(
                                            view! {
                                                <div class="text-sm md:text-md text-white/50">
                                                    {format!("To: {to}")}
                                                </div>
                                            }
                                                .into_any(),
                                        )
                                    }
                                }
                            }
                            TxnInfoType::Burn { from } => {
                                match params.get() {
                                    Ok(_) => None,
                                    Err(_) => {
                                        Some(
                                            view! {
                                                <div class="text-sm md:text-md text-white/50">
                                                    {format!("From: {from}")}
                                                </div>
                                            }
                                                .into_any(),
                                        )
                                    }
                                }
                            }
                            TxnInfoType::Received { from } => {
                                Some(
                                    view! {
                                        <div class="text-sm md:text-md text-white/50">
                                            {format!("From: {from}")}
                                        </div>
                                    }
                                        .into_any(),
                                )
                            }
                            TxnInfoType::Sent { to } => {
                                Some(
                                    view! {
                                        <div class="text-sm md:text-md text-white/50">
                                            {format!("To: {to}")}
                                        </div>
                                    }
                                        .into_any(),
                                )
                            }
                            TxnInfoType::Transfer { from, to } => {
                                Some(
                                    view! {
                                        <div class="flex flex-col space-y-1">
                                            <div class="text-sm md:text-md text-white/50">
                                                {format!("From: {from}")}
                                            </div>
                                            <div class="text-sm md:text-md text-white/50">
                                                {format!("To: {to}")}
                                            </div>
                                        </div>
                                    }
                                        .into_any(),
                                )
                            }
                        }
                    }}
                </div>
            </div>
            <div class="flex flex-col top-0 text-right">
                <span class=move || {
                    match direction {
                        TxnDirection::Added => "text-green-600 font-semibold",
                        _ => "text-white font-semibold",
                    }
                }>{format!("{bal_res} {symbol}")}</span>
                <span class="text-sm md:text-md text-white/50">
                    {parse_ns_to_datetime(info.timestamp).ok()}
                </span>
            </div>
        </div>
    }
}

pub mod provider {

    use candid::Principal;
    use yral_canisters_common::{
        cursored_data::{transaction::IndexOrLedger, CursoredDataProvider},
        Canisters,
    };

    use super::*;

    pub(crate) fn get_history_provider(
        canisters: Canisters<false>,
        source: IndexOrLedger,
        decimals: u8,
    ) -> impl CursoredDataProvider<Data = TxnInfoWallet> + Clone {
        #[cfg(feature = "mock-wallet-history")]
        {
            _ = canisters;
            _ = source;
            _ = decimals;
            mock::MockHistoryProvider
        }
        #[cfg(not(feature = "mock-wallet-history"))]
        {
            use yral_canisters_common::cursored_data::transaction::TxnHistory;
            TxnHistory {
                canisters,
                source,
                decimals,
            }
        }
    }

    #[cfg(feature = "mock-wallet-history")]
    mod mock {
        use std::convert::Infallible;

        use rand::{rngs::SmallRng, RngCore, SeedableRng};
        use yral_canisters_common::{
            cursored_data::PageEntry,
            utils::{time::current_epoch, token::balance::TokenBalance},
        };

        use super::*;

        #[derive(Clone, Copy)]
        pub struct MockHistoryProvider;

        fn tag_from_u32(v: u32) -> TxnInfoType {
            match v % 4 {
                0 => TxnInfoType::Mint {
                    to: Principal::anonymous(),
                },
                1 => TxnInfoType::Burn {
                    from: Principal::anonymous(),
                },
                2 => TxnInfoType::Received {
                    from: Principal::anonymous(),
                },
                3 => TxnInfoType::Sent {
                    to: Principal::anonymous(),
                },
                4 => TxnInfoType::Transfer {
                    from: Principal::anonymous(),
                    to: Principal::anonymous(),
                },
                _ => unreachable!(),
            }
        }
        impl CursoredDataProvider for MockHistoryProvider {
            type Data = TxnInfoWallet;
            type Error = Infallible;

            async fn get_by_cursor_inner(
                &self,
                from: usize,
                end: usize,
            ) -> Result<PageEntry<TxnInfoWallet>, Infallible> {
                let mut rand_gen = SmallRng::seed_from_u64(current_epoch().as_nanos() as u64);
                let data = (from..end)
                    .map(|_| TxnInfoWallet {
                        amount: TokenBalance::new_cdao((rand_gen.next_u64() % 3001).into()),
                        timestamp: rand_gen.next_u64(),
                        tag: tag_from_u32(rand_gen.next_u32()),
                        id: rand_gen.next_u64(),
                    })
                    .collect();
                Ok(PageEntry { data, end: false })
            }
        }
    }
}
