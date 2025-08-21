use super::UploadParams;
use auth::delegate_short_lived_identity;
use component::buttons::HighlightedLinkButton;
use component::modal::Modal;
use component::notification_nudge::NotificationNudge;
use consts::UPLOAD_URL;
use futures::channel::oneshot;
use gloo::net::http::Request;
use leptos::web_sys::{Blob, FormData, ProgressEvent};
use leptos::{
    ev::durationchange,
    html::{Input, Video},
    prelude::*,
};
use leptos_icons::*;
use leptos_use::use_event_listener;
use serde::{Deserialize, Serialize};
use serde_json::json;
use state::canisters::auth_state;
use std::cell::RefCell;
use std::rc::Rc;
use utils::mixpanel::mixpanel_events::*;
use utils::{
    event_streaming::events::{VideoUploadUnsuccessful, VideoUploadVideoSelected},
    try_or_redirect_opt,
    web::FileWithUrl,
};
use wasm_bindgen::{closure::Closure, JsCast};

#[component]
pub fn DropBox() -> impl IntoView {
    view! {
        <div class="flex flex-col justify-center justify-self-center items-center w-full rounded-lg border-2 border-gray-600 border-dashed cursor-pointer hover:bg-gray-600 aspect-3/4 lg:aspect-5/4">
            <Icon attr:class="w-10 h-10 mb-4 text-gray-400" icon=icondata::BiCloudUploadRegular />
            <p class="mx-2 mb-2 text-sm text-center text-gray-400">
                <span class="font-semibold">Click to upload</span>
                or drag and drop
            </p>
            <p class="text-xs text-gray-400">Video File (Max 60s)</p>
        </div>
    }
}

#[component]
pub fn PreVideoUpload(
    file_blob: RwSignal<Option<FileWithUrl>>,
    uid: RwSignal<Option<String>>,
    upload_file_actual_progress: WriteSignal<f64>,
) -> impl IntoView {
    let file_ref = NodeRef::<Input>::new();
    let file = RwSignal::new(None::<FileWithUrl>);
    let video_ref = NodeRef::<Video>::new();
    let modal_show = RwSignal::new(false);
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    let file_upload_clicked = Action::new(move |_: &()| {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_select_file_clicked(global);
        }
        async {}
    });

    let file_selection_success = Action::new(move |_: &()| {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_file_selection_success(global, "video".into());
        }
        async {}
    });

    #[cfg(feature = "hydrate")]
    {
        use leptos::ev::change;
        _ = use_event_listener(file_ref, change, move |ev| {
            use wasm_bindgen::JsCast;
            use web_sys::HtmlInputElement;
            ev.target().and_then(move |target| {
                let input: &HtmlInputElement = target.dyn_ref()?;
                let inp_file = input.files()?.get(0)?;
                file.set(Some(FileWithUrl::new(inp_file.into())));

                VideoUploadVideoSelected.send_event(ev_ctx);
                file_selection_success.dispatch(());
                Some(())
            });
        });
    }

    let upload_action: Action<(), _> = Action::new_unsync(move |_| {
        let captured_progress_signal = upload_file_actual_progress;
        async move {
            #[cfg(feature = "hydrate")]
            {
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
                        MixPanelEvent::track_video_upload_error_shown(global, e.to_string());
                    }
                }));

                uid.set(message.data.and_then(|m| m.uid));
            }

            Some(())
        }
    });

    _ = use_event_listener(video_ref, durationchange, move |_| {
        let duration = video_ref
            .get_untracked()
            .map(|v| v.duration())
            .unwrap_or_default();
        let Some(vid_file) = file.get_untracked() else {
            modal_show.set(false);
            return;
        };
        if duration <= 60.0 || duration.is_nan() {
            modal_show.set(false);
            file_blob.set(Some(vid_file));
            upload_action.dispatch(());
            return;
        }

        modal_show.set(true);
        file.set(None);
        uid.set(None);
        file_blob.set(None);
        if let Some(f) = file_ref.get_untracked() {
            f.set_value("");
        }
    });

    view! {
        <label
            for="dropzone-file"
            class="flex flex-col justify-center items-center p-0 rounded-2xl border-2 border-dashed cursor-pointer select-none sm:w-full sm:h-auto w-[358px] h-[300px] bg-neutral-950 border-neutral-600 sm:min-h-[380px] sm:max-h-[70vh] lg:w-[627px] lg:h-[600px]"
        >
            <Show when=move || { file.with(|file| file.is_none()) }>
                <div class="flex flex-col flex-1 gap-6 justify-center items-center w-full h-full">
                    <div class="font-semibold leading-tight text-center text-white text-[16px]">
                        Upload a video to share with the world!
                    </div>
                    <div class="leading-tight text-center text-neutral-400 text-[13px]">
                        Drag & Drop or select video file ( Max 60s )
                    </div>
                    <span class="inline-block py-2 px-6 font-medium text-pink-300 bg-transparent rounded-lg border border-pink-300 transition-colors duration-150 cursor-pointer select-none text-[15px]">
                        Select File
                    </span>
                </div>
            </Show>
            <Show when=move || { file.with(|file| file.is_some()) }>
                <video
                    node_ref=video_ref
                    class="object-contain p-2 w-full h-full bg-black rounded-xl"
                    playsinline
                    muted
                    autoplay
                    loop
                    oncanplay="this.muted=true"
                    src=move || file.with(|file| file.as_ref().map(|f| f.url()))
                ></video>
            </Show>
            <input
                on:click=move |_| {file_upload_clicked.dispatch(());}
                id="dropzone-file"
                node_ref=file_ref
                type="file"
                accept="video/*"
                class="hidden w-0 h-0"
            />
        </label>
        <Show when=move || modal_show.get()>
            <Modal show=modal_show>
                <span class="flex flex-col justify-center items-center py-10 w-full h-full text-lg text-center text-white md:text-xl">
                    Please ensure that the video is shorter than 60 seconds
                </span>
            </Modal>
        </Show>
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Message {
    pub message: Option<String>,
    pub success: Option<bool>,
    pub data: Option<Data>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Data {
    #[serde(rename = "scheduledDeletion")]
    pub scheduled_deletion: Option<String>,
    pub uid: Option<String>,
    #[serde(rename = "uploadURL")]
    pub upload_url: Option<String>,
    pub watermark: Option<Watermark>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Watermark {
    pub created: Option<String>,
    #[serde(rename = "downloadedFrom")]
    pub downloaded_from: Option<String>,
    pub height: Option<f64>,
    pub name: Option<String>,
    pub opacity: Option<f64>,
    pub padding: Option<f64>,
    pub position: Option<String>,
    pub scale: Option<f64>,
    pub size: Option<f64>,
    pub uid: Option<String>,
}
#[allow(dead_code)]
#[derive(Serialize, Debug)]
pub struct VideoMetadata {
    pub title: String,
    pub description: String,
    pub tags: String,
}

#[derive(Serialize, Debug)]
pub struct SerializablePostDetailsFromFrontend {
    pub is_nsfw: bool,
    pub hashtags: Vec<String>,
    pub description: String,
    pub video_uid: String,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
}

#[cfg(feature = "hydrate")]
pub async fn upload_video_part(
    upload_base_url: &str,
    form_field_name: &str,
    file_blob: &Blob,
    progress_signal: WriteSignal<f64>,
) -> Result<Message, ServerFnError> {
    let get_url_endpoint = format!("{upload_base_url}/get_upload_url_v2");
    let response = Request::get(&get_url_endpoint).send().await?;
    if !response.ok() {
        return Err(ServerFnError::new(format!(
            "Failed to get upload URL: status {}",
            response.status()
        )));
    }
    let response_text = response.text().await?;
    let upload_message: Message = serde_json::from_str(&response_text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse upload URL response: {e}")))?;

    let upload_url = upload_message
        .data
        .clone()
        .and_then(|d| d.upload_url)
        .ok_or_else(|| ServerFnError::new("Upload URL not found in response".to_string()))?;

    let form = FormData::new().map_err(|js_value| {
        ServerFnError::new(format!("Failed to create FormData: {js_value:?}"))
    })?;
    form.append_with_blob(form_field_name, file_blob)
        .map_err(|js_value| {
            ServerFnError::new(format!("Failed to append blob to FormData: {js_value:?}"))
        })?;

    let (tx, rx) = oneshot::channel();
    let xhr = web_sys::XmlHttpRequest::new()
        .map_err(|e| ServerFnError::new(format!("Failed to create XHR: {e:?}")))?;
    let xhr_upload = xhr
        .upload()
        .map_err(|e| ServerFnError::new(format!("Failed to get XHR upload: {e:?}")))?;

    let sender_rc = Rc::new(RefCell::new(Some(tx)));

    let progress_signal_clone = progress_signal;
    let on_progress_callback = Closure::wrap(Box::new(move |event: ProgressEvent| {
        if event.length_computable() {
            let progress = event.loaded() / event.total();
            progress_signal_clone.set(progress);
        }
    }) as Box<dyn FnMut(_)>);
    xhr_upload.set_onprogress(Some(on_progress_callback.as_ref().unchecked_ref()));
    on_progress_callback.forget();

    let sender_onload_rc = sender_rc.clone();
    let progress_signal_onload = progress_signal;
    let xhr_clone_onload = xhr.clone();
    let on_load_callback = Closure::wrap(Box::new(move || {
        if let Some(sender) = sender_onload_rc.borrow_mut().take() {
            match xhr_clone_onload.status() {
                Ok(status) if (200..300).contains(&status) => {
                    progress_signal_onload.set(1.0);
                    let _ = sender.send(Ok(()));
                }
                Ok(status) => {
                    let err_msg = format!(
                        "Upload XHR failed: status {} {}",
                        status,
                        xhr_clone_onload.status_text().unwrap_or_default()
                    );
                    let _ = sender.send(Err(ServerFnError::new(err_msg)));
                }
                Err(_) => {
                    let _ = sender.send(Err(ServerFnError::new(
                        "Upload XHR failed to get status".to_string(),
                    )));
                }
            }
        }
    }) as Box<dyn FnMut()>);
    xhr.set_onload(Some(on_load_callback.as_ref().unchecked_ref()));
    on_load_callback.forget();

    let sender_onerror_rc = sender_rc.clone();
    let xhr_clone_onerror = xhr.clone();
    let on_error_callback = Closure::wrap(Box::new(move || {
        if let Some(sender) = sender_onerror_rc.borrow_mut().take() {
            let status = xhr_clone_onerror.status().unwrap_or(0);
            let status_text = xhr_clone_onerror.status_text().unwrap_or_default();
            let err_msg =
                format!("Upload XHR network error. Status: {status}, Text: {status_text}");
            let _ = sender.send(Err(ServerFnError::new(err_msg)));
        }
    }) as Box<dyn FnMut()>);
    xhr.set_onerror(Some(on_error_callback.as_ref().unchecked_ref()));
    on_error_callback.forget();

    let sender_ontimeout_rc = sender_rc.clone();
    let on_timeout_callback = Closure::wrap(Box::new(move || {
        if let Some(sender) = sender_ontimeout_rc.borrow_mut().take() {
            let _ = sender.send(Err(ServerFnError::new("Upload XHR timeout".to_string())));
        }
    }) as Box<dyn FnMut()>);
    xhr.set_ontimeout(Some(on_timeout_callback.as_ref().unchecked_ref()));
    on_timeout_callback.forget();

    xhr.open("POST", &upload_url)
        .map_err(|e| ServerFnError::new(format!("XHR open failed: {e:?}")))?;

    xhr.send_with_opt_form_data(Some(&form))
        .map_err(|e| ServerFnError::new(format!("XHR send failed: {e:?}")))?;

    match rx.await {
        Ok(Ok(())) => Ok(upload_message),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(ServerFnError::new(
            "XHR future cancelled or sender dropped".to_string(),
        )),
    }
}

#[component]
pub fn VideoUploader(
    params: UploadParams,
    uid: RwSignal<Option<String>>,
    upload_file_actual_progress: ReadSignal<f64>,
) -> impl IntoView {
    let file_blob = StoredValue::new(params.file_blob);
    let hashtags = params.hashtags;
    let description = params.description;

    let published: RwSignal<bool> = RwSignal::new(false);

    let is_nsfw = params.is_nsfw;
    let enable_hot_or_not = params.enable_hot_or_not;

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();
    let ev_ctx = auth.event_ctx();

    let notification_nudge = RwSignal::new(false);

    let publish_action: Action<_, _> = Action::new_unsync(move |&()| {
        leptos::logging::log!("Publish action triggered");
        let hashtags = hashtags.clone();
        let hashtags_len = hashtags.len();
        let description = description.clone();
        leptos::logging::log!("Publish action called");

        async move {
            let uid_value = uid.get_untracked()?;

            let canisters = auth.auth_cans().await.ok()?;
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
                    let global = MixpanelGlobalProps::try_get(&canisters, is_logged_in);
                    MixPanelEvent::track_video_upload_success(
                        global,
                        uid_value.clone(),
                        global_constants::CREATOR_COMMISSION_PERCENT,
                        true,
                        MixpanelPostGameType::HotOrNot,
                        Some("upload_video".to_string()),
                        "".to_string(), // Regular uploads don't use tokens
                    );
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
                    src=move || file_blob.with_value(|f| f.url())
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

// post as in after not the content post
#[component]
pub fn PostUploadScreen() -> impl IntoView {
    view! {
        <div
            style="background: radial-gradient(circle, rgba(0,0,0,0) 0%, rgba(0,0,0,0) 75%, rgba(50,0,28,0.5) 100%);"
            class="flex fixed top-0 right-0 bottom-0 left-0 z-50 justify-center items-center w-screen h-screen"
        >
            <img
                alt="bg"
                src="/img/airdrop/bg.webp"
                class="object-cover absolute inset-0 w-full h-full z-25 fade-in"
            />
            <div class="flex z-50 flex-col items-center text-white">
                <img src="/img/common/coins/sucess-coin.png" width=170 class="mb-6 z-300" />

                <h1 class="mb-2 text-lg font-semibold">Video uploaded sucessfully</h1>

                <p class="px-4 mb-8 text-center">
                    "We're processing your video. It'll be in 'Your Videos' under My Profile soon. Happy scrolling!"
                </p>
                <HighlightedLinkButton
                    alt_style=false
                    disabled=false
                    classes="max-w-96 w-full mx-auto py-[12px] px-[20px]".to_string()
                    href="/".to_string()
                >
                    Done
                </HighlightedLinkButton>
            </div>
        </div>
    }
}
