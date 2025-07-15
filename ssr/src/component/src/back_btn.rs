use gloo::history::{BrowserHistory, History};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;
use reqwest::Url;

use utils::event_streaming::events::HistoryCtx;

/// Go back or navigate to a fallback route
/// does nothing in ssr mode
/// ideal for calling from a button, for example
pub fn go_back_or_fallback(fallback: &str) {
    #[cfg(not(feature = "hydrate"))]
    {
        return;
    }
    #[cfg(feature = "hydrate")]
    {
        let history_ctx = expect_context::<HistoryCtx>();
        let win = window();
        let referrer = win
            .document()
            .map(|d| d.referrer())
            .and_then(|r| Url::parse(&r).ok());
        let cur_url = Url::parse(&win.location().href().unwrap_or_default()).ok();

        // HACK: completely remove history ctx eventually
        if cur_url.as_ref().and_then(|u| u.host_str())
            == referrer.as_ref().and_then(|r| r.host_str())
            || history_ctx.len() > 1
        {
            let history = BrowserHistory::new();
            history.back();
        } else {
            use_navigate()(fallback, Default::default());
        }
    }
}

#[component]
pub fn BackButton(#[prop(into)] fallback: Signal<String>) -> impl IntoView {
    view! {
        <button
            on:click=move |_| go_back_or_fallback(&fallback.get_untracked())
            class="items-center"
        >
            <Icon icon=icondata::AiLeftOutlined />
        </button>
    }
}
