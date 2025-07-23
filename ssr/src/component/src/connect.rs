use leptos::prelude::*;
use state::canisters::auth_state;

use crate::buttons::HighlightedButton;
use utils::{
    event_streaming::events::{LoginCta, LoginJoinOverlayViewed},
    mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps},
};

use super::login_modal::LoginModal;

#[component]
pub fn ConnectLogin(
    #[prop(optional, default = "Login")] login_text: &'static str,
    #[prop(optional, default = "menu")] cta_location: &'static str,
    #[prop(optional, default = RwSignal::new(false))] show_login: RwSignal<bool>,
    #[prop(optional, into)] redirect_to: Option<String>,
) -> impl IntoView {
    let auth = auth_state();
    LoginJoinOverlayViewed.send_event(auth.event_ctx());

    let login_click_action = Action::new(move |()| async move {
        LoginCta.send_event(cta_location.to_string());
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx()) {
            let page_name = global.page_name();
            MixPanelEvent::track_signup_clicked(global, page_name);
        }
    });

    view! {
        <HighlightedButton
            classes="w-full".to_string()
            alt_style=false
            disabled=false
            on_click=move || {
                show_login.set(true);
                login_click_action.dispatch(());
            }
        >
            {move || if show_login() { "Connecting..." } else { login_text }}
        </HighlightedButton>
        <LoginModal show=show_login redirect_to />
    }
}
