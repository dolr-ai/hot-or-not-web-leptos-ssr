pub mod username;

use component::{
    back_btn::BackButton, icons::edit_icons::EditUnderlinedIcon, spinner::Spinner, title::TitleText,
};
use leptos::{either::Either, prelude::*};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::components::Redirect;
use state::{app_state::AppState, canisters::auth_state};
use utils::web::copy_to_clipboard;
use yral_canisters_common::utils::profile::ProfileDetails;

#[component]
pub fn ProfileEdit() -> impl IntoView {
    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Edit Profile";

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
                        <ProfileEditInner details=cans.profile_details />
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
fn EditField<Icon: IntoView>(
    #[prop(into)] name: String,
    #[prop(into)] value: String,
    icon: impl FnOnce() -> Icon,
) -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-2.5">
            <div class="w-full flex flex-row justify-between items-center">
                <span class="text-neutral-300 font-medium text-sm md:text-base">{name}</span>
                {icon()}
            </div>
            <div class="w-full p-3 bg-neutral-900 border-neutral-800 border rounded-lg">
                <p class="text-sm md:text-base text-neutral-400">{value}</p>
            </div>
        </div>
    }
}

#[component]
fn ProfileEditInner(details: ProfileDetails) -> impl IntoView {
    view! {
        <div class="flex flex-col w-full gap-7.5 items-center">
            <img class="w-35 h-35 rounded-full" src=details.profile_pic_or_random() />
            <div class="flex flex-col gap-5 w-full px-4">
                <EditField
                    name="User Name"
                    value=format!("@{}", details.username_or_fallback())
                    icon=|| view! {
                        <a href="/profile/edit/username">
                            <Icon
                                icon=EditUnderlinedIcon
                                attr:class="text-2xl text-neutral-300 fill-none"
                            />
                        </a>
                    }
                />
                <EditField
                    name="Unique ID"
                    value=details.principal.to_text()
                    icon=move || view! {
                        <button class="cursor-pointer" on:click=move |_| { copy_to_clipboard(&details.principal.to_text()); }>
                            <Icon
                                icon=icondata::IoCopyOutline
                                attr:class="text-2xl invert"
                            />
                        </button>
                    }
                />
            </div>
        </div>
    }
}
