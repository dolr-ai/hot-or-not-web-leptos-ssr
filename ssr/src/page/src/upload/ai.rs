use super::{
    validators::{description_validator, hashtags_validator},
    UploadParams,
};
use crate::upload::video_upload::VideoUploader;
use component::buttons::HighlightedButton;
use gloo::net::http::Request;
use leptos::web_sys::Blob;
use leptos::{
    html::{Input, Textarea, Video},
    prelude::*,
};
use leptos_meta::Title;
use serde::{Deserialize, Serialize};
use state::canisters::auth_state;
use utils::{
    event_streaming::events::{VideoUploadInitiated, VideoUploadUploadButtonClicked},
    web::FileWithUrl,
};

const API_KEY: &str = "d364f1fe-209b-43e5-9dff-386c39b67682";
const API_BASE_URL: &str = "https://veo3-video-generator-874803795683.us-central1.run.app";

#[derive(Serialize)]
struct GenerateVideoRequest {
    prompt: String,
    sample_count: u32,
    generate_audio: bool,
    aspect_ratio: String,
    negative_prompt: String,
    duration_seconds: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GenerateVideoResponse {
    operation_name: String,
    message: String,
}

#[derive(Serialize)]
struct CheckStatusRequest {
    operation_name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CheckStatusResponse {
    completed: bool,
    gcs_uri: Option<String>,
    operation_name: String,
    message: String,
}

#[component]
fn PreUploadAiView(
    trigger_upload: WriteSignal<Option<UploadParams>, LocalStorage>,
    _uid: RwSignal<Option<String>, LocalStorage>,
    _upload_file_actual_progress: WriteSignal<f64>,
) -> impl IntoView {
    let description_err = RwSignal::new(String::new());
    let desc_err_memo = Memo::new(move |_| description_err());
    let hashtags = RwSignal::new(Vec::new());
    let hashtags_err = RwSignal::new(String::new());
    let hashtags_err_memo = Memo::new(move |_| hashtags_err());
    let file_blob = RwSignal::new_local(None::<FileWithUrl>);
    let desc = NodeRef::<Textarea>::new();
    let prompt_input = NodeRef::<Textarea>::new();
    let video_ref = NodeRef::<Video>::new();

    let generation_error = RwSignal::new(None::<String>);
    let polling_status = RwSignal::new(String::new());

    let invalid_form = Memo::new(move |_| {
        !desc_err_memo.with(|desc_err_memo| desc_err_memo.is_empty())
            || !hashtags_err_memo.with(|hashtags_err_memo| hashtags_err_memo.is_empty())
            || hashtags.with(|hashtags| hashtags.is_empty())
            || desc.get().map(|d| d.value().is_empty()).unwrap_or(true)
    });

    let hashtag_inp = NodeRef::<Input>::new();
    let is_nsfw = NodeRef::<Input>::new();

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    VideoUploadInitiated.send_event(ev_ctx);

    let generate_action: Action<(), _> = Action::new_local(move |_| {
        leptos::logging::log!("Generating video from prompt...");
        let prompt_input = prompt_input.clone();
        let generation_error = generation_error.clone();
        let polling_status = polling_status.clone();
        let file_blob = file_blob.clone();
        let video_ref = video_ref.clone();

        async move {
            let Some(prompt_elem) = prompt_input.get() else {
                return;
            };

            let prompt = prompt_elem.value();
            if prompt.is_empty() {
                generation_error.set(Some("Please enter a prompt".to_string()));
                return;
            }

            // is_generating.set(true);
            generation_error.set(None);
            polling_status.set("Generating video...".to_string());

            let request_body = GenerateVideoRequest {
                prompt,
                sample_count: 1,
                generate_audio: true,
                aspect_ratio: "16:9".to_string(),
                negative_prompt: String::new(),
                duration_seconds: 8,
            };

            match Request::post(&format!("{}/generate_video_from_prompt", API_BASE_URL))
                .header("accept", "application/json")
                .header("X-API-Key", API_KEY)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .expect("Failed to build request")
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<GenerateVideoResponse>().await {
                            Ok(gen_response) => {
                                polling_status.set(
                                    "Video generation started, polling for completion..."
                                        .to_string(),
                                );
                                #[cfg(feature = "hydrate")]
                                poll_for_video(
                                    gen_response.operation_name,
                                    file_blob,
                                    // is_generating,
                                    generation_error,
                                    polling_status,
                                    video_ref,
                                )
                                .await;

                                #[cfg(not(feature = "hydrate"))]
                                {
                                    generation_error.set(Some(
                                        "Video generation is only supported in the browser"
                                            .to_string(),
                                    ));
                                }
                            }
                            Err(e) => {
                                generation_error
                                    .set(Some(format!("Failed to parse response: {}", e)));
                                // is_generating.set(false);
                            }
                        }
                    } else {
                        generation_error.set(Some(format!("API error: {}", response.status())));
                        // is_generating.set(false);
                    }
                }
                Err(e) => {
                    generation_error.set(Some(format!("Request failed: {}", e)));
                    // is_generating.set(false);
                }
            }
        }
    });

    let on_submit = move || {
        VideoUploadUploadButtonClicked.send_event(ev_ctx, hashtag_inp, is_nsfw, NodeRef::new());

        let description = desc.get_untracked().unwrap().value();
        let hashtags = hashtags.get_untracked();
        let Some(file_blob) = file_blob.get_untracked() else {
            return;
        };
        trigger_upload.set(Some(UploadParams {
            file_blob,
            hashtags,
            description,
            enable_hot_or_not: false,
            is_nsfw: is_nsfw
                .get_untracked()
                .map(|v| v.checked())
                .unwrap_or_default(),
        }));
    };

    let hashtag_on_input = move |hts| match hashtags_validator(hts) {
        Ok(hts) => {
            hashtags.set(hts);
            hashtags_err.set(String::new());
        }
        Err(e) => hashtags_err.set(e),
    };

    Effect::new(move |_| {
        let Some(hashtag_inp) = hashtag_inp.get() else {
            return;
        };

        let val = hashtag_inp.value();
        if !val.is_empty() {
            hashtag_on_input(val);
        }
    });

    view! {
        <div class="flex flex-col gap-4 justify-center items-center p-0 mx-auto w-full min-h-screen bg-transparent lg:flex-row lg:gap-20">
            <div class="flex flex-col justify-center items-center px-2 mx-4 mt-4 mb-4 text-center rounded-2xl sm:px-4 sm:mx-6 sm:w-full sm:h-auto lg:overflow-y-auto lg:px-0 lg:mx-0 w-[358px] h-[300px] sm:min-h-[380px] sm:max-h-[70vh] lg:w-[627px] lg:h-[600px]">
                <Show
                    when=move || file_blob.with(|f| f.is_some())
                    fallback=move || view! {
                        <div class="flex flex-col gap-4 w-full">
                            <h3 class="text-2xl font-light text-white">AI Video Generator</h3>
                            <div class="flex flex-col gap-2">
                                <label for="prompt-input" class="text-neutral-300 font-light text-lg">
                                    Enter your prompt
                                </label>
                                <textarea
                                    id="prompt-input"
                                    node_ref=prompt_input
                                    class="p-3 min-w-full rounded-lg border transition outline-none focus:border-pink-400 focus:ring-pink-400 bg-neutral-900 border-neutral-800 text-[15px] placeholder:text-neutral-500 placeholder:font-light"
                                    rows=4
                                    placeholder="a beautiful scenery"
                                    // disabled=is_generating.get()
                                ></textarea>
                            </div>
                            <Show when=move || generation_error.with(|e| e.is_some())>
                                <div class="text-red-500 text-sm">{move || generation_error.get()}</div>
                            </Show>
                            <Show when=move || generate_action.pending().get()>
                                <div class="text-white text-sm animate-pulse">{move || polling_status.get()}</div>
                            </Show>
                            <button
                                on:click=move |_| { generate_action.dispatch(()); }
                                disabled=false
                                class="w-full mx-auto py-[12px] px-[20px] rounded-xl bg-linear-to-r from-pink-300 to-pink-500 text-white font-light text-[17px] transition disabled:opacity-60 disabled:cursor-not-allowed".to_string()
                            >
                                {move || if generate_action.pending().get() { "Generating..." } else { "Generate Video" }}
                            </button>
                        </div>
                    }
                >
                    <div class="relative w-full h-full">
                        <video
                            node_ref=video_ref
                            class="w-full h-full object-contain rounded-lg"
                            controls=true
                            autoplay=true
                        ></video>
                        <button
                            on:click=move |_| {
                                file_blob.set(None);
                                generation_error.set(None);
                                if let Some(video) = video_ref.get() {
                                    video.set_src("");
                                }
                            }
                            class="absolute top-2 right-2 p-2 bg-red-500 text-white rounded-full hover:bg-red-600 transition"
                        >
                            "X"
                        </button>
                    </div>
                </Show>
            </div>
            // <div class="flex overflow-y-auto flex-col gap-4 justify-between p-2 w-full h-auto rounded-2xl max-w-[627px] min-h-[400px] max-h-[90vh] lg:w-[627px] lg:h-[600px]">
            //     <h2 class="mb-2 font-light text-white text-[32px]">Upload AI Video</h2>
            //     <div class="flex flex-col gap-y-1">
            //         <label for="caption-input" class="mb-1 font-light text-[20px] text-neutral-300">
            //             Caption
            //         </label>
            //         <Show when=move || {
            //             description_err.with(|description_err| !description_err.is_empty())
            //         }>
            //             <span class="text-sm text-red-500">{desc_err_memo()}</span>
            //         </Show>
            //         <textarea
            //             id="caption-input"
            //             node_ref=desc
            //             on:input=move |ev| {
            //                 let desc = event_target_value(&ev);
            //                 description_err
            //                     .set(description_validator(desc).err().unwrap_or_default());
            //             }
            //             class="p-3 min-w-full rounded-lg border transition outline-none focus:border-pink-400 focus:ring-pink-400 bg-neutral-900 border-neutral-800 text-[15px] placeholder:text-neutral-500 placeholder:font-light"
            //             rows=12
            //             placeholder="Enter the caption here"
            //             disabled=move || file_blob.with(|f| f.is_none())
            //         ></textarea>
            //     </div>
            //     <div class="flex flex-col gap-y-1 mt-2">
            //         <label for="hashtag-input" class="mb-1 font-light text-[20px] text-neutral-300">
            //             Add Hashtag
            //         </label>
            //         <Show when=move || {
            //             hashtags_err.with(|hashtags_err| !hashtags_err.is_empty())
            //         }>
            //             <span class="text-sm font-semibold text-red-500">
            //                 {hashtags_err_memo()}
            //             </span>
            //         </Show>
            //         <input
            //             id="hashtag-input"
            //             node_ref=hashtag_inp
            //             on:input=move |ev| {
            //                 let hts = event_target_value(&ev);
            //                 hashtag_on_input(hts);
            //             }
            //             class="p-3 rounded-lg border transition outline-none focus:border-pink-400 focus:ring-pink-400 bg-neutral-900 border-neutral-800 text-[15px] placeholder:text-neutral-500 placeholder:font-light"
            //             type="text"
            //             placeholder="Hit enter to add #hashtags"
            //             disabled=move || file_blob.with(|f| f.is_none())
            //         />
            //     </div>
            //     <div class="flex items-center gap-2">
            //         <input
            //             id="nsfw-checkbox"
            //             node_ref=is_nsfw
            //             type="checkbox"
            //             class="w-4 h-4 text-pink-500 bg-gray-100 border-gray-300 rounded focus:ring-pink-400"
            //             disabled=move || file_blob.with(|f| f.is_none())
            //         />
            //         <label for="nsfw-checkbox" class="text-neutral-300 text-sm">
            //             This content is NSFW
            //         </label>
            //     </div>
            //     {move || {
            //         let disa = invalid_form.get() || file_blob.with(|f| f.is_none());
            //         view! {
            //             <HighlightedButton
            //                 on_click=move || on_submit()
            //                 disabled=disa
            //                 classes="w-full mx-auto py-[12px] px-[20px] rounded-xl bg-linear-to-r from-pink-300 to-pink-500 text-white font-light text-[17px] transition disabled:opacity-60 disabled:cursor-not-allowed"
            //                     .to_string()
            //             >
            //                 "Upload"
            //             </HighlightedButton>
            //         }
            //     }}
            // </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
async fn poll_for_video(
    operation_name: String,
    file_blob: RwSignal<Option<FileWithUrl>, LocalStorage>,
    // is_generating: RwSignal<bool>,
    generation_error: RwSignal<Option<String>>,
    polling_status: RwSignal<String>,
    video_ref: NodeRef<Video>,
) {
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 60; // Poll for up to 5 minutes (60 * 5 seconds)

    loop {
        if attempts >= MAX_ATTEMPTS {
            generation_error.set(Some("Video generation timed out".to_string()));
            break;
        }

        attempts += 1;
        polling_status.set(format!("Checking status... (attempt {})", attempts));

        // Wait 5 seconds before polling
        gloo::timers::future::sleep(std::time::Duration::from_secs(5)).await;

        let request_body = CheckStatusRequest {
            operation_name: operation_name.clone(),
        };

        match Request::post(&format!("{}/check_generation_status", API_BASE_URL))
            .header("accept", "application/json")
            .header("X-API-Key", API_KEY)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .expect("Failed to build request")
            .send()
            .await
        {
            Ok(response) => {
                if response.ok() {
                    match response.json::<CheckStatusResponse>().await {
                        Ok(status_response) => {
                            if status_response.completed {
                                if let Some(gcs_uri) = status_response.gcs_uri {
                                    polling_status.set("Video generated! Loading...".to_string());
                                    convert_gcs_to_blob(
                                        gcs_uri,
                                        file_blob,
                                        generation_error,
                                        video_ref,
                                    )
                                    .await;
                                    break;
                                } else {
                                    generation_error.set(Some(
                                        "Video completed but no URL provided".to_string(),
                                    ));
                                    // is_generating.set(false);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            generation_error
                                .set(Some(format!("Failed to parse status response: {}", e)));
                            // is_generating.set(false);
                            break;
                        }
                    }
                } else {
                    generation_error.set(Some(format!(
                        "Status check API error: {}",
                        response.status()
                    )));
                    // is_generating.set(false);
                    break;
                }
            }
            Err(e) => {
                generation_error.set(Some(format!("Status check request failed: {}", e)));
                // is_generating.set(false);
                break;
            }
        }
    }
}

#[cfg(feature = "hydrate")]
async fn convert_gcs_to_blob(
    _gcs_uri: String,
    file_blob: RwSignal<Option<FileWithUrl>, LocalStorage>,
    generation_error: RwSignal<Option<String>>,
    video_ref: NodeRef<Video>,
) {
    // Convert GCS URI to a public URL
    // For now, we'll assume the video is publicly accessible
    // In production, you might need to use a signed URL or proxy through your backend
    // let public_url = gcs_uri.replace("gs://", "https://storage.googleapis.com/");

    // for testing
    let public_url = "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/2472e3f1cbb742038f0e86a27c8ac98a/downloads/default.mp4";

    match Request::get(&public_url).send().await {
        Ok(response) => {
            if response.ok() {
                match response.binary().await {
                    Ok(bytes) => {
                        let array = js_sys::Uint8Array::from(&bytes[..]);
                        let blob = Blob::new_with_u8_array_sequence(&js_sys::Array::of1(&array))
                            .expect("Failed to create blob");

                        // Create a File from the Blob
                        let file_options = web_sys::FilePropertyBag::new();
                        file_options.set_type("video/mp4");
                        let file = web_sys::File::new_with_blob_sequence_and_options(
                            &js_sys::Array::of1(&blob),
                            "generated_video.mp4",
                            &file_options,
                        )
                        .expect("Failed to create file");

                        let gloo_file = gloo::file::File::from(file);
                        let object_url = gloo::file::ObjectUrl::from(gloo_file.clone());
                        let video_url = String::from(&*object_url);

                        let file_with_url = FileWithUrl {
                            file: gloo_file,
                            url: object_url,
                        };

                        file_blob.set(Some(file_with_url));

                        if let Some(video) = video_ref.get() {
                            video.set_src(&video_url);
                        }

                        generation_error.set(None);
                    }
                    Err(e) => {
                        generation_error.set(Some(format!("Failed to get video binary: {}", e)));
                    }
                }
            } else {
                generation_error.set(Some(format!(
                    "Failed to fetch video: {}",
                    response.status()
                )));
            }
        }
        Err(e) => {
            generation_error.set(Some(format!("Failed to download video: {}", e)));
        }
    }
}

#[component]
pub fn UploadAiPostPage() -> impl IntoView {
    let trigger_upload = RwSignal::new_local(None::<UploadParams>);
    let uid = RwSignal::new_local(None);
    let upload_file_actual_progress = RwSignal::new(0.0f64);

    view! {
        <Title text="YRAL AI - Upload" />
        <div class="flex overflow-y-scroll flex-col gap-6 justify-center items-center px-5 pt-4 pb-12 text-white bg-black md:gap-8 md:px-8 md:pt-6 lg:gap-16 lg:px-12 min-h-dvh w-dvw">
            <div class="flex flex-col place-content-center w-full min-h-full lg:flex-row">
                <Show
                    when=move || { trigger_upload.with(|trigger_upload| trigger_upload.is_some()) }
                    fallback=move || {
                        view! {
                            <PreUploadAiView
                                trigger_upload=trigger_upload.write_only()
                                _uid=uid
                                _upload_file_actual_progress=upload_file_actual_progress.write_only()
                            />
                        }
                    }
                >
                    <VideoUploader
                        params=trigger_upload.get_untracked().unwrap()
                        uid=uid
                        upload_file_actual_progress=upload_file_actual_progress.read_only()
                    />
                </Show>
            </div>
        </div>
    }
}
