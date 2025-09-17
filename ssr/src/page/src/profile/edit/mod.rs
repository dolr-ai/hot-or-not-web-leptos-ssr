pub mod username;

use component::{
    back_btn::BackButton, spinner::Spinner, title::TitleText,
};
use leptos::{either::Either, prelude::*};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::components::Redirect;
use state::{app_state::AppState, canisters::auth_state};
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
                let cans = auth.auth_cans().await;
                match cans {
                    Ok(cans) => Either::Left(view! {
                        <ProfileEditInner details=cans.profile_details() />
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
fn InputField(
    #[prop(into)] label: String,
    #[prop(into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(optional)] is_required: bool,
    #[prop(optional)] prefix: Option<String>,
    #[prop(optional)] multiline: bool,
) -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-[10px]">
            <div class="flex gap-2 items-center">
                <span class="text-[14px] font-medium text-neutral-400 font-['Kumbh_Sans']">
                    {label}
                    {is_required.then(|| "*")}
                </span>
            </div>
            <div class="bg-[#171717] border border-[#212121] rounded-lg p-3 flex items-center gap-0.5">
                {prefix.map(|p| view! {
                    <span class="text-[14px] font-medium text-neutral-400 font-['Kumbh_Sans']">{p}</span>
                })}
                {if multiline {
                    view! {
                        <textarea
                            class="w-full bg-transparent text-[14px] font-medium text-neutral-50 font-['Kumbh_Sans'] placeholder-neutral-400 outline-none resize-none"
                            placeholder=placeholder
                            rows=2
                            bind:value=value
                        />
                    }.into_any()
                } else {
                    view! {
                        <input
                            type="text"
                            class="w-full bg-transparent text-[14px] font-medium text-neutral-50 font-['Kumbh_Sans'] placeholder-neutral-400 outline-none"
                            placeholder=placeholder
                            bind:value=value
                        />
                    }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn ProfileEditInner(details: ProfileDetails) -> impl IntoView {
    // Form state with dummy values
    let username = RwSignal::new("Creator_mavrick".to_string());
    let bio = RwSignal::new("Dreaming big, building tokens that pump 🚀".to_string());
    let website = RwSignal::new("https://creatormavrick.com".to_string());
    let email = RwSignal::new("malvika@gobazzinga.in".to_string());

    let on_save = move || {
        // Handle save action
        leptos::logging::log!("Save clicked");
    };

    view! {
        <div class="flex flex-col w-full items-center">
            // Profile Picture with Edit Overlay
            <div class="relative mb-[40px]">
                <img
                    class="w-[120px] h-[120px] rounded-full"
                    src=details.profile_pic_or_random()
                />
                <div class="absolute bottom-0 right-0 w-[40px] h-[40px] bg-[#171717] rounded-full border border-[#212121] flex items-center justify-center cursor-pointer">
                    <Icon
                        icon=icondata::BiEditRegular
                        attr:class="text-[20px] text-[#d4d4d4]"
                    />
                </div>
            </div>

            // Form Fields
            <div class="flex flex-col gap-[20px] w-full px-4 max-w-[358px]">
                // Username Field
                <InputField
                    label="User Name"
                    placeholder="Type user name"
                    value=username
                    is_required=true
                    prefix="@".to_string()
                />

                // Bio Field
                <InputField
                    label="Bio"
                    placeholder="Tell us about yourself"
                    value=bio
                    multiline=true
                />

                // Website Field
                <InputField
                    label="Website/URL"
                    placeholder="Your website URL"
                    value=website
                />

                // Email Field
                <InputField
                    label="Email"
                    placeholder="Your email address"
                    value=email
                />

                // Save Button
                <button
                    on:click=move |_| on_save()
                    class="w-full h-[45px] rounded-lg flex items-center justify-center mt-[10px] cursor-pointer transition-all hover:opacity-90"
                    style="background: linear-gradient(90deg, #e2017b 0%, #e2017b 100%)"
                >
                    <span class="text-[16px] font-bold text-[#f6b0d6] font-['Kumbh_Sans']">
                        "Save"
                    </span>
                </button>
            </div>
        </div>
    }
}
