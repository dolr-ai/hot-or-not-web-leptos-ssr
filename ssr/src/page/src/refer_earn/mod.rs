mod history;

use candid::Principal;
use gloo::timers::callback::Timeout;
use ic_agent::Identity;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;
use leptos::either::Either;
use leptos_router::components::Redirect;
use leptos_router::hooks::query_signal;
use leptos_use::use_window;

use component::connect::ConnectLogin;
use component::{back_btn::BackButton, buttons::HighlightedButton, title::TitleText};
use state::app_state::AppState;
use state::canisters::auth_state;
use utils::event_streaming::events::{Refer, ReferShareLink};
use utils::web::{copy_to_clipboard, share_url};

#[component]
fn WorkButton(#[prop(into)] text: String, #[prop(into)] head: String) -> impl IntoView {
    view! {
        <div class="flex flex-1 flex-col lg:flex-row items-center justify-center text-xs lg:text-md gap-3 bg-neutral-900 rounded-md px-3 lg:px-4 lg:py-5 py-4">
            <div class="font-bold text-neutral-50 whitespace-nowrap">{head}</div>
            <span class="text-neutral-400">{text}</span>
        </div>
    }
}

#[component]
fn ReferLoaded(user_principal: Principal) -> impl IntoView {
    let window = use_window();
    let refer_link = window
        .as_ref()
        .and_then(|w| {
            let origin = w.location().origin().ok()?;
            Some(format!(
                "{}/?user_refer={}",
                origin,
                user_principal.to_text()
            ))
        })
        .unwrap_or_default();

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    let show_copied_popup = RwSignal::new(false);

    let click_copy = Action::new(move |refer_link: &String| {
        let refer_link = refer_link.clone();

        async move {
            let _ = copy_to_clipboard(&refer_link);

            ReferShareLink.send_event(ev_ctx);

            show_copied_popup.set(true);
            Timeout::new(1200, move || show_copied_popup.set(false)).forget();
        }
    });
    let refer_link_share = refer_link.clone();
    let handle_share = move || {
        let url = format!("Join YRAL—the world's 1st social platform on BITCOIN\nGet FREE BITCOIN (1000 SATS) Instantly\nAdditional BITCOIN (500 SATS) when you log in using {refer_link_share}");
        if share_url(&url).is_some() {
            return;
        }
        click_copy.dispatch(url.clone());
    };

    view! {
        <div class="flex z-[1] w-full gap-2 justify-between">
            <div class="flex flex-1 items-center w-full rounded-md border-dashed border-2 p-3 gap-2 border-neutral-700 bg-neutral-900">
                <span class="text-md lg:text-lg text-ellipsis line-clamp-1 text-neutral-500">{refer_link.clone()}</span>
                <button style="filter: invert(1)" on:click=move |_| { click_copy.dispatch(refer_link.clone()); }>
                    <Icon attr:class="text-xl" icon=icondata::IoCopyOutline />
                </button>
            </div>
            <HighlightedButton
            classes="!w-fit".to_string()
            alt_style=false
            disabled=false
            on_click=move || { handle_share() }>
                Share
            </HighlightedButton>
        </div>

        <Show when=show_copied_popup>
            <div class="absolute flex flex-col justify-center items-center z-[10]">
                <span class="absolute top-28 flex flex-row justify-center items-center bg-white/90 rounded-md h-10 w-28 text-center shadow-lg">
                    <p class="text-black">Link Copied!</p>
                </span>
            </div>
        </Show>
    }
}

#[component]
fn ReferLoading() -> impl IntoView {
    view! {
        <div class="flex flex-1 flex-col lg:flex-row items-center justify-center text-xs lg:text-md gap-3 bg-neutral-900 rounded-md px-3 lg:px-4 lg:py-5 py-4 animate-pulse">
        </div>
    }
}

#[component]
fn ReferCode() -> impl IntoView {
    let auth = auth_state();
    view! {
        <Suspense fallback=ReferLoading>
        {move || Suspend::new(async move {
            let res = auth.user_principal.await;
            match res {
                Ok(user_principal) => {
                    Either::Left(view! {
                        <ReferLoaded user_principal />
                    })
                }
                Err(e) => {
                    Either::Right(view! {
                        <Redirect path=format!("/error?err={e}") />
                    })
                }
            }
        })}
        </Suspense>
    }
}

#[component]
fn ReferView() -> impl IntoView {
    let auth_state = auth_state();
    let logged_in = auth_state.is_logged_in_with_oauth();

    Refer.send_event(auth_state.event_ctx());

    view! {
        <div class="flex flex-col w-full h-full items-center text-white gap-10">
            <div class="absolute inset-x-0 top-0 z-0 w-full max-w-md mx-auto" style="filter: blur(1.5px);">
                <img src="/img/common/refer-bg.webp" class="w-full object-cover" />
            </div>
            <div style="height: 19rem;" class="flex z-[1] relative justify-center w-full items-center gap-4 overflow-visible">
                <img class="shrink-0 h-32 select-none" src="/img/common/wallet.webp" />
                <img src="/img/common/bitcoin-logo.svg" class="absolute top-8 left-5 size-6" style="filter: blur(1px); transform: rotate(30deg);" />
                <img src="/img/common/bitcoin-logo.svg" class="absolute top-16 right-3 size-6" style="filter: blur(1px); transform: rotate(40deg);" />
                <img src="/img/common/bitcoin-logo.svg" class="absolute bottom-4 left-6 size-9" style="filter: blur(1px); transform: rotate(-60deg);" />
            </div>
            <div style="background: radial-gradient(circle, hsla(327, 99%, 45%, 0.3) 0%, transparent 70%); height:29rem" class="absolute z-0 inset-x-0 top-16"></div>

            <div class="flex flex-col w-full z-[1] items-center gap-4 text-center">
                <span class="font-bold text-xl md:text-2xl">Invite & get Bitcoin <span style="color: #A3A3A3">(500 SATS)</span></span>
            </div>
            <div class="flex flex-col w-full z-[1] gap-2 px-4 text-white items-center">
                <Show when=logged_in fallback=|| view! { <ConnectLogin cta_location="refer" /> }>
                    <ReferCode />
                </Show>
            </div>
            <div class="flex flex-col w-full z-[1] items-center gap-8 mt-4">
                <span class="font-xl font-semibold">How it works?</span>
                <div class="flex flex-row gap-4 text-center">
                    <WorkButton
                        text="Share your link
                        with a friend"
                        head="STEP 1"
                    />
                    <WorkButton
                        text="Your friend logs
                        in from the link"
                        head="STEP 2"
                    />
                    <WorkButton
                        text="You both earn
                        Bitcoin (500 SATS)"
                        head="STEP 3"
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ReferEarn() -> impl IntoView {
    let auth = auth_state();
    let logged_in = auth.is_logged_in_with_oauth();

    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Refer & Earn";
    view! {
        <Title text=page_title />
        <div class="flex flex-col items-center min-w-dvw min-h-dvh bg-black pt-2 pb-12 gap-6">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/menu".to_string() />
                    <span class="text-lg font-bold text-white">Refer & Earn</span>
                    <div></div>
                </div>
            </TitleText>
            <div class="px-8 w-full sm:w-7/12">
                <div class="flex flex-row justify-center">
                    <ReferView />
                </div>
            </div>
        </div>
    }
}
