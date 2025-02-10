use leptos::{component, view, IntoView, Params, SignalGetUntracked};
use leptos_router::{use_query, Params};

use crate::try_or_redirect_opt;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Params)]
struct FailureParams {
    gdolr: u64,
}

#[component]
pub fn Failure() -> impl IntoView {
    let params = use_query::<FailureParams>();
    let FailureParams { gdolr } = try_or_redirect_opt!(params.get_untracked());
    Some(view! {
        <div
            style:background-image="url('/img/gradient-grayscale.png')"
            class="min-h-screen w-full flex flex-col text-white pt-2 pb-12 bg-black items-center relative max-md:bg-[length:271vw_100vh] md:bg-[length:max(100vw,100vh)] max-md:bg-[position:-4.5vw_-6.5vh] md:bg-bottom"
        >
            <div id="back-nav" class="flex flex-col items-center w-full gap-20 pb-16"></div>
            <div class="w-full">
                <div class="max-w-md w-full mx-auto px-4 mt-4 pb-6 absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
                    <div class="w-full flex flex-col gap-12 items-center">
                        <img class="max-w-44" src="/img/cross-3d.png" />
                        <div class="flex flex-col gap-8 w-full px-5">
                            <div class="flex flex-col gap-2 items-center">
                                <span class="font-bold text-lg">OOPS!</span>
                                <span class="text-neutral-300">Failed to claim {gdolr} gDOLR!</span>
                            </div>
                            <a class="rounded-lg px-5 py-2 text-center font-bold bg-brand-gradient text-white" href="/pnd/withdraw">
                                Try Again
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    })
}
