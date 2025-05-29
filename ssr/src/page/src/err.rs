use gloo::history::{BrowserHistory, History};
use leptos::prelude::*;
use leptos_router::hooks::use_query;
use leptos_router::params::Params;
use state::canisters::auth_state;
use utils::event_streaming::events::ErrorEvent;

#[derive(Clone, Params, PartialEq)]
struct ServerErrParams {
    err: String,
}

impl ServerErrParams {
    fn map_to_err(&self) -> String {
        match self.err.as_str() {
            _ if self.err.contains("IC agent error") || self.err.contains("error running server function") || self.err.contains("Canister error") || self.err.contains("http fetch error") || self.err.contains("ServerError") || self.err.contains("TypeError") || self.err.contains("CanisterError") => "It looks like our system is taking a coffee break. Try again in a bit, and we'll have it back to work!".to_string(),
            _ => self.err.clone(),
        }
    }
}

#[component]
pub fn ServerErrorPage() -> impl IntoView {
    let params = use_query::<ServerErrParams>();
    let error = Signal::derive(move || {
        params
            .get()
            .map(|p| p.map_to_err())
            .unwrap_or_else(|_| "Server Error".to_string())
    });

    let error_str = params
        .get()
        .map(|p| p.err.clone())
        .unwrap_or_else(|_| "Server Error".to_string());

    let error_str_clone = error_str.clone();
    Effect::new(move |_| {
        let _ = js_sys::eval(&format!(
            r#"
            window.Sentry &&
                        Sentry.onLoad(function () {{
                                Sentry.captureException(new Error("{}"));
                        }});
            "#,
            &error_str_clone
        ));
    });

    let auth = auth_state();
    ErrorEvent.send_event(auth.event_ctx(), error_str);

    view! { <ErrorView error /> }
}

#[component]
pub fn ErrorView(#[prop(into)] error: Signal<String>) -> impl IntoView {
    let go_back = move || {
        let history = BrowserHistory::new();

        //go back
        history.back();
    };

    view! {
        <div class="flex flex-col w-dvw h-dvh bg-black justify-center items-center">
            <img src="/img/common/error-logo.svg" />
            <h1 class="p-2 text-2xl md:text-3xl font-bold text-white">"oh no!"</h1>
            <div class="text-center text-xs md:text-sm text-white/60 w-full md:w-2/3 lg:w-1/3 resize-none px-8 mb-4">
                {error}
            </div>
            <button
                on:click=move |_| go_back()
                class="bg-primary-600 rounded-full mt-6 py-4 px-12 max-w-full text-white text-lg md:text-xl"
            >
                Go back
            </button>
        </div>
    }
}
