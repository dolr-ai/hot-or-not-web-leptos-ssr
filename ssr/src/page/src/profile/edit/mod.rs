pub mod username;

use component::{
    back_btn::BackButton, modal::Modal, spinner::Spinner, title::TitleText,
};
use leptos::{either::Either, html, prelude::*, server_fn::ServerFnError, task::spawn_local};
use leptos_icons::Icon;
use leptos_meta::Title;
use leptos_router::components::Redirect;
use state::{app_state::AppState, canisters::auth_state};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
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
    let profile_pic_url = RwSignal::new(details.profile_pic_or_random());
    let show_image_editor = RwSignal::new(false);

    let on_save = move || {
        // Handle save action
        leptos::logging::log!("Save clicked");
    };

    view! {
        <div class="flex flex-col w-full items-center">
            // Image Editor Popup
            <Show when=move || show_image_editor.get()>
                <ProfileImageEditor show=show_image_editor profile_pic_url />
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

#[server]
async fn upload_profile_image(image_data: String) -> Result<String, ServerFnError> {
    // For now, just return a dummy URL as requested
    // In production, this would upload to a storage service
    leptos::logging::log!("Received image data of length: {}", image_data.len());

    // Return a dummy profile picture URL
    Ok("https://api.dicebear.com/7.x/avataaars/svg?seed=updated".to_string())
}

#[component]
fn ProfileImageEditor(
    show: RwSignal<bool>,
    profile_pic_url: RwSignal<String>,
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
                        on:click=move |_| {
                            if let Some(image_data) = uploaded_image.get() {
                                is_uploading.set(true);

                                spawn_local(async move {
                                    match upload_profile_image(image_data).await {
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