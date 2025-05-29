use super::{
    auth_providers::LoginProviders,
    overlay::{ShadowOverlay, ShowOverlay},
};
use leptos::prelude::*;

#[component]
pub fn LoginModal(
    #[prop(into)] show: RwSignal<bool>,
    redirect_to: Option<String>,
) -> impl IntoView {
    let lock_closing = RwSignal::new(false);
    view! {
        <ShadowOverlay show=ShowOverlay::MaybeClosable {
            show,
            closable: lock_closing,
        }>
            <LoginProviders show_modal=show lock_closing redirect_to=redirect_to.clone() />
        </ShadowOverlay>
    }
}
