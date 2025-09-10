use super::{
    auth_providers::LoginProviders,
    overlay::{ShadowOverlay, ShowOverlay},
};
use leptos::prelude::*;

#[component]
pub fn LoginModal(
    #[prop(into)] show: RwSignal<bool>,
    redirect_to: Option<String>,
    #[prop(optional, into)] reload_window: bool,
    #[prop(optional)] text: String,
) -> impl IntoView {
    let lock_closing = RwSignal::new(false);
    view! {
        <ShadowOverlay show=ShowOverlay::MaybeClosable {
            show,
            closable: lock_closing,
        }>
            <LoginProviders show_modal=show lock_closing redirect_to=redirect_to.clone() reload_window text=text.clone() />
        </ShadowOverlay>
    }
}
