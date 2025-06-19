use candid::Principal;
use consts::NEW_USER_SIGNUP_REWARD;
use gloo::timers::callback::Timeout;
use hon_worker_common::limits::REFERRAL_REWARD;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_meta::*;
use leptos_router::components::Redirect;
use leptos_use::use_window;

use component::connect::ConnectLogin;
use component::{back_btn::BackButton, buttons::HighlightedButton, title::TitleText};
use state::app_state::AppState;
use state::canisters::auth_state;
use utils::event_streaming::events::{Refer, ReferShareLink};
use utils::send_wrap;
use utils::web::copy_to_clipboard;
use yral_types::delegated_identity::DelegatedIdentityWire;

async fn delete_user_test(identity: DelegatedIdentityWire) -> Result<(), ServerFnError> {
    use reqwest::Client;
    use serde_json::json;

    let client = Client::new();
    let body = json!({
        "delegated_identity_wire": identity
    });

    let response = client
        .delete("https://pr-214-dolr-ai-off-chain-agent.fly.dev/api/v1/user")
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::ServerError(format!(
            "Delete user failed with status: {}",
            response.status()
        )))
    }
}

#[component]
fn WorkButton(#[prop(into)] text: String, #[prop(into)] head: String) -> impl IntoView {
    view! {
        <div class="flex flex-col flex-1 gap-3 justify-center items-center py-4 px-3 text-xs rounded-md lg:flex-row lg:py-5 lg:px-4 bg-neutral-900 lg:text-md">
            <div class="font-bold whitespace-nowrap text-neutral-50">{head}</div>
            <span class="text-neutral-400">{text}</span>
        </div>
    }
}

fn share(url: &str, text: &str) -> Option<()> {
    #[cfg(not(feature = "hydrate"))]
    {
        _ = url;
        None
    }
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;
        use web_sys::{js_sys::Reflect, ShareData};
        let window = use_window();
        let nav = window.navigator()?;
        if !Reflect::has(&nav.clone().into(), &JsValue::from_str("share")).unwrap_or_default() {
            return None;
        }
        let share_data = ShareData::new();
        share_data.set_title(text);
        share_data.set_url(url);
        _ = nav.share_with_data(&share_data);
        Some(())
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

        send_wrap(async move {
            let _ = copy_to_clipboard(&refer_link);

            // Call the delete user test server function
            if let Ok(identity) = auth.user_identity.await {
                let res = delete_user_test(identity).await;
                leptos::logging::log!("delete_user_test: {:?}", res);
            }

            ReferShareLink.send_event(ev_ctx);

            show_copied_popup.set(true);
            Timeout::new(1200, move || show_copied_popup.set(false)).forget();
        })
    });
    let refer_link_share = refer_link.clone();
    let handle_share = move || {
        let text = format!("Join YRAL—the world's 1st social platform on BITCOIN\nGet FREE BITCOIN ({NEW_USER_SIGNUP_REWARD} SATS) Instantly\nAdditional BITCOIN ({REFERRAL_REWARD} SATS) when you log in using the link.");
        if share(&refer_link_share, &text).is_some() {
            return;
        }
        click_copy.dispatch(refer_link_share.clone());
    };

    view! {
        <div class="flex gap-2 justify-between w-full z-[1]">
            <div class="flex flex-1 gap-2 items-center p-3 w-full rounded-md border-2 border-dashed border-neutral-700 bg-neutral-900">
                <span class="lg:text-lg text-md text-ellipsis line-clamp-1 text-neutral-500">
                    {refer_link.clone()}
                </span>
                <button
                    style="filter: invert(1)"
                    on:click=move |_| {
                        click_copy.dispatch(refer_link.clone());
                    }
                >
                    <Icon attr:class="text-xl" icon=icondata::IoCopyOutline />
                </button>
            </div>
            <HighlightedButton
                classes="!w-fit".to_string()
                alt_style=false
                disabled=false
                on_click=move || { handle_share() }
            >
                Share
            </HighlightedButton>
        </div>

        <Show when=show_copied_popup>
            <div class="flex absolute flex-col justify-center items-center z-4">
                <span class="flex absolute top-28 flex-row justify-center items-center w-28 h-10 text-center rounded-md shadow-lg bg-white/90">
                    <p class="text-black">Link Copied!</p>
                </span>
            </div>
        </Show>
    }
}

#[component]
fn ReferLoading() -> impl IntoView {
    view! {
        <div class="flex flex-col flex-1 gap-3 justify-center items-center py-4 px-3 text-xs rounded-md animate-pulse lg:flex-row lg:py-5 lg:px-4 bg-neutral-900 lg:text-md"></div>
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
                    Ok(user_principal) => Either::Left(view! { <ReferLoaded user_principal /> }),
                    Err(e) => Either::Right(view! { <Redirect path=format!("/error?err={e}") /> }),
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
        <div class="flex flex-col gap-5 items-center w-full h-full text-white">
            <div
                class="absolute inset-x-0 top-0 z-0 mx-auto w-full max-w-md"
                style="filter: blur(1.5px);"
            >
                <img src="/img/common/refer-bg.webp" class="object-cover w-full" />
            </div>
            <div
                style="height: 14rem;"
                class="flex overflow-visible relative gap-4 justify-center items-center w-full z-[1]"
            >
                <img class="h-32 select-none shrink-0" src="/img/common/wallet.webp" />
                <img
                    src="/img/common/bitcoin.webp"
                    class="absolute left-5 top-8 size-6"
                    style="filter: blur(1px); transform: rotate(30deg);"
                />
                <img
                    src="/img/common/bitcoin.webp"
                    class="absolute right-3 top-16 size-6"
                    style="filter: blur(1px); transform: rotate(40deg);"
                />
                <img
                    src="/img/common/bitcoin.webp"
                    class="absolute bottom-4 left-6 size-9"
                    style="filter: blur(0.3px); transform: rotate(-60deg);"
                />
            </div>
            <div
                style="background: radial-gradient(circle, hsla(327, 99%, 45%, 0.3) 0%, transparent 70%); height:29rem"
                class="absolute inset-x-0 top-16 z-0"
            ></div>

            <div class="flex flex-col gap-4 items-center w-full text-center z-[1]">
                <span class="text-xl font-bold md:text-2xl">
                    Invite & get Bitcoin
                    <span style="color: #A3A3A3">"("{REFERRAL_REWARD} " SATS)"</span>
                </span>
            </div>
            <div class="flex flex-col gap-2 items-center px-4 w-full text-white z-[1]">
                <Show when=logged_in fallback=|| view! { <ConnectLogin cta_location="refer" /> }>
                    <ReferCode />
                </Show>
            </div>
            <div class="flex flex-col gap-6 items-center pb-5 mt-2 w-full z-[1]">
                <span class="font-semibold font-xl">How it works?</span>
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
                        text=format!("You both earn Bitcoin ({REFERRAL_REWARD} SATS)")
                        head="STEP 3"
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ReferEarn() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Refer & Earn";
    view! {
        <Title text=page_title.clone() />

        <div class="flex flex-col items-center pt-2 pb-12 bg-black min-w-dvw min-h-dvh">
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
