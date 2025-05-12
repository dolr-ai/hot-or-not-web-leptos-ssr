use component::{back_btn::BackButton, title::TitleText};
use leptos::prelude::*;
use leptos_router::{hooks::use_query, params::Params};
use utils::event_streaming::events::SatsWithdrawn;
use utils::try_or_redirect_opt;
use yral_canisters_common::utils::token::balance::TokenBalance;
#[derive(Debug, PartialEq, Eq, Clone, Params)]
struct SuccessParams {
    sats: u128,
}

#[component]
pub fn Success() -> impl IntoView {
    let params = use_query::<SuccessParams>();
    let SuccessParams { sats } = try_or_redirect_opt!(params.get_untracked());
    let formatted_btc = TokenBalance::new(sats.into(), 8).humanize_float_truncate_to_dp(5);
    let formatted_sats = TokenBalance::new(sats.into(), 0).humanize_float_truncate_to_dp(0);

    // Track the withdrawal event
    let sats_value = formatted_sats.clone().parse::<f64>().unwrap_or(0.0);

    Effect::new(move |_| {
        SatsWithdrawn.send_event(sats_value);
    });

    Some(view! {
        <div
            style:background-image="url('/img/pumpdump/onboarding-bg.webp')"
            class="min-h-screen w-full flex flex-col text-white pt-2 pb-12 bg-black items-center relative max-md:bg-[length:271vw_100vh] md:bg-[length:max(100vw,100vh)] max-md:bg-[position:-51.2vh_-6vw]"
        >
            <div id="back-nav" class="flex flex-col items-center w-full gap-20 pb-16">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between">
                        <BackButton fallback="/" />
                    </div>
                </TitleText>
            </div>
            <div class="w-full">
                <div class="max-w-md w-full mx-auto px-4 mt-4 pb-6 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
                    <div class="w-full flex flex-col gap-12 items-center">
                        <img class="max-w-44" src="/img/hotornot/tick.webp" />
                        <div class="flex flex-col gap-8 w-full px-5">
                            <div class="flex flex-col gap-2 items-center">
                                <span class="font-bold text-lg">Withdraw Successful!</span>
                                <span class="text-neutral-300">Your wallet has been updated with {formatted_btc} BTC.</span>
                            </div>
                            <a class="rounded-lg px-5 py-2 text-center font-bold bg-white" href="/wallet">
                                <span class="bg-brand-gradient text-transparent bg-clip-text">Go to wallet</span>
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    })
}
