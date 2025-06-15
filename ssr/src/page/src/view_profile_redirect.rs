use component::spinner::FullScreenSpinner;
use leptos::prelude::*;
use leptos_router::components::Redirect;
use state::canisters::auth_state;

#[component]
pub fn ProfileInfo() -> impl IntoView {
    let auth = auth_state();
    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || {
                auth
                    .user_principal
                    .get()
                    .map(|res| match res {
                        Ok(user_principal) => view! { <Redirect path=user_principal.to_text() /> },
                        Err(e) => view! { <Redirect path=format!("/error?err={e}") /> },
                    })
            }}
        </Suspense>
    }
}
