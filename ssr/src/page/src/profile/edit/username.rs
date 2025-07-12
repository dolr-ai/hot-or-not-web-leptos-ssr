use component::{
    back_btn::BackButton, buttons::GradientButton, spinner::Spinner, title::TitleText,
};
use leptos::{either::Either, html, prelude::*};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::components::Redirect;
use state::{app_state::AppState, canisters::auth_state};
use yral_canisters_common::utils::profile::ProfileDetails;

#[component]
pub fn ProfileUsernameEdit() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Change Username";

    let auth = auth_state();

    view! {
        <Title text=page_title.clone() />

        <div class="flex flex-col items-center pt-2 pb-12 bg-black min-w-dvw min-h-dvh">
            <TitleText justify_center=false>
                <div class="flex flex-row justify-between">
                    <BackButton fallback="/profile/posts".to_string() />
                    <span class="text-lg font-bold text-white">Edit Profile</span>
                    <div></div>
                </div>
            </TitleText>
            <Suspense fallback=|| view! {
                <div class="flex items-center justify-center w-full h-full">
                    <Spinner/>
                </div>
            }>
            {move || Suspend::new(async move {
                let cans = auth.cans_wire().await;
                match cans {
                    Ok(cans) => Either::Left(view! {
                        <UsernameEditInner details=cans.profile_details />
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

#[component]
fn UsernameEditInner(details: ProfileDetails) -> impl IntoView {
    let new_username = RwSignal::new(String::new());
    let input_ref = NodeRef::<html::Input>::new();

    let trigger_validity_change = Trigger::new();

    let save_disabled = Signal::derive(move || {
        trigger_validity_change.track();
        let Some(input) = input_ref.get() else {
            return true;
        };
        !input.check_validity() || new_username.with(|u| u.is_empty())
    });

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

    view! {
        <div class="w-full flex flex-col gap-7.5 items-center">
            <img class="w-35 h-35 rounded-full" src=details.profile_pic_or_random() />
            <div class="w-full flex flex-col gap-5 p-4">
                <div class="w-full flex flex-col gap-2.5 text-sm md:text-base group">
                    <span class="font-medium text-neutral-300">User Name</span>
                    <form
                        class="w-full flex flex-row justify-between items-center rounded-lg bg-neutral-950 border border-neutral-800 p-3 group has-[input:valid]:has-[input:focus]:border-primary-500 has-[input:invalid]:border-red-500"
                    >
                        <div class="w-full flex flex-row justify-start items-center gap-0.5">
                            <span class="text-neutral-400">@</span>
                            <input
                                on:input=move |_| trigger_validity_change.notify()
                                node_ref=input_ref pattern="^([a-zA-Z0-9]){3,15}$"
                                bind:value=new_username
                                class="text-white placeholder-neutral-600 w-full focus:outline-none peer"
                                type="text"
                                placeholder="Type user name"
                            />
                        </div>
                        <button type="reset" class="hidden cursor-pointer disabled:cursor-auto group-focus-within:group-has-[input:not(:placeholder-shown)]:block">
                            <Icon attr:class="w-5 h-5 text-neutral-300" icon=icondata::ChCross />
                        </button>
                    </form>
                    <p class="hidden text-xs text-red-500 group-has-invalid:block">{move || error_message()}</p>
                </div>
                <p class="text-xs text-neutral-500">
                    {"Try creating a unique username. Use 3–15 letters or numbers. No spaces or special characters allowed."}
                </p>
                <GradientButton
                    classes="w-full py-3".to_string()
                    disabled=save_disabled
                    on_click=move || { }
                >
                    Save
                </GradientButton>
            </div>
        </div>
    }
}
