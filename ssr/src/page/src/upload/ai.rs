use super::{
    ai_server::{fetch_video_bytes, generate_video_from_prompt, GenerateVideoRequest},
    validators::{description_validator, hashtags_validator},
    UploadParams,
};
// Remove the import of MuteIconOverlay from scrolling_post_view
use crate::upload::video_upload::{
    PostUploadScreen, SerializablePostDetailsFromFrontend, VideoMetadata,
};
use auth::delegate_short_lived_identity;
use component::{buttons::HighlightedButton, notification_nudge::NotificationNudge};
use consts::UPLOAD_URL;
use leptos::reactive::send_wrapper_ext::SendOption;
use leptos::{
    ev::durationchange,
    html::{Input, Textarea, Video},
    prelude::*,
};
use leptos_icons::*;
use leptos_meta::Title;
use leptos_use::use_event_listener;
use serde_json::json;
use state::canisters::{auth_state, unauth_canisters};
use utils::{
    event_streaming::events::{
        VideoUploadInitiated, VideoUploadSuccessful, VideoUploadUnsuccessful,
        VideoUploadUploadButtonClicked,
    },
    mixpanel::mixpanel_events::*,
    try_or_redirect_opt,
    web::FileWithUrl,
};

#[component]
fn AiMuteIconOverlay(show_mute_icon: RwSignal<bool>) -> impl IntoView {
    view! {
        <Show when=show_mute_icon>
            <button
                class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-20 cursor-pointer pointer-events-none"
                on:click=move |_| {
                    show_mute_icon.set(false);
                }
            >
                <Icon
                    attr:class="text-white/80 animate-ping text-4xl"
                    icon=icondata::BiVolumeMuteSolid
                />
            </button>
        </Show>
    }
}

#[component]
fn PreUploadAiView(
    trigger_upload: WriteSignal<SendOption<UploadParams>>,
    uid: RwSignal<Option<String>>,
    upload_file_actual_progress: WriteSignal<f64>,
) -> impl IntoView {
    let description_err = RwSignal::new(String::new());
    let desc_err_memo = Memo::new(move |_| description_err());
    let hashtags = RwSignal::new(Vec::new());
    let hashtags_err = RwSignal::new(String::new());
    let hashtags_err_memo = Memo::new(move |_| hashtags_err());
    let file_blob = RwSignal::new(SendOption::<FileWithUrl>::new_local(None));
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

    let upload_action: Action<(), _> = Action::new_unsync(move |_| {
        async move {
            #[cfg(feature = "hydrate")]
            {
                use crate::upload::video_upload::upload_video_part;

                // Clone signals for use in spawn_local
                let uid_signal = uid;
                let file_blob_signal = file_blob;
                let captured_progress_signal = upload_file_actual_progress;

                let file_data = file_blob_signal.get_untracked();
                if let Some(file_with_url) = file_data.take() {
                    let message = upload_video_part(
                        UPLOAD_URL,
                        "file",
                        file_with_url.file.as_ref(),
                        captured_progress_signal,
                    )
                    .await
                    .unwrap();

                    uid_signal.set(message.data.and_then(|m| m.uid));
                };
            }

            Some(())
        }
    });

    let generate_action: Action<(), _> = Action::new_unsync(move |_| async move {
        #[cfg(feature = "hydrate")]
        {
            leptos::logging::log!("Generating video from prompt...");

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
                    load_video_from_url(result.video_url, file_blob, generation_error, video_ref)
                        .await;
                }
                Err(e) => {
                    generation_error.set(Some(format!("Video generation failed: {}", e)));
                }
            }
        }

        #[cfg(not(feature = "hydrate"))]
        {
            use leptos::task::spawn_local;

            spawn_local(async move {
                generation_error.set(Some(
                    "Video generation is only supported in the browser".to_string(),
                ));
            });
        }
    });

    let on_submit = move || {
        VideoUploadUploadButtonClicked.send_event(ev_ctx, hashtag_inp, is_nsfw, NodeRef::new());

        let description = desc.get_untracked().unwrap().value();
        let hashtags = hashtags.get_untracked();
        let Some(file_blob) = file_blob.get_untracked().as_ref().cloned() else {
            return;
        };
        trigger_upload.set(SendOption::new_local(Some(UploadParams {
            file_blob,
            hashtags,
            description,
            enable_hot_or_not: false,
            is_nsfw: is_nsfw
                .get_untracked()
                .map(|v| v.checked())
                .unwrap_or_default(),
        })));
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
        let Some(_vid_file) = file_blob.get_untracked().as_ref() else {
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
        file_blob.set(SendOption::new_local(None));
        uid.set(None);
    });

    view! {
        <div class="flex flex-col gap-4 justify-center items-center p-0 mx-auto w-full min-h-screen bg-transparent lg:flex-row lg:gap-20">
            <div class="flex flex-col justify-center items-center px-2 mx-4 mt-4 mb-4 text-center rounded-2xl sm:px-4 sm:mx-6 sm:w-full sm:h-auto lg:overflow-y-auto lg:px-0 lg:mx-0 w-[358px] h-[300px] sm:min-h-[380px] sm:max-h-[70vh] lg:w-[627px] lg:h-[600px]">
                <Show
                    when=move || file_blob.with(|f| (**f).is_some())
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
                            src=move || file_blob.with(|file| (**file).as_ref().map(|f| f.url.to_string()))
                        ></video>
                        <AiMuteIconOverlay show_mute_icon=show_mute_icon />
                        <button
                            on:click=move |_| {
                                file_blob.set(SendOption::new_local(None));
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
                        disabled=move || file_blob.with(|f| (**f).is_none())
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
                        disabled=move || file_blob.with(|f| (**f).is_none())
                    />
                </div>
                <div class="flex items-center gap-2">
                    <input
                        id="nsfw-checkbox"
                        node_ref=is_nsfw
                        type="checkbox"
                        class="w-4 h-4 text-pink-500 bg-gray-100 border-gray-300 rounded focus:ring-pink-400"
                        disabled=move || file_blob.with(|f| (**f).is_none())
                    />
                    <label for="nsfw-checkbox" class="text-neutral-300 text-sm">
                        This content is NSFW
                    </label>
                </div>
                {move || {
                    let disa = invalid_form.get() || file_blob.with(|f| (**f).is_none());
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
    file_blob: RwSignal<SendOption<FileWithUrl>>,
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

            file_blob.set(SendOption::new_local(Some(file_with_url)));

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
    let trigger_upload = RwSignal::new(SendOption::<UploadParams>::new_local(None));
    let uid = RwSignal::new(None);
    let upload_file_actual_progress = RwSignal::new(0.0f64);

    view! {
        <Title text="YRAL AI - Upload" />
        <div class="flex overflow-y-scroll flex-col gap-6 justify-center items-center px-5 pt-4 pb-12 text-white bg-black md:gap-8 md:px-8 md:pt-6 lg:gap-16 lg:px-12 min-h-dvh w-dvw">
            <div class="flex flex-col place-content-center w-full min-h-full lg:flex-row">
                <Show
                    when=move || { trigger_upload.with(|trigger_upload| (**trigger_upload).is_some()) }
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
                    <VideoAiUploader
                        params=trigger_upload.get_untracked().as_ref().unwrap().clone()
                        uid=uid
                        upload_file_actual_progress=upload_file_actual_progress.read_only()
                    />
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn VideoAiUploader(
    params: UploadParams,
    uid: RwSignal<Option<String>>,
    upload_file_actual_progress: ReadSignal<f64>,
) -> impl IntoView {
    let file_blob = params.file_blob;
    let hashtags = params.hashtags;
    let description = params.description;

    let published = RwSignal::new(false);
    let video_url = StoredValue::new_local(file_blob.url);

    let is_nsfw = params.is_nsfw;
    let enable_hot_or_not = params.enable_hot_or_not;

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();
    let ev_ctx = auth.event_ctx();

    let notification_nudge = RwSignal::new(false);

    let publish_action: Action<_, _> = Action::new_unsync(move |&()| {
        leptos::logging::log!("Publish action triggered");
        let unauth_cans = unauth_canisters();
        let hashtags = hashtags.clone();
        let hashtags_len = hashtags.len();
        let description = description.clone();
        leptos::logging::log!("Publish action called");

        async move {
            let uid_value = uid.get_untracked()?;

            let canisters = auth.auth_cans(unauth_cans).await.ok()?;
            let id = canisters.identity();
            let delegated_identity = delegate_short_lived_identity(id);
            let res: std::result::Result<reqwest::Response, ServerFnError> = {
                let client = reqwest::Client::new();
                notification_nudge.set(true);
                let req = client
                    .post(format!("{UPLOAD_URL}/update_metadata"))
                    .json(&json!({
                        "video_uid": uid,
                        "delegated_identity_wire": delegated_identity,
                        "meta": VideoMetadata{
                            title: description.clone(),
                            description: description.clone(),
                            tags: hashtags.join(",")
                        },
                        "post_details": SerializablePostDetailsFromFrontend{
                            is_nsfw,
                            hashtags,
                            description,
                            video_uid: uid_value.clone(),
                            creator_consent_for_inclusion_in_hot_or_not: enable_hot_or_not,
                        }
                    }));

                req.send()
                    .await
                    .map_err(|e| ServerFnError::new(e.to_string()))
            };

            match res {
                Ok(_) => {
                    let is_logged_in = is_connected.get_untracked();
                    published.set(true)
                }
                Err(_) => {
                    let e = res.as_ref().err().unwrap().to_string();
                    VideoUploadUnsuccessful.send_event(
                        ev_ctx,
                        e,
                        hashtags_len,
                        is_nsfw,
                        enable_hot_or_not,
                    );
                }
            }
            try_or_redirect_opt!(res);

            VideoUploadSuccessful.send_event(
                ev_ctx,
                uid_value.clone(),
                hashtags_len,
                is_nsfw,
                enable_hot_or_not,
                0,
            );

            Some(())
        }
    });

    Effect::new(move |prev_tracked_uid_val: Option<Option<String>>| {
        let current_uid_val = uid.get();
        let prev_uid_from_last_run: Option<String> = prev_tracked_uid_val.flatten();
        if current_uid_val.is_some()
            && (prev_uid_from_last_run.is_none() || prev_uid_from_last_run != current_uid_val)
            && !publish_action.pending().get()
            && !published.get()
        {
            publish_action.dispatch(());
        }
        current_uid_val
    });

    let video_uploaded_base_width = 200.0 / 3.0;
    let metadata_publish_total_width = 100.0 / 3.0;

    view! {
        <div class="flex flex-col-reverse gap-4 justify-center items-center p-0 mx-auto w-full min-h-screen bg-transparent lg:flex-row lg:gap-20">
            <NotificationNudge pop_up=notification_nudge />
            <div class="flex flex-col justify-center items-center px-4 mt-0 mb-0 w-full h-auto text-center rounded-2xl sm:px-6 sm:mt-0 sm:mb-0 lg:overflow-y-auto lg:px-0 min-h-[200px] max-h-[60vh] sm:min-h-[300px] sm:max-h-[70vh] lg:w-[627px] lg:h-[600px] lg:min-h-[600px] lg:max-h-[600px]">
                <video
                    class="object-contain p-2 w-full h-full bg-black rounded-xl"
                    playsinline
                    muted
                    autoplay
                    loop
                    oncanplay="this.muted=true"
                    src=move || video_url.get_value().to_string()
                ></video>
            </div>
            <div class="flex overflow-y-auto flex-col gap-4 justify-center p-2 w-full h-auto rounded-2xl max-w-[627px] min-h-[400px] max-h-[90vh] lg:w-[627px] lg:h-[600px]">
                <h2 class="mb-2 font-light text-white text-[32px]">Uploading Video</h2>
                <div class="flex flex-col gap-y-1">
                    <p>
                        This may take a moment. Feel free to explore more videos on the home page while you wait!
                    </p>
                </div>
                <div class="mt-2 w-full h-2.5 rounded-full bg-neutral-800">
                    <div
                        class="h-2.5 rounded-full duration-500 ease-in-out bg-linear-to-r from-[#EC55A7] to-[#E2017B] transition-width"
                        style:width=move || {
                            if published.get() {
                                "100%".to_string()
                            } else if publish_action.pending().get() {
                                format!(
                                    "{:.2}%",
                                    video_uploaded_base_width + metadata_publish_total_width * 0.7,
                                )
                            } else if uid.with(|u| u.is_some()) {
                                format!("{video_uploaded_base_width:.2}%")
                            } else {
                                format!(
                                    "{:.2}%",
                                    upload_file_actual_progress.get() * video_uploaded_base_width,
                                )
                            }
                        }
                    ></div>
                </div>
                <p class="mt-1 text-sm text-center text-gray-400">
                    {move || {
                        if published.get() {
                            "Upload complete!".to_string()
                        } else if publish_action.pending().get() {
                            "Processing video metadata...".to_string()
                        } else if uid.with(|u| u.is_none()) {
                            "Uploading video file...".to_string()
                        } else if uid.with(|u| u.is_some()) && !publish_action.pending().get()
                            && !published.get()
                        {
                            "Video file uploaded. Waiting to publish metadata...".to_string()
                        } else {
                            "Waiting to upload...".to_string()
                        }
                    }}
                </p>
            </div>
        </div>
        <Show when=published>
            <PostUploadScreen />
        </Show>
    }.into_any()
}
