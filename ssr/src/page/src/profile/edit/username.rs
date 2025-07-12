use component::{
    back_btn::BackButton, buttons::GradientButton, spinner::SpinnerCircle, title::TitleText,
};
use leptos::{either::Either, html, prelude::*};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::{components::Redirect, hooks::use_navigate, NavigateOptions};
use state::{
    app_state::AppState,
    canisters::{auth_state, unauth_canisters, AuthState},
};
use utils::send_wrap;
use yral_canisters_common::Canisters;

#[component]
pub fn ProfileUsernameEdit() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Change Username";

    view! {
        <Title text=page_title.clone() />

        <div class="flex flex-col items-center pt-2 pb-12 bg-black min-w-dvw min-h-dvh">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/profile/edit".to_string() />
                    <span class="text-lg font-bold text-white">Change Username</span>
                    <div></div>
                </div>
            </TitleText>
            <UsernameEditInner />
        </div>
    }
}

async fn set_username(
    auth: AuthState,
    base: Canisters<false>,
    username: String,
) -> Result<(), String> {
    let cans = auth.auth_cans(base).await.map_err(|e| {
        eprintln!("did not expect to get error: {e}");
        String::from("Unknown Error")
    })?;
    let res = auth.update_username(cans, username).await;

    match res {
        Ok(_) => Ok(()),
        Err(yral_canisters_common::Error::Metadata(yral_metadata_client::Error::Api(
            yral_metadata_types::error::ApiError::DuplicateUsername,
        ))) => Err("This username is not available".into()),
        Err(e) => {
            eprintln!("Error updating username: {e}");
            Err("Unknown Error".into())
        }
    }
}

#[component]
fn UsernameEditInner() -> impl IntoView {
    let new_username = RwSignal::new(String::new());
    let input_ref = NodeRef::<html::Input>::new();

    let trigger_validity_change = Trigger::new();

    let error_message = move || {
        trigger_validity_change.track();
        let input = input_ref.get()?;
        if input.check_validity() {
            return None;
        }

        #[cfg(feature = "hydrate")]
        if input.validity().pattern_mismatch() {
            return Some(
                "Username must be 3-15 characters long and can only contain letters and numbers."
                    .to_string(),
            );
        }

        Some(
            input
                .validation_message()
                .unwrap_or_else(|_| "Invalid input".to_string()),
        )
    };

    let on_input = move || {
        let Some(input) = input_ref.get() else {
            return;
        };
        input.set_custom_validity("");
        trigger_validity_change.notify();
    };

    let auth = auth_state();
    let cans = unauth_canisters();

    let nav = use_navigate();
    let set_username_action = Action::new(move |()| {
        let username = new_username.get_untracked();
        let cans = cans.clone();
        let nav = nav.clone();
        async move {
            let res = send_wrap(set_username(auth, cans, username)).await;
            if let Err(e) = res {
                let Some(input) = input_ref.get_untracked() else {
                    return Ok(());
                };
                input.set_custom_validity(&e);
                trigger_validity_change.notify();
                return Err(());
            }

            let nav_options = NavigateOptions {
                replace: true,
                ..Default::default()
            };
            nav("/profile/edit", nav_options);
            Ok(())
        }
    });
    let changing_username = set_username_action.pending();
    let username_change_res = set_username_action.value();

    let save_disabled = Signal::derive(move || {
        trigger_validity_change.track();
        let Some(input) = input_ref.get() else {
            return true;
        };
        !input.check_validity() || new_username.with(|u| u.is_empty()) || changing_username.get()
    });

    view! {
        <div class="w-full flex flex-col gap-7.5 items-center">
            <div class="w-35 h-35">
                <Transition fallback=|| view! {
                    <div class="w-full h-full rounded-full animate-pulse overflow-clip bg-white/20"/>
                }>
                {move || Suspend::new(async move {
                    let cans = auth.cans_wire().await;
                    match cans {
                        Ok(cans) => Either::Left(view! {
                            <img class="w-35 h-35 rounded-full" src=cans.profile_details.profile_pic_or_random() />
                        }),
                        Err(e) => Either::Right(view! {
                            <Redirect path=format!("/error?err={e}") />
                        })
                    }
                })}
                </Transition>
            </div>
            <div class="w-full flex flex-col gap-5 p-4">
                <div class="w-full flex flex-col gap-2.5 text-sm md:text-base group">
                    <span class="font-medium text-neutral-300">User Name</span>
                    <form
                        prop:novalidate
                        class="w-full flex flex-row justify-between items-center rounded-lg bg-neutral-950 border border-neutral-800 p-3 group has-[input:valid]:has-[input:focus]:border-primary-500 has-[input:invalid]:border-red-500"
                    >
                        <div class="w-full flex flex-row justify-start items-center gap-0.5">
                            <span class="text-neutral-400">@</span>
                            <input
                                on:input=move |_| on_input()
                                node_ref=input_ref
                                pattern="^([a-zA-Z0-9]){3,15}$"
                                bind:value=new_username
                                class="text-white placeholder-neutral-600 w-full focus:outline-none peer"
                                type="text"
                                placeholder="Type user name"
                            />
                        </div>
                        <Show when=changing_username>
                            <div class="w-5 h-5">
                                <SpinnerCircle />
                            </div>
                        </Show>
                        <Show when=move || !changing_username.get() && username_change_res.with(|res| matches!(res, Some(Ok(_))))>
                            <Icon attr:class="w-5 h-5 text-xl text-green-500" icon=icondata::AiCheckCircleOutlined />
                        </Show>
                        <Show when=move || !changing_username.get() && username_change_res.with(|res| matches!(res, None | Some(Err(_))))>
                            <button type="reset" class="hidden cursor-pointer disabled:cursor-auto group-focus-within:group-has-[input:not(:placeholder-shown)]:block">
                                <Icon attr:class="w-5 h-5 text-neutral-300" icon=icondata::ChCross />
                            </button>
                        </Show>

                    </form>
                    <p class="hidden text-xs text-red-500 group-has-invalid:block">{move || error_message()}</p>
                </div>
                <p class="text-xs text-neutral-500">
                    {"Try creating a unique username. Use 3–15 letters or numbers. No spaces or special characters allowed."}
                </p>
                <GradientButton
                    classes="w-full py-3".to_string()
                    disabled=save_disabled
                    on_click=move || {
                        set_username_action.dispatch(());
                    }
                >
                    Save
                </GradientButton>
            </div>
        </div>
    }
}
