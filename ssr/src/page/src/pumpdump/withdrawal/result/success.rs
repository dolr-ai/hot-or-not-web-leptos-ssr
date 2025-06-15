use candid::Nat;
use component::{back_btn::BackButton, title::TitleText};
use leptos::prelude::*;
use leptos_router::{hooks::use_query, params::Params};
use state::canisters::auth_state;
use utils::event_streaming::events::CentsWithdrawn;
use utils::try_or_redirect_opt;
use yral_canisters_common::utils::token::balance::TokenBalance;

#[derive(Debug, PartialEq, Eq, Clone, Params)]
struct SuccessParams {
    cents: Nat,
}

#[component]
pub fn Success() -> impl IntoView {
    let params = use_query::<SuccessParams>();
    let SuccessParams { cents } = try_or_redirect_opt!(params.get_untracked());
    let formatted_dolr = TokenBalance::new(cents.clone(), 8).humanize_float_truncate_to_dp(4);
    let formatted_cents = TokenBalance::new(cents.clone(), 6).humanize_float_truncate_to_dp(4);

    // Track the withdrawal event
    let cents_value = formatted_cents.clone().parse::<f64>().unwrap_or(0.0);
    let auth = auth_state();

    Effect::new(move |_| {
        CentsWithdrawn.send_event(auth.event_ctx(), cents_value);
    });

    Some(view! {
        <div
            style:background-image="url('/img/yral/onboarding-bg.webp')"
            class="flex relative flex-col items-center pt-2 pb-12 w-full min-h-screen text-white bg-black max-md:bg-size-[271vw_100vh] max-md:bg-position-[-51.2vh_-6vw] md:bg-size-[max(100vw,100vh)]"
        >
            <div id="back-nav" class="flex flex-col gap-20 items-center pb-16 w-full">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-between">
                        <BackButton fallback="/" />
                    </div>
                </TitleText>
            </div>
            <div class="w-full">
                <div class="absolute top-1/2 left-1/2 px-4 pb-6 mx-auto mt-4 w-full max-w-md -translate-x-1/2 -translate-y-1/2">
                    <div class="flex flex-col gap-12 items-center w-full">
                        <img class="max-w-44" src="/img/yral/cents-stack.webp" />
                        <div class="flex flex-col gap-8 px-5 w-full">
                            <div class="flex flex-col gap-2 items-center">
                                <span class="text-lg font-bold">
                                    {format!(
                                        "You've successfully claimed {formatted_cents} Cents.",
                                    )}
                                </span>
                                <span class="text-neutral-300">
                                    Your wallet has been updated with {formatted_dolr}DOLR.
                                </span>
                            </div>
                            <a class="py-2 px-5 font-bold text-center bg-white rounded-lg" href="/">
                                <span class="text-transparent bg-clip-text bg-brand-gradient">
                                    Continue Playing
                                </span>
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    })
}
