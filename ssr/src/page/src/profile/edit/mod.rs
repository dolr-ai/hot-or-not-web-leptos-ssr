pub mod username;

use component::{back_btn::BackButton, spinner::Spinner, title::TitleText};
use leptos::{either::Either, html, prelude::*, task::spawn_local};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::{components::Redirect, hooks::use_navigate};
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};
use state::{app_state::AppState, canisters::auth_state};
use utils::send_wrap;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yral_canisters_client::local::USER_INFO_SERVICE_ID;
use yral_canisters_client::user_info_service::{ProfileUpdateDetails, Result_ as UserInfoResult};
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
                let identity = auth.user_identity.await;
                match (cans, identity) {
                    (Ok(cans), Ok(identity)) => Either::Left(view! {
                        <ProfileEditInner details=cans.profile_details() identity=identity auth=auth.clone() />
                    }),
                    (Err(e), _) | (_, Err(e)) => Either::Right(view! {
                        <Redirect path=format!("/error?err={e}") />
                    }),
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
    #[prop(optional, into)] input_type: String,
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
                    let field_type = if input_type.is_empty() { "text".to_string() } else { input_type };
                    view! {
                        <input
                            type=field_type
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

fn validate_and_format_url(url: String) -> Result<String, String> {
    if url.is_empty() {
        return Ok(url);
    }

    let formatted = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url
    };

    // Basic validation
    if formatted.contains(' ') || !formatted.contains('.') {
        return Err("Invalid URL format".to_string());
    }

    Ok(formatted)
}

#[component]
fn ProfileEditInner(
    details: ProfileDetails,
    identity: utils::types::NewIdentity,
    auth: state::canisters::AuthState,
) -> impl IntoView {
    // Form state with actual profile data
    let username = RwSignal::new(details.username_or_fallback());
    let bio = RwSignal::new(details.bio.clone().unwrap_or_default());
    let website = RwSignal::new(details.website_url.clone().unwrap_or_default());
    let profile_pic_url = RwSignal::new(details.profile_pic_or_random());
    let show_image_editor = RwSignal::new(false);
    let user_principal = details.principal.to_text();
    let saving = RwSignal::new(false);
    let save_error = RwSignal::new(Option::<String>::None);
    let success_message = RwSignal::new(Option::<String>::None);
    let nav = use_navigate();

    // Setup auto-dismiss for success message
    let UseTimeoutFnReturn { start: start_success_timeout, .. } = use_timeout_fn(
        move |_| {
            success_message.set(None);
        },
        3000.0,  // 3 seconds
    );

    // Create profile update action
    let update_profile = Action::new(move |_: &()| {
        let bio_val = bio.get_untracked();
        let website_val = website.get_untracked();
        let profile_pic_val = profile_pic_url.get_untracked();
        let _nav = nav.clone();
        let timeout_fn = start_success_timeout.clone();

        send_wrap(async move {
            saving.set(true);
            save_error.set(None);
            success_message.set(None);

            // Validate and format URL
            let formatted_website = match validate_and_format_url(website_val.clone()) {
                Ok(url) => url,
                Err(e) => {
                    save_error.set(Some(e));
                    saving.set(false);
                    return;
                }
            };

            let Ok(canisters) = auth.auth_cans().await else {
                save_error.set(Some("Not authenticated".to_string()));
                saving.set(false);
                return;
            };

            // Only support service canister users
            if canisters.user_canister() != USER_INFO_SERVICE_ID {
                log::error!("Profile update not supported for individual canister users");
                save_error.set(Some("Profile update not available for this account type".to_string()));
                saving.set(false);
                return;
            }

            let service = canisters.user_info_service().await;
            let update_details = ProfileUpdateDetails {
                bio: if bio_val.is_empty() { None } else { Some(bio_val.clone()) },
                website_url: if formatted_website.is_empty() { None } else { Some(formatted_website.clone()) },
                profile_picture_url: Some(profile_pic_val),
            };

            match service.update_profile_details(update_details).await {
                Ok(UserInfoResult::Ok) => {
                    log::info!("Profile updated successfully");

                    // Update the form values with the saved values
                    bio.set(bio_val);
                    website.set(formatted_website);

                    // Show success message
                    success_message.set(Some("Profile updated successfully!".to_string()));
                    timeout_fn(());
                }
                Ok(UserInfoResult::Err(e)) => {
                    log::warn!("Error updating profile: {e}");
                    save_error.set(Some(format!("Update failed: {e}")));
                }
                Err(e) => {
                    log::warn!("Network error updating profile: {e:?}");
                    save_error.set(Some("Network error. Please try again.".to_string()));
                }
            }

            saving.set(false);
        })
    });

    view! {
        <div class="flex flex-col w-full items-center">
            // Image Editor Popup
            <Show when=move || show_image_editor.get()>
                <ProfileImageEditor
                    show=show_image_editor
                    profile_pic_url
                    _user_principal=user_principal.clone()
                    identity=identity.clone()
                />
            </Show>

            // Profile Picture with Edit Overlay
            <div class="relative mb-[40px]">
                <img
                    class="w-[120px] h-[120px] rounded-full"
                    src=move || profile_pic_url.get()
                />
                <div
                    class="absolute bottom-0 right-0 w-[40px] h-[40px] bg-[#171717] rounded-full border border-[#212121] flex items-center justify-center cursor-pointer hover:bg-[#212121] transition-colors"
                    on:click=move |_| show_image_editor.set(true)
                >
                    <Icon
                        icon=icondata::BiEditRegular
                        attr:class="text-[20px] text-[#d4d4d4]"
                    />
                </div>
            </div>

            // Form Fields
            <div class="flex flex-col gap-[20px] w-full px-4 max-w-[358px]">
                // Username Field (Read-only for now)
                <div class="w-full flex flex-col gap-[10px]">
                    <div class="flex gap-2 items-center">
                        <span class="text-[14px] font-medium text-neutral-400 font-['Kumbh_Sans']">
                            "User Name"
                        </span>
                    </div>
                    <div class="bg-[#171717] border border-[#212121] rounded-lg p-3 flex items-center gap-0.5 opacity-60">
                        <span class="text-[14px] font-medium text-neutral-400 font-['Kumbh_Sans']">@</span>
                        <input
                            type="text"
                            class="w-full bg-transparent text-[14px] font-medium text-neutral-50 font-['Kumbh_Sans'] placeholder-neutral-400 outline-none cursor-not-allowed"
                            value=move || username.get()
                            disabled=true
                        />
                    </div>
                    <span class="text-[12px] text-neutral-500 font-['Kumbh_Sans']">
                        "Username cannot be changed at this time"
                    </span>
                </div>

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
                    input_type="url"
                />


                // Error Message
                <Show when=move || save_error.get().is_some()>
                    <div class="w-full p-3 bg-red-900/20 border border-red-500 rounded-lg">
                        <span class="text-[14px] text-red-400">
                            {move || save_error.get().unwrap_or_default()}
                        </span>
                    </div>
                </Show>

                // Success Message
                <Show when=move || success_message.get().is_some()>
                    <div class="w-full p-3 bg-green-900/20 border border-green-500 rounded-lg transition-opacity duration-500">
                        <span class="text-[14px] text-green-400">
                            {move || success_message.get().unwrap_or_default()}
                        </span>
                    </div>
                </Show>

                // Save Button
                <button
                    on:click=move |_| {
                        update_profile.dispatch(());
                    }
                    disabled=move || saving.get()
                    class="w-full h-[45px] rounded-lg flex items-center justify-center mt-[10px] cursor-pointer transition-all"
                    class=("opacity-50", move || saving.get())
                    class=("cursor-not-allowed", move || saving.get())
                    class=("hover:opacity-90", move || !saving.get())
                    style="background: linear-gradient(90deg, #e2017b 0%, #e2017b 100%)"
                >
                    <Show
                        when=move || !saving.get()
                        fallback=|| view! {
                            <span class="text-[16px] font-bold text-[#f6b0d6] font-['Kumbh_Sans']">
                                "Saving..."
                            </span>
                        }
                    >
                        <span class="text-[16px] font-bold text-[#f6b0d6] font-['Kumbh_Sans']">
                            "Save"
                        </span>
                    </Show>
                </button>
            </div>
        </div>
    }
}

// Upload profile image via off-chain agent API
async fn upload_profile_image_to_agent(
    image_data: String,
    delegated_identity_wire: yral_types::delegated_identity::DelegatedIdentityWire,
) -> Result<String, String> {
    use consts::OFF_CHAIN_AGENT_URL;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct UploadRequest {
        delegated_identity_wire: yral_types::delegated_identity::DelegatedIdentityWire,
        image_data: String,
    }

    #[derive(Deserialize)]
    struct UploadResponse {
        profile_image_url: String,
    }

    let url = OFF_CHAIN_AGENT_URL
        .join("api/v1/user/profile-image")
        .map_err(|e| format!("Failed to construct URL: {}", e))?;

    let request_body = UploadRequest {
        delegated_identity_wire,
        image_data,
    };

    #[cfg(feature = "ssr")]
    {
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Upload failed: {}", error_text));
        }

        let upload_response: UploadResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(upload_response.profile_image_url)
    }

    #[cfg(not(feature = "ssr"))]
    {
        // In hydrate mode, use gloo to make the request
        use gloo::net::http::Request;

        let response = Request::post(url.as_str())
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Upload failed: {}", error_text));
        }

        let upload_response: UploadResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(upload_response.profile_image_url)
    }
}

#[component]
fn ProfileImageEditor(
    show: RwSignal<bool>,
    profile_pic_url: RwSignal<String>,
    _user_principal: String,
    identity: utils::types::NewIdentity,
) -> impl IntoView {
    let uploaded_image = RwSignal::new(Option::<String>::None);
    let zoom_level = RwSignal::new(1.0_f64);
    let position_x = RwSignal::new(0.0_f64);
    let position_y = RwSignal::new(0.0_f64);
    let is_dragging = RwSignal::new(false);
    let drag_start_x = RwSignal::new(0.0_f64);
    let drag_start_y = RwSignal::new(0.0_f64);
    let file_input_ref = NodeRef::<html::Input>::new();
    let is_uploading = RwSignal::new(false);

    let handle_file_change = move |ev: leptos::web_sys::Event| {
        let input: leptos::web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let file_reader = leptos::web_sys::FileReader::new().unwrap();
                let file_reader_clone = file_reader.clone();

                let closure = Closure::wrap(Box::new(move |_event: leptos::web_sys::Event| {
                    if let Ok(result) = file_reader_clone.result() {
                        if let Ok(data_url) = result.dyn_into::<js_sys::JsString>() {
                            uploaded_image.set(Some(data_url.as_string().unwrap()));
                            // Reset zoom and position when new image is loaded
                            zoom_level.set(1.0);
                            position_x.set(0.0);
                            position_y.set(0.0);
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));
                file_reader.read_as_data_url(&file).unwrap();
                closure.forget();
            }
        }
    };

    let handle_zoom_in = move |_| {
        zoom_level.update(|z| *z = (*z * 1.1).min(3.0));
    };

    let handle_zoom_out = move |_| {
        zoom_level.update(|z| *z = (*z / 1.1).max(0.5));
    };

    let handle_mouse_down = move |ev: leptos::web_sys::MouseEvent| {
        is_dragging.set(true);
        drag_start_x.set(ev.client_x() as f64 - position_x.get());
        drag_start_y.set(ev.client_y() as f64 - position_y.get());
        ev.prevent_default();
    };

    let handle_mouse_move = move |ev: leptos::web_sys::MouseEvent| {
        if is_dragging.get() {
            position_x.set(ev.client_x() as f64 - drag_start_x.get());
            position_y.set(ev.client_y() as f64 - drag_start_y.get());
            ev.prevent_default();
        }
    };

    let handle_mouse_up = move |_| {
        is_dragging.set(false);
    };

    // Touch event handlers for mobile
    let handle_touch_start = move |ev: leptos::web_sys::TouchEvent| {
        if let Some(touch) = ev.touches().get(0) {
            is_dragging.set(true);
            drag_start_x.set(touch.client_x() as f64 - position_x.get());
            drag_start_y.set(touch.client_y() as f64 - position_y.get());
            ev.prevent_default();
        }
    };

    let handle_touch_move = move |ev: leptos::web_sys::TouchEvent| {
        if is_dragging.get() {
            if let Some(touch) = ev.touches().get(0) {
                position_x.set(touch.client_x() as f64 - drag_start_x.get());
                position_y.set(touch.client_y() as f64 - drag_start_y.get());
                ev.prevent_default();
            }
        }
    };

    let handle_touch_end = move |_| {
        is_dragging.set(false);
    };

    view! {
        <component::overlay::ShadowOverlay show>
            <div class="flex flex-col justify-around items-center py-4 w-[90vw] md:w-[80vw] lg:w-[70vw] max-w-5xl rounded-md cursor-auto px-6 bg-neutral-900">
                <div class="flex justify-between items-center w-full mb-4">
                    <h2 class="text-xl font-bold text-white">"Edit Profile Picture"</h2>
                    <button
                        on:click=move |_| show.set(false)
                        class="p-1 text-lg text-center text-white rounded-full md:text-xl bg-neutral-600"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                </div>
                <div class="flex flex-col gap-4 w-full">
                    // File input
                    <input
                        type="file"
                        accept="image/*"
                        node_ref=file_input_ref
                        on:change=handle_file_change
                        class="hidden"
                    />

                    // Image editor area
                    <div class="relative w-full bg-neutral-800 rounded-lg overflow-hidden h-[400px] md:h-[500px]">
                    {move || {
                        if let Some(image_url) = uploaded_image.get() {
                            view! {
                                <div
                                    class="relative w-full h-full flex items-center justify-center overflow-hidden cursor-move touch-none"
                                    on:mousedown=handle_mouse_down
                                    on:mousemove=handle_mouse_move
                                    on:mouseup=handle_mouse_up
                                    on:mouseleave=handle_mouse_up
                                    on:touchstart=handle_touch_start
                                    on:touchmove=handle_touch_move
                                    on:touchend=handle_touch_end
                                >
                                    // Circular mask overlay
                                    <div class="absolute inset-0 pointer-events-none" style="background: radial-gradient(circle at center, transparent 150px, rgba(0,0,0,0.7) 150px);" />

                                    // Image with transformations
                                    <img
                                        src=image_url
                                        class="absolute max-w-none select-none"
                                        style=move || format!(
                                            "transform: translate({}px, {}px) scale({});",
                                            position_x.get(),
                                            position_y.get(),
                                            zoom_level.get()
                                        )
                                        draggable="false"
                                    />

                                    // Circular guide
                                    <div class="absolute w-[300px] h-[300px] md:w-[350px] md:h-[350px] border-2 border-white/30 rounded-full pointer-events-none" />
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div
                                    class="w-full h-full flex flex-col items-center justify-center cursor-pointer hover:bg-neutral-700/30 transition-colors border-2 border-dashed border-neutral-600 rounded-lg"
                                    on:click=move |_| {
                                        if let Some(input) = file_input_ref.get() {
                                            input.click();
                                        }
                                    }
                                >
                                    <Icon icon=icondata::BiImageAddRegular attr:class="text-6xl md:text-8xl text-neutral-400 mb-4" />
                                    <p class="text-base md:text-lg text-neutral-400 font-medium">"Click to upload an image"</p>
                                    <p class="text-xs md:text-sm text-neutral-500 mt-2">"or drag and drop"</p>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Zoom controls
                {move || {
                    if uploaded_image.get().is_some() {
                        view! {
                            <div class="flex items-center gap-4">
                                <button
                                    on:click=handle_zoom_out
                                    class="p-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg text-white"
                                >
                                    <Icon icon=icondata::AiMinusOutlined attr:class="text-xl" />
                                </button>

                                <input
                                    type="range"
                                    min="0.5"
                                    max="3"
                                    step="0.1"
                                    prop:value=move || zoom_level.get().to_string()
                                    on:input=move |ev| {
                                        let target: leptos::web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                        if let Ok(val) = target.value().parse::<f64>() {
                                            zoom_level.set(val);
                                        }
                                    }
                                    class="flex-1 h-2 bg-neutral-700 rounded-lg appearance-none cursor-pointer"
                                />

                                <button
                                    on:click=handle_zoom_in
                                    class="p-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg text-white"
                                >
                                    <Icon icon=icondata::AiPlusOutlined attr:class="text-xl" />
                                </button>

                                <span class="text-white text-sm min-w-[50px]">
                                    {move || format!("{}%", (zoom_level.get() * 100.0) as i32)}
                                </span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div /> }.into_any()
                    }
                }}

                // Action buttons
                <div class="flex gap-3 mt-4">
                    {move || {
                        if uploaded_image.get().is_some() {
                            view! {
                                <button
                                    on:click=move |_| {
                                        if let Some(input) = file_input_ref.get() {
                                            input.click();
                                        }
                                    }
                                    class="flex-1 px-4 py-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg text-white font-medium"
                                >
                                    "Change Image"
                                </button>
                            }.into_any()
                        } else {
                            view! { <div /> }.into_any()
                        }
                    }}

                    <button
                        on:click=move |_| {
                            show.set(false);
                            uploaded_image.set(None);
                        }
                        class="flex-1 px-4 py-2 bg-neutral-700 hover:bg-neutral-600 rounded-lg text-white font-medium"
                    >
                        "Cancel"
                    </button>

                    <button
                        on:click={
                            let identity = identity.clone();
                            move |_| {
                                if let Some(image_data) = uploaded_image.get() {
                                    is_uploading.set(true);
                                    let delegated_identity = identity.id_wire.clone();

                                spawn_local(async move {
                                    match upload_profile_image_to_agent(image_data, delegated_identity).await {
                                        Ok(new_url) => {
                                            profile_pic_url.set(new_url);
                                            show.set(false);
                                            uploaded_image.set(None);
                                            is_uploading.set(false);
                                            leptos::logging::log!("Profile picture updated");
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("Failed to upload image: {}", e);
                                            is_uploading.set(false);
                                        }
                                    }
                                });
                            }
                        }
                        }
                        disabled=move || uploaded_image.get().is_none() || is_uploading.get()
                        class="flex-1 px-4 py-2 bg-gradient-to-r from-[#e2017b] to-[#e2017b] hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-white font-medium"
                    >
                        {move || if is_uploading.get() { "Saving..." } else { "Save" }}
                    </button>
                </div>
                </div>
            </div>
        </component::overlay::ShadowOverlay>
    }
}
