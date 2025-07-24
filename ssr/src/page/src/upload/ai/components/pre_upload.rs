use super::ModelDropdown;
use crate::upload::ai::models::{VideoGenerationParams, VideoModel};
use crate::upload::UploadParams;
use component::{back_btn::BackButton, buttons::GradientButton, login_modal::LoginModal};
use leptos::reactive::send_wrapper_ext::SendOption;
use leptos::{html::Input, prelude::*};
use leptos_icons::*;
use state::canisters::auth_state;
use utils::event_streaming::events::VideoUploadInitiated;

#[component]
pub fn PreUploadAiView(
    _trigger_upload: WriteSignal<SendOption<UploadParams>>,
    _uid: RwSignal<Option<String>>,
    _upload_file_actual_progress: WriteSignal<f64>,
    generate_action: Action<VideoGenerationParams, Result<String, String>>,
) -> impl IntoView {
    // Form state
    let selected_model = RwSignal::new(VideoModel::get_models().into_iter().next().unwrap());
    let show_dropdown = RwSignal::new(false);
    let prompt_text = RwSignal::new(String::new());
    let character_count = Signal::derive(move || prompt_text.get().len());
    let uploaded_image = RwSignal::new(None::<String>);

    // Login modal state
    let show_login_modal = RwSignal::new(false);

    // Balance state (mock for now - will integrate with real balance later)
    // let user_balance = RwSignal::new(100u64); // Current balance in SATS

    // Get auth state and user principal
    let auth = auth_state();
    let user_principal_opt = auth.user_principal_if_available();
    let is_logged_in = Signal::stored(true); // auth.is_logged_in_with_oauth();

    // Form validation
    let form_valid = Signal::derive(move || !prompt_text.get().trim().is_empty());
    // let sufficient_balance =
    //     Signal::derive(move || user_balance.get() >= selected_model.get().cost_sats);
    let can_generate = Signal::derive(move || {
        // Allow button click for non-logged-in users (to show login modal)
        // For logged-in users, check form validity and balance
        // !is_logged_in.get() ||
        // (form_valid.get() && sufficient_balance.get() && !generate_action.pending().get())
        form_valid.get() && !generate_action.pending().get()
    });

    // Error handling from action
    let generation_error = Signal::derive(move || {
        if let Some(result) = generate_action.value().get() {
            match result {
                Err(err) => Some(err),
                Ok(_) => None,
            }
        } else {
            None
        }
    });

    // File input for image upload
    let image_input = NodeRef::<Input>::new();

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    VideoUploadInitiated.send_event(ev_ctx);

    // Handle image upload
    let handle_image_upload = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(input) = image_input.get() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        use wasm_bindgen::prelude::*;
                        use wasm_bindgen::JsCast;
                        use web_sys::FileReader;
                        
                        // Log the file's mime type
                        leptos::logging::log!("File type from browser: {}", file.type_());
                        
                        let file_reader = FileReader::new().unwrap();
                        let file_clone = file.clone();
                        
                        // Set up callback for when file is loaded
                        let uploaded_image_clone = uploaded_image.clone();
                        let onload = Closure::wrap(Box::new(move |event: web_sys::Event| {
                            if let Some(target) = event.target() {
                                if let Ok(reader) = target.dyn_into::<FileReader>() {
                                    if let Ok(result) = reader.result() {
                                        if let Some(data_url) = result.as_string() {
                                            // Set the data URL which includes the base64 data
                                            leptos::logging::log!("Image uploaded as data URL: {}", &data_url[..50.min(data_url.len())]);
                                            uploaded_image_clone.set(Some(data_url));
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>);
                        
                        file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                        onload.forget();
                        
                        // Read file as data URL (includes mime type and base64 data)
                        let _ = file_reader.read_as_data_url(&file_clone);
                    }
                }
            }
        }
    };

    view! {
        <div class="flex flex-col bg-black min-w-dvw min-h-dvh">
            // Header with back button and title
            <div class="flex items-center justify-between p-4 pt-12">
                <div class="text-white">
                    <BackButton fallback="/upload-options".to_string() />
                </div>
                <h1 class="text-lg font-bold text-white">"Create AI Video"</h1>
                <div class="w-6"></div> // Spacer for centering
            </div>

            // Main form content
            <div class="flex-1 px-4 py-6 pb-24 max-w-md mx-auto w-full">
                <div class="flex flex-col gap-6">

                    // Model Selection Dropdown
                    <ModelDropdown selected_model=selected_model show_dropdown=show_dropdown />

                    // Image Upload Section (Optional)
                    <div class="w-full">
                        <div class="flex items-center gap-2 mb-2">
                            <label class="block text-sm font-medium text-white">Image</label>
                            <span class="text-xs text-neutral-400">(Optional)</span>
                            <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                        </div>

                        <div class="relative">
                            <input
                                type="file"
                                accept="image/*"
                                node_ref=image_input
                                on:change=handle_image_upload
                                class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                            />
                            <div class="flex flex-col items-center justify-center p-12 bg-neutral-900 border border-neutral-800 rounded-lg hover:bg-neutral-800 transition-colors cursor-pointer">
                                <Show
                                    when=move || uploaded_image.get().is_some()
                                    fallback=move || view! {
                                        <div class="flex flex-col items-center gap-3">
                                            <Icon icon=icondata::AiPictureOutlined attr:class="text-neutral-500 text-3xl" />
                                            <span class="text-neutral-500 text-sm">"Click to upload an image"</span>
                                        </div>
                                    }
                                >
                                    <img
                                        src=move || uploaded_image.get().unwrap_or_default()
                                        class="max-w-full max-h-32 object-contain rounded"
                                        alt="Uploaded preview"
                                    />
                                </Show>
                            </div>
                        </div>
                    </div>

                    // Prompt Section
                    <div class="w-full">
                        <label class="block text-sm font-medium text-white mb-2">Prompt</label>
                        <div class="relative">
                            <textarea
                                class="w-full p-4 bg-neutral-900 border border-neutral-800 rounded-lg text-white placeholder:text-neutral-500 resize-none focus:outline-none focus:border-pink-400 transition-colors"
                                rows=6
                                placeholder="Enter the Prompt here..."
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    if value.len() <= 500 {
                                        prompt_text.set(value);
                                    }
                                }
                                prop:value=move || prompt_text.get()
                            ></textarea>

                            // Generate with AI button inside textarea
                            // <button class="absolute bottom-3 left-3 flex items-center gap-1 px-2 py-1 bg-neutral-800 rounded text-xs text-neutral-300 hover:text-white transition-colors">
                            //     <Icon icon=icondata::AiStarOutlined attr:class="text-sm" />
                            //     "Generate with AI"
                            // </button>

                            // Character counter
                            <div class="absolute bottom-3 right-3 text-xs text-neutral-400">
                                {move || format!("{}/500", character_count.get())}
                            </div>
                        </div>
                    </div>

                    // SATS Required Section
                    // <div class="flex items-center justify-between p-4 bg-neutral-900 rounded-lg">
                    //     <div class="flex items-center gap-2">
                    //         <span class="text-white font-medium">"SATS Required:"</span>
                    //         <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                    //     </div>
                    //     <div class="flex items-center gap-1">
                    //         <span class="text-orange-400">"🪙"</span>
                    //         <span class="text-orange-400 font-bold">{move || format!("{} SATS", selected_model.get().cost_sats)}</span>
                    //     </div>
                    // </div>

                    // Current Balance
                    // <div class="text-center text-sm text-neutral-400">
                    //     {move || format!("(Current balance: {}SATS)", user_balance.get())}
                    // </div>

                    // Error display
                    <Show when=move || generation_error.get().is_some()>
                        <div class="p-3 bg-red-900/20 border border-red-500/30 rounded-lg">
                            <div class="text-red-400 text-sm">
                                {move || generation_error.get().unwrap_or_default()}
                            </div>
                        </div>
                    </Show>

                    // Create AI Video Button
                    <div class="mt-4">
                        <GradientButton
                            on_click=move || {
                                // Check if user is logged in
                                if !is_logged_in.get_untracked() {
                                    // Show login modal if not logged in
                                    show_login_modal.set(true);
                                } else if let Some(user_principal) = user_principal_opt {
                                    // Get current form values
                                    let prompt = prompt_text.get_untracked();
                                    let model = selected_model.get_untracked();
                                    let image_data = uploaded_image.get_untracked();
                                    
                                    leptos::logging::log!("Dispatching video generation with image_data: {:?}", 
                                        image_data.as_ref().map(|d| &d[..50.min(d.len())]));

                                    // Create params struct and dispatch the action - this is the clean way!
                                    let params = VideoGenerationParams {
                                        user_principal,
                                        prompt,
                                        model,
                                        image_data,
                                    };
                                    generate_action.dispatch(params);
                                } else {
                                    leptos::logging::warn!("User logged in but no principal found");
                                }
                            }
                            classes="w-full h-12 rounded-lg font-bold".to_string()
                            disabled=Signal::derive(move || !can_generate.get())
                        >
                            "Create AI Video"
                        </GradientButton>
                    </div>
                </div>
            </div>
        </div>

        // Login Modal
        <LoginModal show=show_login_modal redirect_to=None />
    }
}
