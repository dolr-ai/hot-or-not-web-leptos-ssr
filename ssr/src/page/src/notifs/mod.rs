use leptos::{either::Either, prelude::*};
use leptos_router::components::Redirect;
use state::canisters::auth_state;

use utils::notifications::get_device_registeration_token;
use yral_canisters_common::utils::profile::ProfileDetails;
use yral_metadata_client::MetadataClient;

#[component]
fn NotifInnerComponent(details: ProfileDetails) -> impl IntoView {
    let auth_state = auth_state();

    let on_token_click: Action<(), ()> = Action::new_unsync(move |()| async move {
        let metaclient: MetadataClient<false> = MetadataClient::default();

        let cans = auth_state.auth_cans(expect_context()).await.unwrap();

        let token = get_device_registeration_token().await.unwrap();
        metaclient
            .register_device(cans.identity(), token)
            .await
            .unwrap();
    });

    view! {
        <h1>"YRAL Notifs for"</h1>
        <h2>{details.username_or_principal()}</h2>
        <br />
        <div class="flex flex-row gap-2 text-black">
            <button
                class="p-2 bg-gray-200 rounded-md"
                on:click=move |_| {
                    on_token_click.dispatch(());
                }
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
        <div class="grid grid-cols-1 justify-items-center place-content-center w-screen h-screen">
            <Suspense>
                {move || Suspend::new(async move {
                    let res = auth.cans_wire().await;
                    match res {
                        Ok(cans) => {
                            Either::Left(
                                view! { <NotifInnerComponent details=cans.profile_details /> },
                            )
                        }
                        Err(e) => {
                            Either::Right(view! { <Redirect path=format!("/error?err={e}") /> })
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
