use candid::Nat;
use leptos::{component, prelude::*, view, IntoView, Params};
use leptos_router::{hooks::use_query, params::Params};
use utils::try_or_redirect_opt;
use yral_canisters_common::utils::token::balance::TokenBalance;

#[derive(Debug, PartialEq, Eq, Clone, Params)]
struct FailureParams {
    cents: Nat,
}

#[component]
pub fn Failure() -> impl IntoView {
    let params = use_query::<FailureParams>();
    let FailureParams { cents } = try_or_redirect_opt!(params.get_untracked());
    let formatted_cents = TokenBalance::new(cents.clone(), 6).humanize_float_truncate_to_dp(4);
    Some(view! {
        <div
            style:background-image="url('/img/yral/onboarding-bg-grayscale.webp')"
            class="flex relative flex-col items-center pt-2 pb-12 w-full min-h-screen text-white bg-black md:bg-bottom max-md:bg-size-[271vw_100vh] max-md:bg-position-[-4.5vw_-6.5vh] md:bg-size-[max(100vw,100vh)]"
        >
            <div id="back-nav" class="flex flex-col gap-20 items-center pb-16 w-full"></div>
            <div class="w-full">
                <div class="absolute top-1/2 left-1/2 px-4 pb-6 mx-auto mt-4 w-full max-w-md -translate-x-1/2 -translate-y-1/2">
                    <div class="flex flex-col gap-12 items-center w-full">
                        <img class="max-w-44" src="/img/yral/cross-3d.webp" />
                        <div class="flex flex-col gap-8 px-5 w-full">
                            <div class="flex flex-col gap-2 items-center">
                                <span class="text-lg font-bold">OOPS!</span>
                                <span class="text-neutral-300">
                                    Failed to claim {formatted_cents}Cents!
                                </span>
                            </div>
                            <a
                                class="py-2 px-5 font-bold text-center text-white rounded-lg bg-brand-gradient"
                                href="/pnd/withdraw"
                            >
                                Try Again
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    })
}
