use codee::string::FromToStringCodec;
use component::buttons::HighlightedButton;
use component::icons::sound_off_icon::SoundOffIcon;
use component::icons::sound_on_icon::SoundOnIcon;
use component::icons::volume_high_icon::VolumeHighIcon;
use component::icons::volume_mute_icon::VolumeMuteIcon;
use component::modal::Modal;

use consts::{NSFW_ENABLED_COOKIE, WALLET_BALANCE_STORE_KEY};
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_location;
use leptos_use::storage::use_local_storage;
use leptos_use::use_window;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use state::audio_state::AudioState;
use state::canisters::auth_state;
use utils::host::show_nsfw_content;

use utils::mixpanel::mixpanel_events::*;
use yral_canisters_common::utils::posts::PostDetails;

use crate::wallet::airdrop::sats_airdrop::claim_sats_airdrop;
use crate::wallet::airdrop::SatsAirdropPopup;
use leptos::prelude::ServerFnError;

#[component]
pub fn VideoDetailsOverlay(
    post: PostDetails,
    #[prop(optional, into)] high_priority: bool,
) -> impl IntoView {
    // No need for local context - using global context from App

    let show_nsfw_permission = RwSignal::new(false);
    let base_url = || {
        use_window()
            .as_ref()
            .and_then(|w| w.location().origin().ok())
    };
    let post_clone = post.clone();
    let post_id = post.post_id.clone();
    let video_url = Signal::derive(move || {
        base_url()
            .map(|b| format!("{b}/hot-or-not/{}/{}", post_clone.canister_id, post_id))
            .unwrap_or_default()
    });

    let display_name = post.username_or_fallback();

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    let track_video_id_for_impressions = post.uid.clone();
    let post_clone = post.clone();
    Effect::new(move |_| {
        // To trigger the effect on initial render
        let _ = use_location().pathname.get();
        let track_video_id_for_impressions = track_video_id_for_impressions.clone();
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            if Some(video_url()) == window().location().href().ok() {
                MixPanelEvent::track_video_impression(
                    global,
                    track_video_id_for_impressions,
                    post_clone.poster_principal.to_text(),
                    MixpanelPostGameType::HotOrNot,
                    post_clone.likes,
                    post_clone.views,
                    post_clone.is_nsfw,
                    true,
                );
            }
        }
    });

    let profile_url = format!("/profile/{}/tokens", post.username_or_principal());

    let profile_click_video_id = post.uid.clone();
    let post_clone = post.clone();

    let (nsfw_enabled, set_nsfw_enabled) = use_cookie_with_options::<bool, FromToStringCodec>(
        NSFW_ENABLED_COOKIE,
        UseCookieOptions::default()
            .path("/")
            .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
            .same_site(leptos_use::SameSite::Lax),
    );

    let click_nsfw = Action::new(move |()| {
        let video_id = post_clone.uid.clone();
        let post_clone = post_clone.clone();
        async move {
            if show_nsfw_content() {
                return;
            }

            if !nsfw_enabled().unwrap_or(false) && !show_nsfw_permission() {
                show_nsfw_permission.set(true);
                if let Some(global) = MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(ev_ctx, false)
                {
                    let is_hot_or_not = true;
                    MixPanelEvent::track_video_clicked(
                        global,
                        post.poster_principal.to_text(),
                        is_hot_or_not,
                        video_id,
                        MixpanelPostGameType::HotOrNot,
                        MixpanelVideoClickedCTAType::NsfwToggle,
                    );
                }
            } else {
                if !nsfw_enabled().unwrap_or(false) && show_nsfw_permission() {
                    show_nsfw_permission.set(false);
                    if let Some(global) =
                        MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(ev_ctx, false)
                    {
                        MixPanelEvent::track_nsfw_enabled(
                            global,
                            post_clone.poster_principal.to_text(),
                            video_id,
                            post_clone.is_nsfw,
                            "home".to_string(),
                            None,
                        );
                    }
                    set_nsfw_enabled(Some(true));
                } else {
                    set_nsfw_enabled(Some(false));
                    if let Some(global) =
                        MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(ev_ctx, false)
                    {
                        let is_hot_or_not = true;
                        MixPanelEvent::track_video_clicked(
                            global,
                            post.poster_principal.to_text(),
                            is_hot_or_not,
                            video_id,
                            MixpanelPostGameType::HotOrNot,
                            MixpanelVideoClickedCTAType::NsfwToggle,
                        );
                    }
                }
                // using set_href to hard reload the page
                let window = window();
                let _ = window.location().set_href(&format!(
                    "/?nsfw={}",
                    nsfw_enabled.get_untracked().unwrap_or(false)
                ));
            }
        }
    });

    let mixpanel_track_profile_click = move || {
        let video_id = profile_click_video_id.clone();
        let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
            return;
        };
        let is_hot_or_not = true;
        MixPanelEvent::track_video_clicked(
            global,
            post.poster_principal.to_string(),
            is_hot_or_not,
            video_id,
            MixpanelPostGameType::HotOrNot,
            MixpanelVideoClickedCTAType::CreatorProfile,
        );
    };

    let auth = auth_state();

    let show_sats_airdrop_popup = RwSignal::new(false);
    let sats_airdrop_claimed = RwSignal::new(false);
    let sats_airdrop_amount = RwSignal::new(0u64);
    let sats_airdrop_error = RwSignal::new(false);

    let claim_sats_airdrop_action = Action::new_local(move |_| async move {
        show_sats_airdrop_popup.set(true);
        sats_airdrop_claimed.set(false);
        sats_airdrop_error.set(false);

        let Ok(auth_cans) = auth.auth_cans().await else {
            if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                MixPanelEvent::track_claim_airdrop_clicked(
                    global,
                    StakeType::Sats,
                    "home".to_string(),
                );
            }
            log::warn!("Failed to get authenticated canisters");
            sats_airdrop_error.set(true);
            return Err(ServerFnError::new("Failed to get authenticated canisters"));
        };
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_claim_airdrop_clicked(global, StakeType::Sats, "home".to_string());
        }
        let user_canister = auth_cans.user_canister();
        let user_principal = auth_cans.user_principal();
        let request = hon_worker_common::ClaimRequest { user_principal };
        let signature =
            hon_worker_common::sign_claim_request(auth_cans.identity(), request.clone()).unwrap();
        claim_sats_airdrop(user_canister, request, signature)
            .await
            .inspect(|&amount| {
                sats_airdrop_claimed.set(true);
                sats_airdrop_amount.set(amount);

                let (_, set_wallet_balance_store, _) =
                    use_local_storage::<u64, FromToStringCodec>(WALLET_BALANCE_STORE_KEY);

                set_wallet_balance_store.update(|balance| {
                    *balance += amount;
                });

                if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                    MixPanelEvent::track_airdrop_claimed(
                        global,
                        StakeType::Sats,
                        true,
                        amount,
                        "home".to_string(),
                    );
                }
            })
            .inspect_err(|_| {
                log::warn!("Something went wrong claiming airdrop");
                sats_airdrop_error.set(true);
                if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                    MixPanelEvent::track_airdrop_claimed(
                        global,
                        StakeType::Sats,
                        false,
                        0,
                        "home".to_string(),
                    );
                }
            })
    });

    let AudioState { muted, volume } = AudioState::get();

    view! {
        <MuteUnmuteControl muted volume />
        <div class="flex absolute bottom-0 left-0 flex-col flex-nowrap justify-between pt-5 pb-20 w-full h-full text-white bg-transparent pointer-events-none px-[16px] z-4 md:px-[16px]">
            // Group top content together
            <div class="flex flex-col w-full">
                <div class="flex flex-row justify-between items-center w-full pointer-events-auto">
                    <div class="flex flex-row gap-2 items-center p-2 w-9/12 rounded-s-full bg-linear-to-r from-black/25 via-80% via-black/10">
                    <div class="flex w-fit">
                        <a
                            href=profile_url.clone()
                            class="w-10 h-10 rounded-full border-2 md:w-12 md:h-12 overflow-clip border-primary-600"
                        >
                            <img class="object-cover w-full h-full" src=post.propic_url fetchpriority="low" loading={if high_priority { "eager" } else { "lazy" }} />
                        </a>
                    </div>
                    <div class="flex flex-col justify-center min-w-0">
                        <div class="flex flex-row gap-1 items-center text-xs md:text-sm lg:text-base">
                            <span class="font-semibold truncate">
                                <a
                                    on:click=move |_| mixpanel_track_profile_click()
                                    href=profile_url
                                >
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
                        <ExpandableText clone:post description=post.description />
                    </div>
                </div>
                <button class="py-2 pointer-events-auto">
                    <img
                        on:click=move |_| {
                            let _ = click_nsfw.dispatch(());
                        }
                        src=move || {
                            if nsfw_enabled().unwrap_or(false) {
                                "/img/yral/nsfw/nsfw-toggle-on.webp"
                            } else {
                                "/img/yral/nsfw/nsfw-toggle-off.webp"
                            }
                        }
                        class="object-contain w-[76px] h-[36px]"
                        alt="NSFW Toggle"
                    />
                    </button>
                </div>
            </div>
        </div>
        <Modal show=show_nsfw_permission>
            <div class="flex flex-col gap-4 justify-center items-center text-white">
                <img class="object-contain w-32 h-32" src="/img/yral/nsfw/nsfw-modal-logo.svg" />
                <h1 class="text-xl font-bold font-kumbh">Enable NSFW Content?</h1>
                <span class="text-sm font-thin text-center md:w-80 w-50 font-kumbh">
                    By enabling NSFW content, you confirm that you are 18 years or older and consent to viewing content that may include explicit, sensitive, or mature material. This content is intended for adult audiences only and may not be suitable for all viewers. Viewer discretion is advised.
                </span>
                <div class="flex flex-col gap-4 items-center w-full">
                    <a
                        class="text-sm font-bold text-center text-[#E2017B] font-kumbh"
                        href="/terms-of-service"
                    >
                        View NSFW Content Policy
                    </a>
                </div>
                <HighlightedButton
                    classes="w-full mt-4".to_string()
                    alt_style=false
                    disabled=false
                    on_click=move || {
                        click_nsfw.dispatch(());
                    }
                >
                    I Agree
                </HighlightedButton>
            </div>
        </Modal>
        <SatsAirdropPopup
            show=show_sats_airdrop_popup
            claimed=sats_airdrop_claimed.read_only()
            amount_claimed=sats_airdrop_amount.read_only()
            error=sats_airdrop_error.read_only()
            try_again=claim_sats_airdrop_action
        />
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
                        step="0.05"
                        class="z-[2] appearance-none bg-zinc-500 h-1.5 rounded-full accent-white"
                        prop:value={move || volume_.try_get().unwrap_or(0.0)}
                        on:change=move |ev: leptos::ev::Event| {
                            let input = event_target_value(&ev);
                            if let Ok(value) = input.parse::<f64>() {
                                volume.set(value);
                                if value > 0.0 {
                                    muted.set(false);
                                } else {
                                    muted.set(true);
                                }
                            }
                        }

                    />
                    </div>
                </div>
            </div>
    }
}
