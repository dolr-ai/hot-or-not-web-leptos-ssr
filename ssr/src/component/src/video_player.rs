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
        <label class="grid absolute top-0 left-0 grid-cols-1 justify-items-center items-center w-full h-full cursor-pointer z-3">
            <input
                on:change=move |_| AudioState::toggle_mute()
                type="checkbox"
                value=""
                class="sr-only"
            />
            <video
                node_ref=node_ref
                class="object-contain cursor-pointer h-dvh max-h-dvh transition-opacity duration-150"
                class:hidden=move || view_video_url().is_none()
                poster=view_bg_url
                src=view_video_url
                loop
                muted
                playsinline
                disablepictureinpicture
                disableremoteplayback
                preload="auto"
            ></video>
        </label>
    }
}
