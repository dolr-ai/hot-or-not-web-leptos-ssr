mod history;

use candid::Principal;
use gloo::timers::callback::Timeout;
use ic_agent::Identity;
use leptos::{prelude::*, reactive::wrappers::write::SignalSetter};
use leptos_icons::*;
use leptos_meta::*;
use leptos_router::hooks::query_signal;
use leptos_use::use_window;

use component::canisters_prov::AuthCansProvider;
use component::connect::ConnectLogin;
use component::{
    back_btn::BackButton, buttons::HighlightedButton, dashbox::DashboxLoading, title::TitleText,
};
use history::HistoryView;
use state::app_state::AppState;
use utils::event_streaming::events::{account_connected_reader, auth_canisters_store};
use utils::event_streaming::events::{Refer, ReferShareLink};
use utils::web::{copy_to_clipboard, share_url};

#[component]
fn WorkButton(#[prop(into)] text: String, #[prop(into)] head: String) -> impl IntoView {
    view! {
        <div class="flex flex-1 flex-col items-center gap-3 bg-neutral-900 rounded-md px-2 py-4">
            <div class="grid place-items-center rounded-sm text-xs md:text-sm font-bold">{head}</div>
            <span class="text-xs md:text-sm">{text}</span>
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

    let (logged_in, _) = account_connected_reader();
    let show_copied_popup = RwSignal::new(false);
    let canister_store = auth_canisters_store();

    let click_copy = Action::new(move |refer_link: &String| {
        let refer_link = refer_link.clone();

        async move {
            let _ = copy_to_clipboard(&refer_link);

            ReferShareLink.send_event(logged_in, canister_store);

            show_copied_popup.set(true);
            Timeout::new(1200, move || show_copied_popup.set(false)).forget();
        }
    });
    let refer_link_share = refer_link.clone();
    let handle_share = move || {
        let url = &refer_link_share;
        if share_url(url).is_some() {
            return;
        }
        click_copy.dispatch(url.clone());
    };

    view! {
        <div class="flex w-full justify-between">
            <div class="flex flex-1 items-center w-full rounded-md border-dashed border-2 p-3 gap-2 border-neutral-700 bg-neutral-900">
                <span class="text-md lg:text-lg text-ellipsis line-clamp-1">{refer_link.clone()}</span>
                <button on:click=move |_| { click_copy.dispatch(refer_link.clone()); }>
                    <Icon attr:class="text-xl text-primary-500" icon=icondata::IoCopyOutline />
                </button>
            </div>
            <HighlightedButton
            alt_style=false
            disabled=false
            on_click=move || { handle_share() }>
                Share
            </HighlightedButton>
        </div>

        <Show when=show_copied_popup>
            <div class="absolute flex flex-col justify-center items-center z-[4]">
                <span class="absolute top-28 flex flex-row justify-center items-center bg-white/90 rounded-md h-10 w-28 text-center shadow-lg">
                    <p class="text-black">Link Copied!</p>
                </span>
            </div>
        </Show>
    }
}

#[component]
fn ReferCode() -> impl IntoView {
    view! {
        <AuthCansProvider fallback=DashboxLoading let:cans>
            <ReferLoaded user_principal=cans.identity().sender().unwrap() />
        </AuthCansProvider>
    }
}

#[component]
fn ReferView() -> impl IntoView {
    let (logged_in, _) = account_connected_reader();

    Refer.send_event(logged_in);

    view! {
        <div class="flex flex-col w-full h-full items-center text-white gap-10">
            <div class="absolute inset-x-0 top-0 z-0 w-full max-w-md mx-auto" style="filter: blur(1.5px);">
                <img src="/img/common/refer-bg.webp" class="w-full object-cover" />
            </div>
            <div style="height: 19rem;" class="flex z-[1] relative justify-center w-full items-center gap-4 overflow-visible">
                <img class="shrink-0 h-40 select-none" src="/img/common/wallet.webp" />
                <div style="background: radial-gradient(circle, hsla(327, 99%, 45%, 0.3) 0%, transparent 70%)" class="absolute z-0 inset-0"></div>
                <img src="/img/common/bitcoin-logo.svg" class="absolute top-8 left-5 size-6" style="filter: blur(1px); transform: rotate(30deg);" />
                <img src="/img/common/bitcoin-logo.svg" class="absolute top-16 right-3 size-6" style="filter: blur(1px); transform: rotate(40deg);" />
                <img src="/img/common/bitcoin-logo.svg" class="absolute bottom-4 left-6 size-9" style="filter: blur(1px); transform: rotate(-60deg);" />
            </div>
            <div class="flex flex-col w-full z-[1] items-center gap-4 text-center">
                <span class="font-bold text-2xl">Invite & get Bitcoin <span style="color: #A3A3A3">(500 SATS)</span></span>
            </div>
            <div class="flex flex-col w-full z-[1] gap-2 px-4 text-white items-center">
                <Show when=logged_in fallback=|| view! { <ConnectLogin cta_location="refer" /> }>
                    <ReferCode />
                </Show>
            </div>
            <div class="flex flex-col w-full z-[1] items-center gap-8 mt-4">
                <span class="font-xl font-semibold">HOW IT WORKS?</span>
                <div class="flex flex-row gap-8 text-center">
                    <WorkButton
                        text="Share your link
                        with a friend"
                        head="STEP 1"
                    />
                    <WorkButton
                        text="Your friend uses
                        logs in from the link"
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
fn TabSelector(
    tab_idx: i32,
    text: String,
    tab_str: String,
    current_tab: Memo<i32>,
    set_cur_tab: SignalSetter<Option<String>>,
) -> impl IntoView {
    let button_class = move || {
        if tab_idx == current_tab() {
            "text-white font-bold"
        } else {
            "text-white/50 font-bold"
        }
    };
    let selector_class = move || {
        if tab_idx == current_tab() {
            "bg-primary-500 w-2 h-2 rounded-full"
        } else {
            "bg-transparent w-2 h-2 rounded-full"
        }
    };

    view! {
        <div class="flex w-full flex-col items-center gap-y-2">
            <button class=button_class on:click=move |_| set_cur_tab(Some(tab_str.clone()))>
                {text}
            </button>
            <div class=selector_class></div>
        </div>
    }
}

#[component]
fn ListSwitcher() -> impl IntoView {
    let (cur_tab, set_cur_tab) = query_signal::<String>("tab");
    let current_tab = Memo::new(move |_| {
        cur_tab.with(|cur_tab| match cur_tab.as_deref() {
            Some("how-to") => 0,
            Some("history") => 1,
            _ => 0,
        })
    });

    view! {
        <div class="flex flex-row w-full text-md md:text-lg lg:text-xl text-center">
            <TabSelector
                text="How to earn".into()
                tab_idx=0
                tab_str="how-to".to_string()
                current_tab
                set_cur_tab=set_cur_tab
            />
            <TabSelector
                text="History".into()
                tab_idx=1
                tab_str="history".to_string()
                current_tab
                set_cur_tab=set_cur_tab
            />
        </div>
        <div class="flex flex-row justify-center">
            <Show when=move || current_tab() == 0 fallback=HistoryView>
                <ReferView />
            </Show>
        </div>
    }
}

#[component]
pub fn ReferEarn() -> impl IntoView {
    let (logged_in, _) = account_connected_reader();

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
                <Show when=logged_in fallback=ReferView>
                    <ListSwitcher />
                </Show>
            </div>
        </div>
    }
}
