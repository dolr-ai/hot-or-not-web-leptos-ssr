use component::canisters_prov::AuthCansProvider;
use leptos::prelude::*;
use state::canisters::authenticated_canisters;
use utils::event_streaming::events::account_connected_reader;
use utils::notifications::get_device_registeration_token;

use yral_canisters_common::utils::profile::ProfileDetails;
use yral_canisters_common::Canisters;
use yral_metadata_client::MetadataClient;

#[component]
fn NotifInnerComponent(details: ProfileDetails) -> impl IntoView {
    let (_, _) = account_connected_reader();

    let auth_cans = authenticated_canisters();

    let on_token_click: Action<(), (), LocalStorage> = Action::new_unsync(move |()| async move {
        let metaclient: MetadataClient<false> = MetadataClient::with_base_url(
            reqwest::Url::parse("https://pr-31-dolr-ai-yral-metadata.fly.dev/").unwrap(),
        );

        let cans = Canisters::from_wire(auth_cans.await.unwrap(), expect_context()).unwrap();

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
                on:click=move |_| {on_token_click.dispatch(());}
            >
                "Get Token"
            </button>
        </div>
    }
}

#[component]
pub fn Notif() -> impl IntoView {
    view! {
        <div class="h-screen w-screen grid grid-cols-1 justify-items-center place-content-center">
            <AuthCansProvider let:cans>
                <NotifInnerComponent details=cans.profile_details() />
            </AuthCansProvider>
        </div>
    }
}
