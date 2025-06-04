use super::spinner::Spinner;
use auth::delegate_short_lived_identity;
use state::{
    canisters::{auth_state, unauth_canisters},
    content_seed_client::ContentSeedClient,
};
#[derive(Default, Clone, Copy)]
pub struct AuthorizedUserToSeedContent(pub RwSignal<Option<(bool, Principal)>>);
use candid::Principal;
use leptos::prelude::*;
use yral_canisters_common::Canisters;

#[component]
fn YoutubeUploadInner(#[prop(optional)] url: String) -> impl IntoView {
    let url_value = RwSignal::new(url);
    let create_short_lived_delegated_identity = |canisters: &Canisters<true>| {
        let id = canisters.identity();
        delegate_short_lived_identity(id)
    };

    let auth = auth_state();
    let base = unauth_canisters();
    let on_submit: Action<(), String> = Action::new_unsync(move |_| {
        let base = base.clone();
        async move {
            let cans = match auth.auth_cans(base).await {
                Ok(c) => c,
                Err(e) => return e.to_string(),
            };

            let delegated_identity = create_short_lived_delegated_identity(&cans);
            let content_seed_client: ContentSeedClient = expect_context();
            let res = content_seed_client
                .upload_content(url_value(), delegated_identity)
                .await;
            match res {
                Err(e) => e.to_string(),
                _ => "Submitted!".to_string(),
            }
        }
    });
    let submit_res = on_submit.value();

    view! {
        <div data-hk="1-0-0-3" class="flex h-full items-center justify-around p-4">
            <div data-hk="1-0-0-4" class="flex flex-col items-center justify-center">
                <div class="flex h-full flex-col justify-around gap-6">
                    <div class="flex basis-9/12 flex-col items-center justify-center">
                        <h1 data-hk="1-0-0-5" class="text-2xl md:text-3xl text-white">
                            VIDEO IMPORTER
                        </h1>
                    </div>
                    <div class="flex basis-3/12 flex-col justify-around items-center gap-4">
                        <input
                            type="text"
                            value=move || url_value.get()
                            on:input=move |ev| {
                                let val = event_target_value(&ev);
                                url_value.set(val);
                            }

                            placeholder=" Paste your link here"
                            class="p-1 md:text-xl"
                        />
                        <button
                            type="submit"
                            class="border border-solid px-4 text-xl md:text-2xl w-fit text-white hover:bg-white hover:text-black"
                            on:click=move |_| {on_submit.dispatch(());}
                        >

                            Submit
                        </button>
                        <p class="text-base md:text-lg text-white">
                            {move || submit_res.get().unwrap_or_default()}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn YoutubeUpload(#[prop(optional)] url: String, user_principal: Principal) -> impl IntoView {
    let url_s = StoredValue::new(url);

    let authorized_ctx: AuthorizedUserToSeedContent = expect_context();
    let authorized = authorized_ctx.0;
    let loaded = move || {
        authorized()
            .map(|(_, principal)| principal == user_principal)
            .unwrap_or_default()
    };

    view! {
        <Show when=loaded fallback=Spinner>
            <Show when=move || authorized().map(|(a, _)| a).unwrap_or_default()>
                <YoutubeUploadInner url=url_s.get_value() />
            </Show>
        </Show>
    }
}
