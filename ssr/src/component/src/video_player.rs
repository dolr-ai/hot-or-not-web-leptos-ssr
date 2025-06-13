use leptos::html::Video;
use leptos::prelude::*;

use state::audio_state::AudioState;

#[component]
pub fn VideoPlayer(
    #[prop(optional)] node_ref: NodeRef<Video>,
    #[prop(into)] view_bg_url: Signal<Option<String>>,
    #[prop(into)] view_video_url: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        <label class="h-full w-full absolute top-0 left-0 grid grid-cols-1 justify-items-center items-center cursor-pointer z-3">
            // <input
            //     on:change=move |_| AudioState::toggle_mute()
            //     type="checkbox"
            //     value=""
            //     class="sr-only"
            // />
            <video
                node_ref=node_ref
                class="object-contain h-dvh max-h-dvh cursor-pointer"
                poster=view_bg_url
                src=view_video_url
                loop
                // autoplay
                muted
                playsinline
                disablepictureinpicture
                disableremoteplayback
                preload="none"
                on:click=move |_| {
                    AudioState::toggle_mute();

                    if let Some(vid) = node_ref.get() {
                        if vid.paused() {
                            let play_promise = vid.play();
                            if let Ok(promise) = play_promise {
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                });
                            }
                        }
                    }
                }
            >
            </video>
        </label>
    }
}
