use leptos::{either::Either, prelude::*};
use leptos_router::components::Redirect;
use state::canisters::auth_state;
use utils::notifications::get_token_for_principal;

use yral_canisters_common::utils::profile::ProfileDetails;

#[component]
fn NotifInnerComponent(details: ProfileDetails) -> impl IntoView {
    let on_token_click: Action<(), (), LocalStorage> = Action::new_unsync(move |()| async move {
        get_token_for_principal(details.principal.to_string()).await;
    });

    view! {
        <h1>"YRAL Notifs for"</h1>
        <h2>{details.username_or_principal()}</h2>
        <br />
        <div class="flex flex-row gap-2 text-black">
            <button
                class="p-2 bg-gray-200 rounded-md"
                on:click=move |_| {on_token_click.dispatch(());}
            >
                "Get Token"
            </button>
        </div>
    }
}

#[component]
pub fn Notif() -> impl IntoView {
    let auth = auth_state();
    view! {
        <div class="h-screen w-screen grid grid-cols-1 justify-items-center place-content-center">
            <Suspense>
            {move || Suspend::new(async move {
                let res = auth.cans_wire().await;
                match res {
                    Ok(cans) => Either::Left(view! {
                        <NotifInnerComponent details=cans.profile_details />
                    }),
                    Err(e) => Either::Right(view! {
                        <Redirect path=format!("/error?err={e}") />
                    })
                }
            })}
            </Suspense>
        </div>
    }
}
