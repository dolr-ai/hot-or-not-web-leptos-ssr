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
        <div class="flex flex-col justify-center items-center bg-black w-dvw h-dvh">
            <img src="/img/common/error-logo.svg" />
            <h1 class="p-2 text-2xl font-bold text-white md:text-3xl">"oh no!"</h1>
            <div class="px-8 mb-4 w-full text-xs text-center resize-none md:w-2/3 md:text-sm lg:w-1/3 text-white/60">
                {error}
            </div>
            <button
                on:click=move |_| go_back()
                class="py-4 px-12 mt-6 max-w-full text-lg text-white rounded-full md:text-xl bg-primary-600"
            >
                Go back
            </button>
        </div>
    }
}
