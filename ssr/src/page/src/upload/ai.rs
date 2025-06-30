use super::{
    ai_server::{fetch_video_bytes, generate_video_from_prompt, GenerateVideoRequest},
    validators::{description_validator, hashtags_validator},
    UploadParams,
};
use crate::scrolling_post_view::MuteIconOverlay;
use crate::upload::video_upload::VideoUploader;
use component::buttons::HighlightedButton;
use consts::UPLOAD_URL;
use leptos::{
    ev::durationchange,
    html::{Input, Textarea, Video},
    prelude::*,
};
use leptos_icons::*;
use leptos_meta::Title;
use leptos_use::use_event_listener;
use state::canisters::auth_state;
use utils::{
    event_streaming::events::{
        VideoUploadInitiated, VideoUploadUnsuccessful, VideoUploadUploadButtonClicked,
    },
    mixpanel::mixpanel_events::*,
    try_or_redirect_opt,
    web::FileWithUrl,
};

#[component]
fn PreUploadAiView(
    trigger_upload: WriteSignal<Option<UploadParams>, LocalStorage>,
    uid: RwSignal<Option<String>, LocalStorage>,
    upload_file_actual_progress: WriteSignal<f64>,
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
    let show_mute_icon = RwSignal::new(false);

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

    let upload_action: Action<(), _> = Action::new_local(move |_| {
        let captured_progress_signal = upload_file_actual_progress;
        async move {
            #[cfg(feature = "hydrate")]
            {
                use crate::upload::video_upload::upload_video_part;

                let message = try_or_redirect_opt!(upload_video_part(
                    UPLOAD_URL,
                    "file",
                    file_blob.get_untracked().unwrap().file.as_ref(),
                    captured_progress_signal,
                )
                .await
                .inspect_err(|e| {
                    VideoUploadUnsuccessful.send_event(ev_ctx, e.to_string(), 0, false, true);
                    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                        MixPanelEvent::track_video_upload_error_shown(
                            MixpanelVideoUploadFailureProps {
                                user_id: global.user_id,
                                visitor_id: global.visitor_id,
                                is_logged_in: global.is_logged_in,
                                canister_id: global.canister_id,
                                is_nsfw_enabled: global.is_nsfw_enabled,
                                error: e.to_string(),
                            },
                        );
                    }
                }));

                uid.set(message.data.and_then(|m| m.uid));
            }

            Some(())
        }
    });

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

            generation_error.set(None);
            polling_status.set("Generating video... This may take a few minutes.".to_string());

            let request_body = GenerateVideoRequest {
                prompt,
                user_id: uuid::Uuid::new_v4().to_string(),
                generate_audio: true,
                negative_prompt: String::new(),
            };

            match generate_video_from_prompt(request_body).await {
                Ok(result) => {
                    polling_status.set("Video generated! Loading...".to_string());

                    #[cfg(feature = "hydrate")]
                    load_video_from_url(result.video_url, file_blob, generation_error, video_ref)
                        .await;

                    #[cfg(not(feature = "hydrate"))]
                    {
                        generation_error.set(Some(
                            "Video generation is only supported in the browser".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    generation_error.set(Some(format!("Video generation failed: {}", e)));
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

    _ = use_event_listener(video_ref, durationchange, move |_| {
        let duration = video_ref
            .get_untracked()
            .map(|v| v.duration())
            .unwrap_or_default();
        let Some(_vid_file) = file_blob.get_untracked() else {
            return;
        };
        if duration <= 60.0 || duration.is_nan() {
            upload_action.dispatch(());
            return;
        }
        // Video is too long, handle error
        generation_error.set(Some(
            "Generated video is longer than 60 seconds".to_string(),
        ));
        file_blob.set(None);
        uid.set(None);
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
                            <HighlightedButton
                                on_click=move || { generate_action.dispatch(()); }
                                disabled=false
                                classes="w-full mx-auto py-[12px] px-[20px] rounded-xl bg-linear-to-r from-pink-300 to-pink-500 text-white font-light text-[17px] transition disabled:opacity-60 disabled:cursor-not-allowed".to_string()
                            >
                                {move || if generate_action.pending().get() { "Generating..." } else { "Generate Video" }}
                            </HighlightedButton>
                        </div>
                    }
                >
                    <div class="relative w-full h-full">
                        <video
                            node_ref=video_ref
                            class="w-full h-full object-contain rounded-lg cursor-pointer"
                            playsinline
                            muted=true
                            autoplay
                            loop
                            on:click=move |_| {
                                if let Some(video) = video_ref.get() {
                                    let new_muted = !video.muted();
                                    video.set_muted(new_muted);
                                    if new_muted {
                                        show_mute_icon.set(true);
                                    }
                                }
                            }
                            src=move || file_blob.with(|file| file.as_ref().map(|f| f.url.to_string()))
                        ></video>
                        <MuteIconOverlay show_mute_icon=show_mute_icon />
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
            <div class="flex overflow-y-auto flex-col gap-4 justify-between p-2 w-full h-auto rounded-2xl max-w-[627px] min-h-[400px] max-h-[90vh] lg:w-[627px] lg:h-[600px]">
                <h2 class="mb-2 font-light text-white text-[32px]">Upload AI Video</h2>
                <div class="flex flex-col gap-y-1">
                    <label for="caption-input" class="mb-1 font-light text-[20px] text-neutral-300">
                        Caption
                    </label>
                    <Show when=move || {
                        description_err.with(|description_err| !description_err.is_empty())
                    }>
                        <span class="text-sm text-red-500">{desc_err_memo()}</span>
                    </Show>
                    <textarea
                        id="caption-input"
                        node_ref=desc
                        on:input=move |ev| {
                            let desc = event_target_value(&ev);
                            description_err
                                .set(description_validator(desc).err().unwrap_or_default());
                        }
                        class="p-3 min-w-full rounded-lg border transition outline-none focus:border-pink-400 focus:ring-pink-400 bg-neutral-900 border-neutral-800 text-[15px] placeholder:text-neutral-500 placeholder:font-light"
                        rows=12
                        placeholder="Enter the caption here"
                        disabled=move || file_blob.with(|f| f.is_none())
                    ></textarea>
                </div>
                <div class="flex flex-col gap-y-1 mt-2">
                    <label for="hashtag-input" class="mb-1 font-light text-[20px] text-neutral-300">
                        Add Hashtag
                    </label>
                    <Show when=move || {
                        hashtags_err.with(|hashtags_err| !hashtags_err.is_empty())
                    }>
                        <span class="text-sm font-semibold text-red-500">
                            {hashtags_err_memo()}
                        </span>
                    </Show>
                    <input
                        id="hashtag-input"
                        node_ref=hashtag_inp
                        on:input=move |ev| {
                            let hts = event_target_value(&ev);
                            hashtag_on_input(hts);
                        }
                        class="p-3 rounded-lg border transition outline-none focus:border-pink-400 focus:ring-pink-400 bg-neutral-900 border-neutral-800 text-[15px] placeholder:text-neutral-500 placeholder:font-light"
                        type="text"
                        placeholder="Hit enter to add #hashtags"
                        disabled=move || file_blob.with(|f| f.is_none())
                    />
                </div>
                <div class="flex items-center gap-2">
                    <input
                        id="nsfw-checkbox"
                        node_ref=is_nsfw
                        type="checkbox"
                        class="w-4 h-4 text-pink-500 bg-gray-100 border-gray-300 rounded focus:ring-pink-400"
                        disabled=move || file_blob.with(|f| f.is_none())
                    />
                    <label for="nsfw-checkbox" class="text-neutral-300 text-sm">
                        This content is NSFW
                    </label>
                </div>
                {move || {
                    let disa = invalid_form.get() || file_blob.with(|f| f.is_none());
                    view! {
                        <HighlightedButton
                            on_click=move || on_submit()
                            disabled=disa
                            classes="w-full mx-auto py-[12px] px-[20px] rounded-xl bg-linear-to-r from-pink-300 to-pink-500 text-white font-light text-[17px] transition disabled:opacity-60 disabled:cursor-not-allowed"
                                .to_string()
                        >
                            "Upload"
                        </HighlightedButton>
                    }
                }}
            </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
async fn load_video_from_url(
    video_url: String,
    file_blob: RwSignal<Option<FileWithUrl>, LocalStorage>,
    generation_error: RwSignal<Option<String>>,
    _video_ref: NodeRef<Video>,
) {
    leptos::logging::log!("Attempting to load video from URL: {}", video_url);

    // Use server function to fetch video to avoid CORS issues
    match fetch_video_bytes(video_url).await {
        Ok(bytes) => {
            leptos::logging::log!("Received {} bytes from server", bytes.len());

            // Create a Uint8Array from the bytes
            let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
            uint8_array.copy_from(&bytes[..]);

            // Create blob with proper options
            let blob_parts = js_sys::Array::new();
            blob_parts.push(&uint8_array.into());

            let mut blob_options = web_sys::BlobPropertyBag::new();
            blob_options.type_("video/mp4");

            let blob =
                web_sys::Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_options)
                    .expect("Failed to create blob");

            leptos::logging::log!("Created blob with size: {}", blob.size());

            // Create a File from the Blob
            let file_options = web_sys::FilePropertyBag::new();
            file_options.set_type("video/mp4");
            let file = web_sys::File::new_with_blob_sequence_and_options(
                &js_sys::Array::of1(&blob),
                "generated_video.mp4",
                &file_options,
            )
            .expect("Failed to create file");

            let file_with_url = FileWithUrl::new(file.into());

            file_blob.set(Some(file_with_url));

            // if let Some(video) = video_ref.get() {
            //     video.set_src(&video_url);
            //     // Force a load to ensure video starts properly
            //     let _ = video.load();
            //     leptos::logging::log!("Set video src and called load()");
            // }

            generation_error.set(None);
        }
        Err(e) => {
            leptos::logging::log!("Error fetching video: {}", e);
            generation_error.set(Some(format!("Failed to fetch video: {}", e)));
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
                                uid=uid
                                upload_file_actual_progress=upload_file_actual_progress.write_only()
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
