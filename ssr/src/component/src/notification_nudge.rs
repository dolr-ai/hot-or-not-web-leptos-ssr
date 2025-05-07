use codee::string::FromToStringCodec;
use consts::NOTIFICATIONS_ENABLED_STORE;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_use::storage::use_local_storage;
use state::canisters::authenticated_canisters;
use utils::notifications::get_device_registeration_token;
use yral_canisters_common::Canisters;
use yral_metadata_client::MetadataClient;

use crate::{
    buttons::HighlightedButton, icons::notification_nudge::NotificationNudgeIcon,
    overlay::ShadowOverlay,
};

#[component]
pub fn NotificationNudge(pop_up: RwSignal<bool>) -> impl IntoView {
    let cans = authenticated_canisters();

    let (notifs_enabled, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let popup_signal = Signal::derive(move || !notifs_enabled.get() && pop_up.get());

    let notification_action: Action<(), (), LocalStorage> =
        Action::new_unsync(move |()| async move {
            let metaclient = MetadataClient::default();
            let cans = Canisters::from_wire(cans.await.unwrap(), expect_context()).unwrap();

            // Removed send_wrap as get_device_registeration_token involves !Send JS futures
            let token = get_device_registeration_token().await.unwrap();

            metaclient
                .register_device(cans.identity(), token)
                .await
                .unwrap();

            set_notifs_enabled(true);
        });
    view! {
        <ShadowOverlay show=popup_signal >
            <div class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-neutral-900 text-white p-8 rounded-lg shadow-xl w-full min-w-[343px] max-w-[550px]">
                <button
                    on:click=move |_| pop_up.set(false)
                    class="absolute top-3 right-3 p-1 bg-neutral-800 rounded-full text-neutral-300 hover:text-white transition-colors">
                    <Icon icon=icondata::IoClose attr:class="w-6 h-6" />
                </button>

                <div class="flex flex-col items-center text-center gap-4 pt-4">
                    <Icon icon=NotificationNudgeIcon attr:class="w-32 h-32 mb-2 text-orange-500" />
                    <h1 class="text-2xl font-bold mb-2">"Stay in the Loop!"</h1>
                    <p class="text-neutral-400 text-lg mb-6 max-w-xs font-light">
                        "Your video is processing in the background. Enable notifications so you don\'t miss a beat — feel free to explore the app while we handle the upload!"
                    </p>
                    <HighlightedButton
                            alt_style=false
                            on_click=move || {notification_action.dispatch(());}
                        classes="w-full py-3 bg-gradient-to-r from-fuchsia-600 to-pink-500 hover:from-fuchsia-500 hover:to-pink-400 text-white font-semibold rounded-lg shadow-md transition-all".to_string()
                    >
                        <span>"Turn on alerts"</span>
                    </HighlightedButton>
                </div>
            </div>
        </ShadowOverlay>
    }
}
