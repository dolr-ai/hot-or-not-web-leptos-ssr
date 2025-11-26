use component::buttons::HighlightedButton;
use component::icons::sound_off_icon::SoundOffIcon;
use component::icons::sound_on_icon::SoundOnIcon;
use component::icons::volume_high_icon::VolumeHighIcon;
use component::icons::volume_mute_icon::VolumeMuteIcon;
use leptos::prelude::*;
use leptos_icons::*;
use state::audio_state::AudioState;
use yral_canisters_common::utils::posts::PostDetails;

use super::api::approve_video;

/// Simplified overlay for the approval page
/// Contains only: creator info, description, mute control, and approval button
#[component]
pub fn ApprovalOverlay(
    post: PostDetails,
    current_idx: RwSignal<usize>,
    #[prop(optional, into)] high_priority: bool,
) -> impl IntoView {
    let display_name = post.username_or_fallback();
    let profile_url = format!("/profile/{}/tokens", post.username_or_principal());
    let video_uid = post.uid.clone();

    let AudioState { muted, volume } = AudioState::get();

    // Approval action
    let approve_action = Action::new(move |video_uid: &String| {
        let video_uid = video_uid.clone();
        async move {
            match approve_video(video_uid.clone()).await {
                Ok(true) => {
                    leptos::logging::log!("Video {} approved successfully", video_uid);
                    true
                }
                Ok(false) => {
                    leptos::logging::warn!("Video {} approval returned false", video_uid);
                    false
                }
                Err(e) => {
                    leptos::logging::error!("Failed to approve video {}: {:?}", video_uid, e);
                    false
                }
            }
        }
    });

    // Auto-advance to next video after successful approval
    let advance_to_next = move || {
        current_idx.update(|idx| *idx += 1);
        // Scroll to next video
        if let Some(win) = leptos::web_sys::window() {
            // The scrolling container will handle snap scrolling
            let _ = win.scroll_by_with_x_and_y(0.0, win.inner_height().ok().and_then(|h| h.as_f64()).unwrap_or(800.0));
        }
    };

    // Effect to advance after successful approval
    Effect::new(move |_| {
        if let Some(result) = approve_action.value().get() {
            if result {
                advance_to_next();
            }
        }
    });

    let is_approving = approve_action.pending();
    let video_uid_for_button = video_uid.clone();

    view! {
        <MuteUnmuteControl muted volume />
        <div class="flex absolute bottom-0 left-0 flex-col flex-nowrap justify-between pt-5 pb-20 w-full h-full text-white bg-transparent pointer-events-none px-[16px] z-4 md:px-[16px]">
            // Top content - creator info
            <div class="flex flex-col w-full">
                <div class="flex flex-row justify-between items-center w-full pointer-events-auto">
                    <div class="flex flex-row gap-2 items-center p-2 w-9/12 rounded-s-full bg-linear-to-r from-black/25 via-80% via-black/10">
                        <div class="flex w-fit">
                            <a
                                href=profile_url.clone()
                                class="w-10 h-10 rounded-full border-2 md:w-12 md:h-12 overflow-clip border-primary-600"
                            >
                                <img
                                    class="object-cover w-full h-full"
                                    src=post.propic_url.clone()
                                    fetchpriority="low"
                                    loading={if high_priority { "eager" } else { "lazy" }}
                                />
                            </a>
                        </div>
                        <div class="flex flex-col justify-center min-w-0">
                            <div class="flex flex-row gap-1 items-center text-xs md:text-sm lg:text-base">
                                <span class="font-semibold truncate">
                                    <a href=profile_url>
                                        {display_name}
                                    </a>
                                </span>
                                <span class="font-semibold">"|"</span>
                                <span class="flex flex-row gap-1 items-center">
                                    <Icon
                                        attr:class="text-sm md:text-base lg:text-lg"
                                        icon=icondata::AiEyeOutlined
                                    />
                                    {post.views}
                                </span>
                            </div>
                            <ExpandableText description=post.description.clone() />
                        </div>
                    </div>
                </div>
            </div>

            // Bottom content - Approval button
            <div class="flex flex-col items-center w-full pointer-events-auto mb-4">
                <HighlightedButton
                    classes="w-full max-w-xs".to_string()
                    alt_style=false
                    disabled=is_approving.get()
                    on_click=move || {
                        approve_action.dispatch(video_uid_for_button.clone());
                    }
                >
                    {move || if is_approving.get() { "Approving..." } else { "Approve" }}
                </HighlightedButton>
            </div>
        </div>
    }.into_any()
}

#[component]
fn ExpandableText(description: String) -> impl IntoView {
    let truncated = RwSignal::new(true);

    view! {
        <span
            class="w-full text-xs md:text-sm lg:text-base"
            class:truncate=truncated
            on:click=move |_| truncated.update(|e| *e = !*e)
        >
            {description}
        </span>
    }
}

#[component]
pub fn MuteUnmuteControl(muted: RwSignal<bool>, volume: RwSignal<f64>) -> impl IntoView {
    let volume_ = Signal::derive(move || if muted.get() { 0.0 } else { volume.get() });
    view! {
        <button
            tabindex="0"
            class="z-10 select-none rounded-r-lg bg-black/25 py-2 px-3 cursor-pointer text-sm font-medium text-white items-center gap-1
            pointer-coarse:flex pointer-fine:hidden absolute top-[7rem] left-0 safari:transition-none
            active:translate-x-0 -translate-x-2/3 focus:delay-[3.5s] active:focus:delay-0 transition-all duration-100"
            on:click=move |_| {
                let is_muted = muted.get_untracked();
                muted.set(!is_muted);
                volume.set(if is_muted { 1.0 } else { 0.0 });
            }
        >
            <div class="w-[10ch] text-center">{move || if muted.get() { "Unmute" } else { "Mute" }}</div>
            <Show
                when=move || muted.get()
                fallback=|| view! { <SoundOnIcon classes="w-4 h-4".to_string() /> }
            >
                <SoundOffIcon classes="w-4 h-4".to_string() />
            </Show>
        </button>
        <div class="z-10 select-none rounded-full bg-black/35 p-2.5 cursor-pointer text-sm font-medium text-white items-center gap-3
            pointer-coarse:hidden pointer-fine:flex absolute top-[7rem] left-4
            size-11 hover:size-auto group">
            <button
                class="shrink-0"
                on:click=move |_| {
                    let is_muted = muted.get_untracked();
                    muted.set(!is_muted);
                    volume.set(if is_muted { 1.0 } else { 0.0 });
                }
            >
                <Show
                    when=move || muted.get() || volume.get() == 0.0
                    fallback=|| view! {<VolumeHighIcon classes="w-6 h-6".to_string() /> }
                >
                    <VolumeMuteIcon classes="w-6 h-6".to_string() />
                </Show>
            </button>
            <div class="overflow-hidden max-w-0 group-hover:max-w-[500px] transition-all duration-1000">
                <div class="relative w-fit -translate-y-0.5">
                    <div class="absolute inset-0 flex items-center pointer-events-none">
                        <div
                            style:width=move || format!("calc({}% - 0.25%)", volume_.try_get().unwrap_or(0.0) * 100.0)
                            class="bg-white w-full h-1.5 translate-y-[0.15rem] rounded-full"
                        >
                        </div>
                    </div>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        prop:value=volume_
                        class="w-24 cursor-pointer appearance-none bg-white/50 h-1.5 rounded-full
                        [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:bg-white [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:rounded-full"
                        on:input=move |ev| {
                            let target = event_target::<web_sys::HtmlInputElement>(&ev);
                            if let Ok(val) = target.value().parse::<f64>() {
                                volume.set(val);
                                muted.set(val == 0.0);
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
