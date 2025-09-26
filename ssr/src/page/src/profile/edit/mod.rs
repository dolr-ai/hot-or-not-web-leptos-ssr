pub mod username;

use component::{back_btn::BackButton, spinner::Spinner, title::TitleText};
use leptos::{either::Either, html, prelude::*, task::spawn_local};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::{components::Redirect, hooks::use_navigate};
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
                        <ProfileEditInner details=cans.profile_details() identity=identity auth=auth />
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
                    {is_required.then_some("*")}
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
        format!("https://{url}")
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
    let original_username = details.username_or_fallback();
    let username = RwSignal::new(original_username.clone());
    let bio = RwSignal::new(details.bio.clone().unwrap_or_default());
    let website = RwSignal::new(details.website_url.clone().unwrap_or_default());
    let profile_pic_url = RwSignal::new(details.profile_pic_or_random());
    let show_image_editor = RwSignal::new(false);
    let user_principal = details.principal.to_text();
    let saving = RwSignal::new(false);
    let save_error = RwSignal::new(Option::<String>::None);
    let success_message = RwSignal::new(Option::<String>::None);
    let nav = use_navigate();

    // Username validation state
    let username_input_ref = NodeRef::<html::Input>::new();
    let username_validity_trigger = Trigger::new();
    let username_changing = RwSignal::new(false);
    let username_changed_successfully = RwSignal::new(false);

    // Username validation helpers
    let username_error_message = move || {
        username_validity_trigger.track();
        let input = username_input_ref.get()?;
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

    let on_username_input = move || {
        let Some(input) = username_input_ref.get() else {
            return;
        };
        input.set_custom_validity("");
        username_validity_trigger.notify();
        username_changed_successfully.set(false);
    };

    // Effect to auto-dismiss success message after 3 seconds
    Effect::new(move |_| {
        if success_message.get().is_some() {
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::prelude::*;
                let window = web_sys::window().unwrap();
                let closure = Closure::once(move || {
                    success_message.set(None);
                });
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    3000,
                );
                closure.forget();
            }
        }
    });

    // Create profile update action
    let original_username_clone = original_username.clone();
    let original_profile_pic = details.profile_pic_or_random();
    let original_bio = details.bio.clone().unwrap_or_default();
    let original_website = details.website_url.clone().unwrap_or_default();
    let update_profile = Action::new(move |_: &()| {
        let username_val = username.get_untracked();
        let bio_val = bio.get_untracked();
        let website_val = website.get_untracked();
        let profile_pic_val = profile_pic_url.get_untracked();
        let _nav = nav.clone();
        let orig_username = original_username_clone.clone();
        let orig_profile_pic = original_profile_pic.clone();
        let orig_bio = original_bio.clone();
        let orig_website = original_website.clone();

        send_wrap(async move {
            saving.set(true);
            save_error.set(None);
            success_message.set(None);
            username_changing.set(true);

            // Validate and format URL
            let formatted_website = match validate_and_format_url(website_val.clone()) {
                Ok(url) => url,
                Err(e) => {
                    save_error.set(Some(e));
                    saving.set(false);
                    return;
                }
            };

            // Optimistically update UI immediately
            username.set(username_val.clone());
            bio.set(bio_val.clone());
            website.set(formatted_website.clone());

            let Ok(mut canisters) = auth.auth_cans().await else {
                save_error.set(Some("Not authenticated".to_string()));
                saving.set(false);
                username_changing.set(false);
                return;
            };

            // Only support service canister users
            if canisters.user_canister() != USER_INFO_SERVICE_ID {
                log::error!("Profile update not supported for individual canister users");
                save_error.set(Some(
                    "Profile update not available for this account type".to_string(),
                ));
                saving.set(false);
                username_changing.set(false);
                return;
            }

            // Track what was updated
            let mut username_update_success = false;
            let mut profile_update_success = false;

            // First, check if we need to validate username (but don't update yet)
            let username_changed = username_val != orig_username && !username_val.is_empty();
            if username_changed {
                // Validate username format (3-15 characters, alphanumeric only)
                let is_valid = username_val.len() >= 3
                    && username_val.len() <= 15
                    && username_val.chars().all(|c| c.is_ascii_alphanumeric());
                if !is_valid {
                    save_error.set(Some(
                        "Username must be 3-15 characters and contain only letters and numbers"
                            .to_string(),
                    ));
                    saving.set(false);
                    username_changing.set(false);
                    return;
                }
            }

            // Update profile first (before username to avoid reload with old data)
            let service = canisters.user_info_service().await;
            let update_details = ProfileUpdateDetails {
                bio: if bio_val.is_empty() {
                    None
                } else {
                    Some(bio_val.clone())
                },
                website_url: if formatted_website.is_empty() {
                    None
                } else {
                    Some(formatted_website.clone())
                },
                profile_picture_url: Some(profile_pic_val.clone()),
            };

            // Check if profile needs updating
            let profile_changed = bio_val != orig_bio
                || formatted_website != orig_website
                || profile_pic_val != orig_profile_pic;

            if profile_changed {
                match service.update_profile_details(update_details).await {
                    Ok(UserInfoResult::Ok) => {
                        log::info!("Profile updated successfully");
                        profile_update_success = true;

                        // Update cached profile details
                        canisters.update_profile_details(
                            if bio_val.is_empty() {
                                None
                            } else {
                                Some(bio_val.clone())
                            },
                            if formatted_website.is_empty() {
                                None
                            } else {
                                Some(formatted_website.clone())
                            },
                            Some(profile_pic_val.clone()),
                        );
                        // Trigger reactive updates for profile changes
                        auth.update_canisters(canisters.clone());
                    }
                    Ok(UserInfoResult::Err(e)) => {
                        // Revert all values on profile update error
                        username.set(orig_username.clone());
                        bio.set(orig_bio.clone());
                        website.set(orig_website.clone());

                        log::warn!("Error updating profile: {e}");
                        save_error.set(Some(format!("Update failed: {e}")));
                        saving.set(false);
                        username_changing.set(false);
                        return;
                    }
                    Err(e) => {
                        // Revert all values on network error
                        username.set(orig_username.clone());
                        bio.set(orig_bio.clone());
                        website.set(orig_website.clone());

                        log::warn!("Network error updating profile: {e:?}");
                        save_error.set(Some("Network error. Please try again.".to_string()));
                        saving.set(false);
                        username_changing.set(false);
                        return;
                    }
                }
            }

            // Now update username if it changed (without triggering reload)
            if username_changed {
                match auth
                    .update_username(canisters.clone(), username_val.clone())
                    .await
                {
                    Ok(_) => {
                        log::info!("Username updated successfully");
                        username_update_success = true;
                        // Username is already updated in profile_details and reactive state is triggered
                    }
                    Err(yral_canisters_common::Error::Metadata(
                        yral_metadata_client::Error::Api(
                            yral_metadata_types::error::ApiError::DuplicateUsername,
                        ),
                    )) => {
                        // Revert username on duplicate error
                        username.set(orig_username.clone());

                        // Set custom validity on the input element
                        if let Some(input) = username_input_ref.get_untracked() {
                            input.set_custom_validity("This username is not available");
                            username_validity_trigger.notify();
                        }
                        save_error.set(Some("This username is not available".to_string()));
                        saving.set(false);
                        username_changing.set(false);
                        return;
                    }
                    Err(e) => {
                        // Revert username on error
                        username.set(orig_username.clone());

                        log::error!("Error updating username: {e}");
                        save_error.set(Some(format!("Failed to update username: {e}")));
                        saving.set(false);
                        username_changing.set(false);
                        return;
                    }
                }
            }

            // Show success message if anything was updated
            if username_update_success || profile_update_success {
                // Generate success message
                let mut updated_items = Vec::new();
                if username_update_success {
                    updated_items.push("Username");
                }
                if profile_update_success {
                    updated_items.push("Profile");
                }

                let success_msg = format!("{} updated successfully!", updated_items.join(" and "));
                success_message.set(Some(success_msg));
            }

            saving.set(false);
            username_changing.set(false);
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
                    class="w-[120px] h-[120px] rounded-full object-cover object-center"
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
                // Username Field
                <div class="w-full flex flex-col gap-[10px] group">
                    <div class="flex gap-2 items-center">
                        <span class="text-[14px] font-medium text-neutral-400 font-['Kumbh_Sans']">
                            "User Name"
                        </span>
                    </div>
                    <form
                        prop:novalidate
                        class="bg-[#171717] border border-[#212121] rounded-lg p-3 flex items-center gap-0.5 has-[input:valid]:has-[input:focus]:border-green-500/50 has-[input:invalid]:has-[input:not(:placeholder-shown)]:border-red-500"
                    >
                        <span class="text-[14px] font-medium text-neutral-400 font-['Kumbh_Sans']">@</span>
                        <input
                            type="text"
                            pattern="^([a-zA-Z0-9]){3,15}$"
                            node_ref=username_input_ref
                            on:input=move |_| on_username_input()
                            bind:value=username
                            class="w-full bg-transparent text-[14px] font-medium text-neutral-50 font-['Kumbh_Sans'] placeholder-neutral-400 outline-none peer"
                            placeholder="Enter username"
                        />
                        // Visual feedback indicators
                        <Show when=username_changing>
                            <div class="w-4 h-4 animate-spin rounded-full border-2 border-neutral-500 border-t-white" />
                        </Show>
                        <Show when=move || !username_changing.get() && username_changed_successfully.get()>
                            <Icon
                                attr:class="w-4 h-4 text-green-500"
                                icon=icondata::AiCheckCircleOutlined
                            />
                        </Show>
                        <Show when={
                            let orig = original_username.clone();
                            move || !username_changing.get() && !username_changed_successfully.get() && username.with(|u| !u.is_empty() && *u != orig)
                        }>
                            <button
                                type="button"
                                on:click={
                                    let orig = original_username.clone();
                                    move |_| {
                                        username.set(orig.clone());
                                        if let Some(input) = username_input_ref.get() {
                                            input.set_custom_validity("");
                                            username_validity_trigger.notify();
                                        }
                                        username_changed_successfully.set(false);
                                    }
                                }
                                class="cursor-pointer hover:opacity-80"
                            >
                                <Icon
                                    attr:class="w-4 h-4 text-neutral-400"
                                    icon=icondata::ChCross
                                />
                            </button>
                        </Show>
                    </form>
                    <p class="hidden text-xs text-red-500 group-has-[input:invalid]:group-has-[input:not(:placeholder-shown)]:block">
                        {move || username_error_message()}
                    </p>
                    <span class="text-[12px] text-neutral-500 font-['Kumbh_Sans']">
                        "Username must be 3-15 characters. Letters and numbers only."
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

// Crop and process the image using Canvas API
#[cfg(feature = "hydrate")]
async fn crop_and_process_image(
    image_data_url: String,
    zoom: f64,
    offset_x: f64,
    offset_y: f64,
) -> Result<String, String> {
    use leptos::web_sys::{HtmlCanvasElement, HtmlImageElement};
    use wasm_bindgen::{JsCast, JsValue};

    let document = leptos::web_sys::window()
        .ok_or("No window")?
        .document()
        .ok_or("No document")?;

    // Create a new image element
    let img = document
        .create_element("img")
        .map_err(|_| "Failed to create img")?
        .dyn_into::<HtmlImageElement>()
        .map_err(|_| "Failed to cast to img")?;

    // Create promise for image load
    let (tx, rx) = futures::channel::oneshot::channel();
    let tx = std::rc::Rc::new(std::cell::RefCell::new(Some(tx)));

    let onload_tx = tx.clone();
    let onload = Closure::once(move || {
        if let Some(tx) = onload_tx.borrow_mut().take() {
            let _ = tx.send(Ok(()));
        }
    });

    let onerror_tx = tx.clone();
    let onerror = Closure::once(move || {
        if let Some(tx) = onerror_tx.borrow_mut().take() {
            let _ = tx.send(Err("Failed to load image"));
        }
    });

    img.set_onload(Some(onload.as_ref().unchecked_ref()));
    img.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    img.set_src(&image_data_url);

    // Wait for image to load
    rx.await.map_err(|_| "Image load cancelled")??;

    onload.forget();
    onerror.forget();

    // Create canvas for cropping
    let canvas = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "Failed to cast to canvas")?;

    // Set canvas size to desired output size (square)
    const OUTPUT_SIZE: u32 = 500;
    canvas.set_width(OUTPUT_SIZE);
    canvas.set_height(OUTPUT_SIZE);

    let context = canvas
        .get_context("2d")
        .map_err(|_| "Failed to get context")?
        .ok_or("No context")?;

    // Calculate the source rectangle based on zoom and position
    let img_width = img.natural_width() as f64;
    let img_height = img.natural_height() as f64;

    // The visible area size in the preview (300px circle)
    let preview_size = 300.0;

    // Calculate the actual crop size on the source image
    let crop_size = preview_size / zoom;

    // Calculate the center position accounting for pan offset
    let center_x = img_width / 2.0 - offset_x / zoom;
    let center_y = img_height / 2.0 - offset_y / zoom;

    // Calculate source rectangle
    let src_x = (center_x - crop_size / 2.0).max(0.0);
    let src_y = (center_y - crop_size / 2.0).max(0.0);
    let src_width = crop_size.min(img_width - src_x);
    let src_height = crop_size.min(img_height - src_y);

    // Use JavaScript to draw the image
    let draw_image_fn = js_sys::Reflect::get(&context, &JsValue::from_str("drawImage"))
        .map_err(|_| "Failed to get drawImage")?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "Failed to cast drawImage")?;

    let args = js_sys::Array::new();
    args.push(&img);
    args.push(&JsValue::from_f64(src_x));
    args.push(&JsValue::from_f64(src_y));
    args.push(&JsValue::from_f64(src_width));
    args.push(&JsValue::from_f64(src_height));
    args.push(&JsValue::from_f64(0.0));
    args.push(&JsValue::from_f64(0.0));
    args.push(&JsValue::from_f64(OUTPUT_SIZE as f64));
    args.push(&JsValue::from_f64(OUTPUT_SIZE as f64));

    let _ = js_sys::Reflect::apply(&draw_image_fn, &context, &args)
        .map_err(|_| "Failed to draw image")?;

    // Convert to JPEG with compression
    let quality = JsValue::from_f64(0.85);
    let data_url = canvas
        .to_data_url_with_type_and_encoder_options("image/jpeg", &quality)
        .map_err(|_| "Failed to convert to data URL")?;

    Ok(data_url)
}

#[cfg(not(feature = "hydrate"))]
async fn crop_and_process_image(
    image_data_url: String,
    _zoom: f64,
    _offset_x: f64,
    _offset_y: f64,
) -> Result<String, String> {
    // Server-side rendering: just return the original
    Ok(image_data_url)
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
        .map_err(|e| format!("Failed to construct URL: {e}"))?;

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
            .map_err(|e| format!("Failed to send request: {e}"))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Upload failed: {error_text}"));
        }

        let upload_response: UploadResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

        Ok(upload_response.profile_image_url)
    }

    #[cfg(not(feature = "ssr"))]
    {
        // In hydrate mode, use gloo to make the request
        use gloo::net::http::Request;

        let response = Request::post(url.as_str())
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {e}"))?
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {e}"))?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Upload failed: {error_text}"));
        }

        let upload_response: UploadResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {e}"))?;

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
    let file_error = RwSignal::new(Option::<String>::None);

    // Maximum file size: 5MB
    const MAX_FILE_SIZE: f64 = 5.0 * 1024.0 * 1024.0;

    let handle_file_change = move |ev: leptos::web_sys::Event| {
        let input: leptos::web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                file_error.set(None);

                // Check file size
                let file_size = file.size() as f64;
                if file_size > MAX_FILE_SIZE {
                    let size_mb = (file_size / (1024.0 * 1024.0)).round();
                    file_error.set(Some(format!("File too large: {size_mb}MB (max 5MB)")));
                    return;
                }

                // Check file type
                let file_type = file.type_();
                if !file_type.starts_with("image/") {
                    file_error.set(Some("Please select an image file (JPEG, PNG, WebP)".to_string()));
                    return;
                }

                let accepted_formats = ["image/jpeg", "image/png", "image/webp"];
                if !accepted_formats.iter().any(|&fmt| file_type == fmt) {
                    file_error.set(Some("Unsupported format. Please use JPEG, PNG, or WebP".to_string()));
                    return;
                }

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
                        accept="image/jpeg,image/png,image/webp"
                        node_ref=file_input_ref
                        on:change=handle_file_change
                        class="hidden"
                    />

                    // Error message display
                    <Show when=move || file_error.get().is_some()>
                        <div class="w-full p-3 bg-red-900/20 border border-red-500 rounded-lg mb-4">
                            <span class="text-[14px] text-red-400">
                                {move || file_error.get().unwrap_or_default()}
                            </span>
                        </div>
                    </Show>

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
                                    {move || { let percent = (zoom_level.get() * 100.0) as i32; format!("{percent}%") }}
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
                                if uploaded_image.get().is_some() {
                                    is_uploading.set(true);
                                    let delegated_identity = identity.id_wire.clone();

                                    // Get the cropped image data
                                    spawn_local(async move {
                                        // Create a canvas and crop the image
                                        let cropped_data = crop_and_process_image(
                                            uploaded_image.get_untracked().unwrap(),
                                            zoom_level.get_untracked(),
                                            position_x.get_untracked(),
                                            position_y.get_untracked()
                                        ).await;

                                        match cropped_data {
                                            Ok(processed_image) => {
                                                match upload_profile_image_to_agent(processed_image, delegated_identity).await {
                                                    Ok(new_url) => {
                                                        profile_pic_url.set(new_url);
                                                        show.set(false);
                                                        uploaded_image.set(None);
                                                        is_uploading.set(false);
                                                        leptos::logging::log!("Profile picture updated");
                                                    }
                                                    Err(e) => {
                                                        leptos::logging::error!("Failed to upload image: {}", e);
                                                        file_error.set(Some(format!("Upload failed: {}", e)));
                                                        is_uploading.set(false);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                leptos::logging::error!("Failed to process image: {}", e);
                                                file_error.set(Some(format!("Processing failed: {}", e)));
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
