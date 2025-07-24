use super::UploadParams;
use auth::delegate_short_lived_identity;
use candid::Principal;
use codee::string::JsonSerdeCodec;
use component::{back_btn::BackButton, buttons::GradientButton};
use consts::UPLOAD_URL;
use leptos::reactive::send_wrapper_ext::SendOption;
use leptos::{html::Input, prelude::*};
use leptos_icons::*;
use leptos_meta::Title;
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};
use serde_json::json;
use state::canisters::{auth_state, unauth_canisters};
use utils::event_streaming::events::VideoUploadInitiated;

// Local storage key for video generation parameters
const AI_VIDEO_PARAMS_STORE: &str = "ai_video_generation_params";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct VideoModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub duration: String,
    pub cost_sats: u64,
}

impl VideoModel {
    pub fn get_models() -> Vec<Self> {
        vec![
            VideoModel {
                id: "pollo_1_6".to_string(),
                name: "Pollo 1.6".to_string(),
                description: "Better, faster and cheaper".to_string(),
                duration: "60 Sec".to_string(),
                cost_sats: 5,
            },
            VideoModel {
                id: "cling_2_1_master".to_string(),
                name: "Cling 2.1 Master".to_string(),
                description: "Enhanced visual realism and motion".to_string(),
                duration: "60 Mins".to_string(),
                cost_sats: 25,
            },
            VideoModel {
                id: "cling_2_1".to_string(),
                name: "Cling 2.1".to_string(),
                description: "Enhanced visual realism and motion".to_string(),
                duration: "2 Mins".to_string(),
                cost_sats: 15,
            },
        ]
    }
}

#[component]
fn ModelDropdown(
    selected_model: RwSignal<VideoModel>,
    show_dropdown: RwSignal<bool>,
) -> impl IntoView {
    // Store models in a StoredValue to avoid the closure trait bounds issue
    let models = StoredValue::new(VideoModel::get_models());

    view! {
        <div class="relative w-full">
            <label class="block text-sm font-medium text-white mb-2">Model</label>

            // Selected model display
            <div
                class="flex items-center justify-between p-4 bg-neutral-900 border border-neutral-800 rounded-lg cursor-pointer hover:bg-neutral-800"
                on:click=move |_| show_dropdown.update(|show| *show = !*show)
            >
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 bg-pink-500 rounded-lg flex items-center justify-center">
                        <span class="text-white font-bold text-sm">"P"</span>
                    </div>
                    <div>
                        <div class="text-white font-medium">{selected_model.get().name}</div>
                        <div class="text-neutral-400 text-sm">{selected_model.get().description}</div>
                    </div>
                </div>
                <Icon
                    icon=icondata::AiDownOutlined
                    attr:class=move || format!("text-white transition-transform {}",
                        if show_dropdown.get() { "rotate-180" } else { "" }
                    )
                />
            </div>

            // Dropdown menu
            <Show when=show_dropdown>
                <div class="absolute top-full left-0 right-0 mt-1 bg-neutral-900 border border-neutral-800 rounded-lg shadow-lg z-10">
                    <For
                        each=move || models.get_value()
                        key=|model| model.id.clone()
                        children=move |model| {
                            let model_id = model.id.clone();
                            let model_name = model.name.clone();
                            let model_description = model.description.clone();
                            let model_duration = model.duration.clone();
                            let model_cost_sats = model.cost_sats;
                            let model_clone = model.clone();

                            let is_selected = Signal::derive(move || selected_model.get().id == model_id);
                            view! {
                                <div
                                    class="flex items-center justify-between p-4 hover:bg-neutral-800 cursor-pointer border-b border-neutral-800 last:border-b-0"
                                    on:click=move |_| {
                                        selected_model.set(model_clone.clone());
                                        show_dropdown.set(false);
                                    }
                                >
                                    <div class="flex items-center gap-3">
                                        <div class=move || format!("w-6 h-6 rounded-full border-2 flex items-center justify-center {}",
                                            if is_selected.get() { "border-pink-500 bg-pink-500" } else { "border-neutral-600" }
                                        )>
                                            <Show when=is_selected>
                                                <div class="w-2 h-2 bg-white rounded-full"></div>
                                            </Show>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            <div class="w-6 h-6 bg-pink-500 rounded flex items-center justify-center">
                                                <span class="text-white font-bold text-xs">"P"</span>
                                            </div>
                                            <div>
                                                <div class="text-white font-medium text-sm">{model_name}</div>
                                                <div class="text-neutral-400 text-xs">{model_description}</div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-4 text-xs">
                                        <div class="flex items-center gap-1">
                                            <Icon icon=icondata::AiClockCircleOutlined attr:class="text-neutral-400" />
                                            <span class="text-neutral-400">{model_duration}</span>
                                        </div>
                                        <div class="flex items-center gap-1">
                                            <span class="text-orange-400">"🪙"</span>
                                            <span class="text-orange-400">{format!("{} SATS", model_cost_sats)}</span>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct VideoGenerationParams {
    user_principal: Principal,
    prompt: String,
    model: VideoModel,
    image_data: Option<String>,
}

impl Default for VideoGenerationParams {
    fn default() -> Self {
        Self {
            user_principal: Principal::anonymous(),
            prompt: String::new(),
            model: VideoModel::default(),
            image_data: None,
        }
    }
}


use yral_types::delegated_identity::DelegatedIdentityWire;

// Server function to download AI video and upload using existing worker flow
#[server]
pub async fn upload_ai_video_from_url(
    video_url: String,
    hashtags: Vec<String>,
    description: String,
    delegated_identity_wire: DelegatedIdentityWire,
    is_nsfw: bool,
    enable_hot_or_not: bool,
) -> Result<String, ServerFnError> {
    leptos::logging::log!("Starting AI video upload from URL: {}", video_url);

    // Step 1: Download video using reqwest
    let client = reqwest::Client::new();
    let response = client
        .get(&video_url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to download video: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to download video: HTTP {}",
            response.status()
        )));
    }

    let video_bytes = response
        .bytes()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read video bytes: {}", e)))?;

    leptos::logging::log!("Downloaded video, size: {} bytes", video_bytes.len());

    // Step 2: Get upload URL from worker
    let upload_response = client
        .get(&format!("{}/get_upload_url_v2", UPLOAD_URL))
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get upload URL: {}", e)))?;

    if !upload_response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Failed to get upload URL: HTTP {}",
            upload_response.status()
        )));
    }

    let upload_response_text = upload_response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read upload URL response: {}", e)))?;

    #[derive(Deserialize)]
    struct UploadUrlResponse {
        data: Option<UploadUrlData>,
        success: bool,
        message: Option<String>,
    }

    #[derive(Deserialize)]
    struct UploadUrlData {
        uid: Option<String>,
        #[serde(rename = "uploadURL")]
        upload_url: Option<String>,
    }

    let upload_message: UploadUrlResponse = serde_json::from_str(&upload_response_text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse upload URL response: {}", e)))?;

    if !upload_message.success {
        return Err(ServerFnError::new(format!(
            "Upload URL request failed: {}",
            upload_message.message.unwrap_or_default()
        )));
    }

    let upload_data = upload_message
        .data
        .ok_or_else(|| ServerFnError::new("Upload URL data not found in response".to_string()))?;

    let upload_url = upload_data
        .upload_url
        .ok_or_else(|| ServerFnError::new("Upload URL not found in response".to_string()))?;

    let video_uid = upload_data
        .uid
        .ok_or_else(|| ServerFnError::new("Video UID not found in response".to_string()))?;

    leptos::logging::log!("Got upload URL and video UID: {}", video_uid);

    // Step 3: Upload to Cloudflare Stream
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(video_bytes.to_vec())
            .file_name("ai_generated_video.mp4")
            .mime_str("video/mp4")
            .map_err(|e| ServerFnError::new(format!("Failed to set MIME type: {}", e)))?,
    );

    let upload_result = client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to upload to Cloudflare: {}", e)))?;

    if !upload_result.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Cloudflare upload failed: HTTP {}",
            upload_result.status()
        )));
    }

    leptos::logging::log!("Successfully uploaded to Cloudflare Stream");

    // Step 4: Update metadata using types from video_upload.rs
    #[derive(Serialize)]
    struct VideoMetadata {
        title: String,
        description: String,
        tags: String,
    }

    #[derive(Serialize)]
    struct SerializablePostDetailsFromFrontend {
        is_nsfw: bool,
        hashtags: Vec<String>,
        description: String,
        video_uid: String,
        creator_consent_for_inclusion_in_hot_or_not: bool,
    }

    let metadata_request = json!({
        "video_uid": video_uid,
        "delegated_identity_wire": delegated_identity_wire,
        "meta": VideoMetadata{
            title: description.clone(),
            description: description.clone(),
            tags: hashtags.join(",")
        },
        "post_details": SerializablePostDetailsFromFrontend{
            is_nsfw,
            hashtags,
            description,
            video_uid: video_uid.clone(),
            creator_consent_for_inclusion_in_hot_or_not: enable_hot_or_not,
        }
    });

    let metadata_result = client
        .post(&format!("{}/update_metadata", UPLOAD_URL))
        .json(&metadata_request)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update metadata: {}", e)))?;

    if !metadata_result.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Metadata update failed: HTTP {}",
            metadata_result.status()
        )));
    }

    leptos::logging::log!("Successfully updated metadata for video: {}", video_uid);

    Ok(video_uid)
}

// Helper function to generate video using videogen.rs
async fn generate_video(
    _user_principal: Principal,
    prompt: String,
    _model: VideoModel,
    _image_data: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    // For now, simulate video generation with a delay
    // TODO: Implement actual videogen.rs integration
    leptos::logging::log!("Starting video generation with prompt: {}", prompt);

    // Simulate some processing time
    #[cfg(feature = "hydrate")]
    {
        gloo::timers::future::TimeoutFuture::new(2000).await;
    }

    // Mock successful response
    let mock_video_url = "https://storage.googleapis.com/yral_ai_generated_videos/veo-output/5790230970440583959/sample_0.mp4";

    leptos::logging::log!("Video generation completed with URL: {}", mock_video_url);
    Ok(mock_video_url.to_string())
}

#[component]
fn VideoGenerationLoadingScreen() -> impl IntoView {
    view! {
        <div class="flex flex-col bg-black min-w-dvw min-h-dvh">
            // Header with back button and title
            <div class="flex items-center justify-between p-4 pt-12">
                <div class="text-white">
                    <BackButton fallback="/upload-options".to_string() />
                </div>
                <h1 class="text-lg font-bold text-white">"Generate Video"</h1>
                <div class="w-6"></div> // Spacer for centering
            </div>

            // Main loading content
            <div class="flex-1 flex flex-col items-center justify-center px-4">
                <div class="flex flex-col items-center gap-8 max-w-md w-full">

                    // Progress animation circle
                    <div class="relative w-32 h-32">
                        // Outer circle (background)
                        <div class="absolute inset-0 rounded-full border-4 border-neutral-800"></div>

                        // Progress circle with gradient
                        <svg class="absolute inset-0 w-full h-full -rotate-90 animate-spin" viewBox="0 0 128 128">
                            <circle
                                cx="64"
                                cy="64"
                                r="60"
                                fill="none"
                                stroke="url(#gradient)"
                                stroke-width="4"
                                stroke-linecap="round"
                                stroke-dasharray="377"
                                stroke-dashoffset="94.25"
                                class="animate-pulse"
                            />
                            <defs>
                                <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="100%">
                                    <stop offset="0%" stop-color="#FF6DC4" />
                                    <stop offset="50%" stop-color="#F7007C" />
                                    <stop offset="100%" stop-color="#690039" />
                                </linearGradient>
                            </defs>
                        </svg>

                        // Center icon
                        <div class="absolute inset-0 flex items-center justify-center">
                            <Icon
                                icon=icondata::AiPlayCircleOutlined
                                attr:class="text-white text-4xl"
                            />
                        </div>
                    </div>

                    // Status text
                    <div class="text-center">
                        <h2 class="text-xl font-bold text-white mb-2">"Generating video"</h2>
                        <p class="text-sm text-neutral-400">"This may take a few minutes..."</p>
                    </div>

                    // Progress dots animation
                    <div class="flex items-center gap-2">
                        <div class="w-2 h-2 bg-pink-500 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
                        <div class="w-2 h-2 bg-pink-500 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
                        <div class="w-2 h-2 bg-pink-500 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn VideoResultScreen(
    video_url: String,
    _generate_action: Action<VideoGenerationParams, Result<String, String>>,
    on_upload: impl Fn() + 'static,
    on_regenerate: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="flex flex-col bg-black min-w-dvw min-h-dvh">
            // Header with back button and title
            <div class="flex items-center justify-between p-4 pt-12">
                <div class="text-white">
                    <BackButton fallback="/upload-options".to_string() />
                </div>
                <h1 class="text-lg font-bold text-white">"Generate Video"</h1>
                <div class="w-6"></div> // Spacer for centering
            </div>

            // Main video content
            <div class="flex-1 flex flex-col px-4 py-6 max-w-md mx-auto w-full">
                <div class="flex flex-col gap-6">

                    // Video preview section
                    <div class="w-full">
                        <video
                            class="w-full rounded-lg bg-neutral-900 aspect-video"
                            controls=true
                            preload="metadata"
                            src=video_url.clone()
                        >
                            <p class="text-white p-4">"Your browser doesn't support video playback."</p>
                        </video>
                    </div>

                    // Status text
                    <div class="text-center">
                        <h2 class="text-xl font-bold text-white mb-2">"Video generated successfully!"</h2>
                        <p class="text-sm text-neutral-400">"Your AI video is ready. You can re-generate or upload it."</p>
                    </div>

                    // Action buttons
                    <div class="flex flex-col gap-3 mt-4">

                        // Re-generate button
                        <button
                            class="w-full h-12 px-5 py-3 rounded-lg border-2 border-neutral-600 bg-transparent text-white font-bold hover:border-neutral-500 transition-colors flex items-center justify-center gap-2"
                            on:click=move |_| {
                                on_regenerate();
                            }
                        >
                            <Icon icon=icondata::AiReloadOutlined attr:class="text-lg" />
                            "Re-generate"
                        </button>

                        // Upload button (primary action)
                        <GradientButton
                            on_click=move || {
                                on_upload();
                            }
                            classes="w-full h-12 rounded-lg font-bold".to_string()
                            disabled=Signal::derive(|| false)
                        >
                            <div class="flex items-center justify-center gap-2">
                                <Icon icon=icondata::AiUploadOutlined attr:class="text-lg" />
                                "Upload"
                            </div>
                        </GradientButton>
                    </div>

                    // Video info (optional)
                    <div class="mt-6 p-4 bg-neutral-900 rounded-lg">
                        <div class="flex items-center justify-between text-sm">
                            <span class="text-neutral-400">"Duration:"</span>
                            <span class="text-white">"Auto-detected"</span>
                        </div>
                        <div class="flex items-center justify-between text-sm mt-2">
                            <span class="text-neutral-400">"Format:"</span>
                            <span class="text-white">"MP4"</span>
                        </div>
                        <div class="flex items-center justify-between text-sm mt-2">
                            <span class="text-neutral-400">"Quality:"</span>
                            <span class="text-white">"HD"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PreUploadAiView(
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

    // Balance state (mock for now - will integrate with real balance later)
    let user_balance = RwSignal::new(100u64); // Current balance in SATS

    // Get auth state and user principal
    let auth = auth_state();
    let user_principal_opt = auth.user_principal_if_available();

    // Form validation
    let form_valid = Signal::derive(move || !prompt_text.get().trim().is_empty());
    let sufficient_balance =
        Signal::derive(move || user_balance.get() >= selected_model.get().cost_sats);
    let can_generate = Signal::derive(move || {
        form_valid.get() && sufficient_balance.get() && !generate_action.pending().get()
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
                        // Create object URL for preview
                        let url =
                            web_sys::Url::create_object_url_with_blob(&file).unwrap_or_default();
                        uploaded_image.set(Some(url));
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
                            <button class="absolute bottom-3 left-3 flex items-center gap-1 px-2 py-1 bg-neutral-800 rounded text-xs text-neutral-300 hover:text-white transition-colors">
                                <Icon icon=icondata::AiStarOutlined attr:class="text-sm" />
                                "Generate with AI"
                            </button>

                            // Character counter
                            <div class="absolute bottom-3 right-3 text-xs text-neutral-400">
                                {move || format!("{}/500", character_count.get())}
                            </div>
                        </div>
                    </div>

                    // SATS Required Section
                    <div class="flex items-center justify-between p-4 bg-neutral-900 rounded-lg">
                        <div class="flex items-center gap-2">
                            <span class="text-white font-medium">"SATS Required:"</span>
                            <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                        </div>
                        <div class="flex items-center gap-1">
                            <span class="text-orange-400">"🪙"</span>
                            <span class="text-orange-400 font-bold">{move || format!("{} SATS", selected_model.get().cost_sats)}</span>
                        </div>
                    </div>

                    // Current Balance
                    <div class="text-center text-sm text-neutral-400">
                        {move || format!("(Current balance: {}SATS)", user_balance.get())}
                    </div>

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
                                if let Some(user_principal) = user_principal_opt {
                                    // Get current form values
                                    let prompt = prompt_text.get_untracked();
                                    let model = selected_model.get_untracked();
                                    let image_data = uploaded_image.get_untracked();

                                    // Create params struct and dispatch the action - this is the clean way!
                                    let params = VideoGenerationParams {
                                        user_principal,
                                        prompt,
                                        model,
                                        image_data,
                                    };
                                    generate_action.dispatch(params);
                                } else {
                                    leptos::logging::warn!("User not logged in");
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
    }
}

#[component]
pub fn UploadAiPostPage() -> impl IntoView {
    let trigger_upload = RwSignal::new(SendOption::<UploadParams>::new_local(None));
    let uid = RwSignal::new(None);
    let upload_file_actual_progress = RwSignal::new(0.0f64);

    // Signal to control returning to form for re-generation
    let show_form = RwSignal::new(true);

    // Local storage for video generation parameters (for regeneration)
    let (stored_params, set_stored_params, _) =
        use_local_storage::<VideoGenerationParams, JsonSerdeCodec>(AI_VIDEO_PARAMS_STORE);

    // Get auth state outside actions for reuse
    let auth = auth_state();

    // Video generation action - this is the proper way to handle async operations
    let generate_action: Action<VideoGenerationParams, Result<String, String>> =
        Action::new_unsync({
            let show_form = show_form;
            let set_stored_params = set_stored_params;
            move |params: &VideoGenerationParams| {
                let params = params.clone();
                let show_form = show_form;
                let set_stored_params = set_stored_params;

                // Store parameters for regeneration
                set_stored_params.set(params.clone());

                async move {
                    match generate_video(
                        params.user_principal,
                        params.prompt,
                        params.model,
                        params.image_data,
                    )
                    .await
                    {
                        Ok(video_url) => {
                            // Set show_form to false to show result screen
                            show_form.set(false);
                            Ok(video_url)
                        }
                        Err(err) => {
                            leptos::logging::error!("Video generation failed: {}", err);
                            Err(format!("Failed to generate video: {}", err))
                        }
                    }
                }
            }
        });

    // Upload action parameters struct
    #[derive(Clone)]
    struct UploadActionParams {
        video_url: String,
    }

    // Upload action - handles server-side video download and upload
    let upload_action: Action<UploadActionParams, Result<String, String>> =
        Action::new_unsync({
            let auth = auth.clone();
            move |params: &UploadActionParams| {
                let params = params.clone();
                let auth = auth.clone();
                async move {
                    // Get unauth_canisters within the Action (like video_upload.rs)
                    let unauth_cans = unauth_canisters();
                    
                    // Get delegated identity within the Action
                    match auth.auth_cans(unauth_cans).await {
                        Ok(canisters) => {
                            let id = canisters.identity();
                            let delegated_identity = delegate_short_lived_identity(id);
                            
                            // Call server function with delegated identity
                            match upload_ai_video_from_url(
                                params.video_url,
                                vec!["AI".to_string(), "Generated".to_string()],
                                "AI Generated Video".to_string(),
                                delegated_identity,
                                false, // is_nsfw
                                false, // enable_hot_or_not
                            ).await {
                                Ok(video_uid) => {
                                    leptos::logging::log!("Video uploaded successfully with UID: {}", video_uid);
                                    Ok(video_uid)
                                }
                                Err(e) => {
                                    leptos::logging::error!("Failed to upload video: {}", e);
                                    Err(format!("Upload failed: {}", e))
                                }
                            }
                        }
                        Err(e) => {
                            leptos::logging::error!("Failed to get auth canisters: {:?}", e);
                            Err(format!("Auth failed: {:?}", e))
                        }
                    }
                }
            }
        });

    view! {
        <Title text="YRAL AI - Upload" />
        <div class="w-full h-full">
            <Show
                when=move || generate_action.pending().get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || {
                                // Show result screen if video generation was successful AND we're not in form mode
                                if let Some(result) = generate_action.value().get() {
                                    result.is_ok() && !show_form.get()
                                } else {
                                    false
                                }
                            }
                            fallback=move || {
                                view! {
                                    <Show
                                        when=move || { trigger_upload.with(|trigger_upload| (**trigger_upload).is_some()) }
                                        fallback=move || {
                                            view! {
                                                <PreUploadAiView
                                                    _trigger_upload=trigger_upload.write_only()
                                                    _uid=uid
                                                    _upload_file_actual_progress=upload_file_actual_progress.write_only()
                                                    generate_action=generate_action
                                                />
                                            }
                                        }
                                    >
                                        // TODO: Implement video upload flow after generation
                                        <div class="flex items-center justify-center min-h-screen bg-black text-white">
                                            "Video upload flow coming soon..."
                                        </div>
                                    </Show>
                                }
                            }
                        >
                            <VideoResultScreen
                                video_url={
                                    generate_action.value().get()
                                        .and_then(|result| result.ok())
                                        .unwrap_or_default()
                                }
                                _generate_action=generate_action
                                on_upload=move || {
                                    // Get the generated video URL and upload using Action
                                    if let Some(result) = generate_action.value().get() {
                                        if let Ok(video_url) = result {
                                            leptos::logging::log!("Starting upload Action for video: {}", video_url);
                                            
                                            // Dispatch the upload action - it handles auth internally
                                            upload_action.dispatch(UploadActionParams {
                                                video_url,
                                            });
                                        }
                                    }
                                }
                                on_regenerate=move || {
                                    // Get the stored parameters and regenerate
                                    let params = stored_params.get_untracked();
                                    // Check if we have valid parameters (not default/empty)
                                    if !params.prompt.is_empty() && params.user_principal != Principal::anonymous() {
                                        leptos::logging::log!("Re-generating video with stored parameters");
                                        // Dispatch the action again with the stored parameters
                                        generate_action.dispatch(params);
                                    } else {
                                        leptos::logging::warn!("No valid stored parameters found for regeneration");
                                        // Fallback to showing form
                                        show_form.set(true);
                                    }
                                }
                            />
                        </Show>
                    }
                }
            >
                <VideoGenerationLoadingScreen />
            </Show>
        </div>
    }
}
