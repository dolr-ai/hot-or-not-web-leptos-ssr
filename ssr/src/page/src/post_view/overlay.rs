use candid::Principal;
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use component::buttons::HighlightedButton;
use component::icons::sound_off_icon::SoundOffIcon;
use component::icons::sound_on_icon::SoundOnIcon;
use component::icons::volume_high_icon::VolumeHighIcon;
use component::icons::volume_mute_icon::VolumeMuteIcon;
use component::leaderboard::GlobalRankBadge;
use component::overlay::ShadowOverlay;
use component::spinner::SpinnerFit;
use component::{hn_icons::HomeFeedShareIcon, modal::Modal, option::SelectOption};
use global_constants::REFERRAL_REWARD_SATS;

use consts::{
    UserOnboardingStore, NSFW_ENABLED_COOKIE, USER_ONBOARDING_STORE_KEY, WALLET_BALANCE_STORE_KEY,
};
use gloo::timers::callback::Timeout;
use leptos::html::Audio;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::*;
use leptos_router::hooks::{use_location, use_navigate};
use leptos_use::storage::use_local_storage;
use leptos_use::use_window;
use leptos_use::{
    use_cookie_with_options, use_interval_fn_with_options, UseCookieOptions, UseIntervalFnOptions,
};
use state::audio_state::AudioState;
use state::canisters::auth_state;
use utils::host::show_nsfw_content;
use utils::{
    event_streaming::events::{LikeVideo, ShareVideo},
    report::ReportOption,
    send_wrap,
    web::{copy_to_clipboard, share_url},
};

use utils::mixpanel::mixpanel_events::*;
use yral_canisters_common::utils::posts::PostDetails;
use yral_canisters_common::Canisters;

use crate::wallet::airdrop::sats_airdrop::{claim_sats_airdrop, get_sats_airdrop_status};
use crate::wallet::airdrop::{AirdropStatus, SatsAirdropPopup};
use leptos::prelude::ServerFnError;

use super::bet::HNGameOverlay;

#[component]
fn LikeAndAuthCanLoader(post: PostDetails) -> impl IntoView {
    let likes = RwSignal::new(post.likes);

    let liked = RwSignal::new(None::<bool>);
    let icon_name = Signal::derive(move || {
        if liked().unwrap_or_default() {
            "/img/heart-icon-liked.svg"
        } else {
            "/img/heart-icon-white.svg"
        }
    });

    let initial_liked = (post.liked_by_user, post.likes);

    let auth: state::canisters::AuthState = auth_state();
    let is_logged_in = auth.is_logged_in_with_oauth();
    let ev_ctx = auth.event_ctx();

    let post_clone = post.clone();
    let like_toggle = Action::new(move |&()| {
        let post_clone = post_clone.clone();
        send_wrap(async move {
            let Ok(canisters) = auth.auth_cans().await else {
                log::warn!("Trying to toggle like without auth");
                return;
            };

            let should_like = {
                let mut liked_w = liked.write();
                let current = liked_w.unwrap_or_default();
                *liked_w = Some(!current);
                !current
            };

            if should_like {
                likes.update(|l| *l += 1);
                LikeVideo.send_event(ev_ctx, post_clone.clone(), likes);

                let is_logged_in = is_logged_in.get_untracked();
                let global = MixpanelGlobalProps::try_get(&canisters, is_logged_in);
                let is_hot_or_not = true;
                MixPanelEvent::track_video_clicked(
                    global,
                    post.poster_principal.to_text(),
                    is_hot_or_not,
                    post_clone.uid.clone(),
                    MixpanelPostGameType::HotOrNot,
                    MixpanelVideoClickedCTAType::Like,
                );
            } else {
                likes.update(|l| *l -= 1);
            }

            //TODO: refactor this to use cans<true>
            match canisters
                .like_post(post_clone.canister_id, post_clone.post_id.clone())
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    log::warn!("Error toggling like status: {e:?}");
                    liked.update(|l| _ = l.as_mut().map(|l| *l = !*l));
                }
            }
        })
    });

    let post_canister = post.canister_id;
    let post_id = post.post_id.clone();

    let liked_fetch = auth.derive_resource(
        || (),
        move |cans: Canisters<true>, _| {
            let post_id = post_id.clone();
            async move {
                let result = if let Some(liked) = initial_liked.0 {
                    (liked, initial_liked.1)
                } else {
                    match cans.post_like_info(post_canister, post_id.clone()).await {
                        Ok(liked) => liked,
                        Err(e) => {
                            log::warn!("faild to fetch likes {e}");
                            (false, likes.try_get_untracked().unwrap_or_default())
                        }
                    }
                };
                Ok::<_, ServerFnError>(result)
            }
        },
    );

    view! {
        <div class="flex flex-col gap-1 items-center">
            <button on:click=move |_| {
                like_toggle.dispatch(());
            }>
                <img src=icon_name style="width: 1em; height: 1em;" />
            </button>
            <span class="text-xs md:text-sm">{likes}</span>
            <Suspense>
                {move || Suspend::new(async move {
                    match liked_fetch.await {
                        Ok(res) => {
                            likes.set(res.1);
                            liked.set(Some(res.0))
                        }
                        Err(e) => {
                            log::warn!("failed to fetch like status {e}");
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
pub fn VideoDetailsOverlay(
    post: PostDetails,
    prev_post: Option<(Principal, String)>,
    win_audio_ref: NodeRef<Audio>,
    #[prop(optional, into)] high_priority: bool,
) -> impl IntoView {
    // No need for local context - using global context from App

    let show_share = RwSignal::new(false);
    let show_report = RwSignal::new(false);
    let show_nsfw_permission = RwSignal::new(false);
    let report_option = RwSignal::new(ReportOption::Nudity.as_str().to_string());
    let show_copied_popup = RwSignal::new(false);
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

    let display_name = post.display_name_or_fallback();

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    let post_details_share = post.clone();
    let track_video_id = post.uid.clone();
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

    let track_video_clicked = move |cta_type: MixpanelVideoClickedCTAType| {
        let video_id = track_video_id.clone();
        let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
            return;
        };
        let is_hot_or_not = true;
        MixPanelEvent::track_video_clicked(
            global,
            post.poster_principal.to_text(),
            is_hot_or_not,
            video_id,
            MixpanelPostGameType::HotOrNot,
            cta_type,
        );
    };
    let track_video_share = track_video_clicked.clone();
    let track_video_share = move || track_video_share(MixpanelVideoClickedCTAType::Share);
    let track_video_refer = track_video_clicked.clone();
    let track_video_refer = move || track_video_refer(MixpanelVideoClickedCTAType::ReferAndEarn);
    let track_video_report = track_video_clicked.clone();
    let track_video_report = move || track_video_report(MixpanelVideoClickedCTAType::Report);

    let share = move || {
        let post_details = post_details_share.clone();
        let url = video_url();
        track_video_share();
        if share_url(&url).is_some() {
            return;
        }
        show_share.set(true);
        ShareVideo.send_event(ev_ctx, post_details);
    };

    let profile_url = format!("/profile/{}/tokens", post.username_or_principal());
    let post_c = post.clone();

    let click_copy = move |text: String| {
        _ = copy_to_clipboard(&text);
        show_copied_popup.set(true);
        Timeout::new(1200, move || show_copied_popup.set(false)).forget();
    };

    let post_details_report = post.clone();
    let profile_click_video_id = post.uid.clone();
    let report_video_click_id = post.uid.clone();
    let post_clone = post.clone();
    let click_report = Action::new(move |()| {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_video_reported(
                global,
                post_clone.poster_principal.to_text(),
                true,
                report_video_click_id.clone(),
                MixpanelPostGameType::HotOrNot,
                post_clone.is_nsfw,
                report_option.get_untracked(),
            );
        }
        #[cfg(feature = "ga4")]
        {
            use utils::report::send_report_offchain;

            let post_details = post_details_report.clone();

            spawn_local(async move {
                let cans = auth.auth_cans().await.unwrap();
                let details = cans.profile_details();
                send_report_offchain(
                    details.principal(),
                    post_details.poster_principal.to_string(),
                    post_details.canister_id.to_string(),
                    post_details.post_id.clone(),
                    post_details.uid.clone(),
                    report_option.get_untracked(),
                    video_url(),
                )
                .await
                .unwrap();
            });
        }
        async move {
            show_report.set(false);
        }
    });

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

    let show_tutorial: RwSignal<bool> = RwSignal::new(false);

    let (_, set_onboarding_store, _) =
        use_local_storage::<UserOnboardingStore, JsonSerdeCodec>(USER_ONBOARDING_STORE_KEY);

    let close_help_popup_action = Action::new(move |_: &()| {
        set_onboarding_store.update(|store| {
            store.has_seen_hon_bet_help = true;
        });
        async move {}
    });

    let show_low_balance_popup: RwSignal<bool> = RwSignal::new(false);
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

    let navigate = use_navigate();
    let navigate_to_refer = Action::new(move |is_airdrop_eligible: &bool| {
        let navigate = navigate.clone();
        let is_airdrop_eligible = *is_airdrop_eligible;
        async move {
            let Ok(_) = auth.auth_cans().await else {
                return;
            };
            if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                MixPanelEvent::track_refer_friend_clicked(
                    global,
                    is_airdrop_eligible,
                    "low_sats_popup".to_string(),
                    "home".to_string(),
                );
            }
            navigate("/refer-earn", Default::default());
        }
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
                            if post.is_nsfw {
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
                // Add the rank badge here, below the profile/NSFW row
                <div class="flex justify-end w-full mt-2 pointer-events-auto">
                    <GlobalRankBadge />
                </div>
            </div>
            // Bottom content stays at the bottom
            <div class="flex flex-col gap-2 w-full">
                <div class="flex flex-col gap-6 items-end self-end text-2xl pointer-events-auto md:text-3xl lg:text-4xl">
                    <button on:click=move |_| {
                        track_video_report();
                        show_report.set(true);
                    }>
                        <Icon attr:class="drop-shadow-lg" icon=icondata::TbMessageReport />
                    </button>
                    <a on:click=move |_| track_video_refer() href="/refer-earn">
                        <Icon attr:class="drop-shadow-lg" icon=icondata::AiGiftFilled />
                    </a>
                    <LikeAndAuthCanLoader post=post_c.clone() />
                    <button on:click=move |_| share()>
                        <Icon attr:class="drop-shadow-lg" icon=HomeFeedShareIcon />
                    </button>
                </div>
                <div class="w-full bg-transparent pointer-events-auto max-w-lg mx-auto">
                    <HNGameOverlay post=post_c prev_post=prev_post win_audio_ref show_tutorial show_low_balance_popup />
                </div>
            </div>
        </div>
        <Modal show=show_share>
            <div class="flex flex-col gap-4 justify-center items-center text-white">
                <span class="text-lg">Share</span>
                <div class="flex flex-row gap-2 w-full">
                    <p class="overflow-x-scroll p-2 max-w-full whitespace-nowrap rounded-full text-md bg-white/10">
                        {move  || { video_url()}}
                    </p>
                    <button on:click=move |_| click_copy(video_url())>
                        <Icon attr:class="text-xl" icon=icondata::FaCopyRegular />
                    </button>
                </div>
            </div>

            <Show when=show_copied_popup>
                <div class="flex flex-col justify-center items-center">
                    <span class="flex absolute flex-row justify-center items-center mt-80 w-28 h-10 text-center rounded-md shadow-lg bg-white/90">
                        <p>Link Copied!</p>
                    </span>
                </div>
            </Show>
        </Modal>
        <Modal show=show_report>
            <div class="flex flex-col gap-4 justify-center items-center text-white">
                <span class="text-lg">Report Post</span>
                <span class="text-lg">Please select a reason:</span>
                <div class="max-w-full text-black text-md">
                    <select
                        class="block p-2 w-full text-sm rounded-lg"
                        on:change=move |ev| {
                            let new_value = event_target_value(&ev);
                            report_option.set(new_value);
                        }
                    >

                        <SelectOption
                            value=report_option.read_only()
                            is=format!("{}", ReportOption::Nudity.as_str())
                        />
                        <SelectOption
                            value=report_option.read_only()
                            is=format!("{}", ReportOption::Violence.as_str())
                        />
                        <SelectOption
                            value=report_option.read_only()
                            is=format!("{}", ReportOption::Offensive.as_str())
                        />
                        <SelectOption
                            value=report_option.read_only()
                            is=format!("{}", ReportOption::Spam.as_str())
                        />
                        <SelectOption
                            value=report_option.read_only()
                            is=format!("{}", ReportOption::Other.as_str())
                        />
                    </select>
                </div>
                <button on:click=move |_| {
                    click_report.dispatch(());
                }>
                    <div class="p-1 bg-pink-500 rounded-lg">Submit</div>
                </button>
            </div>
        </Modal>
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
        <HotOrNotTutorialOverlay show=show_tutorial close_action=close_help_popup_action />
        <LowSatsBalancePopup
            show=show_low_balance_popup
            navigate_refer_page=navigate_to_refer
            claim_airdrop=Action::new(move |_| {
                show_low_balance_popup.set(false);
                claim_sats_airdrop_action.dispatch(auth.is_logged_in_with_oauth().get());
                async move {}
            })
            auth=auth
        />
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

#[component]
pub fn HotOrNotTutorialOverlay(
    show: RwSignal<bool>,
    close_action: Action<(), ()>,
) -> impl IntoView {
    view! {
        <ShadowOverlay show=show >
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div style="max-height: 90vh;" class="overflow-hidden overflow-y-auto h-fit max-w-md items-center cursor-auto bg-neutral-950 rounded-md w-full relative">
                    <div
                        style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                        class="absolute z-[1] -left-1/2 top-0 size-[32rem]" >
                    </div>
                    <button
                        on:click=move |_| {
                            show.set(false);
                            close_action.dispatch(());
                        }
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[3] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                    <div class="flex z-[2] relative flex-col items-center gap-2 text-white justify-center p-12">
                        <div class="text-lg font-bold">"How to play?"</div>
                        <div class="font-bold text-yellow-500 pb-4 text-center">"Stake YRAL to vote HOT or NOT."</div>
                        <div class="border rounded-md border-neutral-800 bg-neutral-950 flex p-3 gap-4 items-center">
                            <img src="/img/hotornot/hot-circular.svg" class="size-12 shrink-0" />
                            <div class="text-neutral-400"><span class="font-bold text-white">"'Hot'"</span>" = Higher engagement score than the previous"</div>
                        </div>
                        <div class="border rounded-md border-neutral-800 bg-neutral-950 flex p-3 gap-4 items-center">
                            <div class="text-neutral-400"><span class="font-bold text-white">"'Not'"</span>" = Lower engagement score than the previous"</div>
                            <img src="/img/hotornot/hot-circular.svg" class="size-12 shrink-0" />
                        </div>
                        <div class="border rounded-md border-neutral-800 bg-neutral-950 flex flex-col p-3 gap-1 items-center justify-center">
                            <div class="text-neutral-400">Example</div>
                            <div class="text-center font-bold text-neutral-300">
                                <div>"Previous video score: 36"</div>
                                <div>"Your vote on the current video: HOT 🔥"</div>
                                <div>"Current video score: 42"</div>
                                <div class="font-semibold">"You scored it right. YRAL coming your way!"</div>
                            </div>
                            <div class="text-sm text-neutral-400"><span class="font-bold text-neutral-300">"Note: "</span>"First video results are random."</div>
                        </div>
                        <div class="text-yellow-500 font-bold text-center py-4">
                            "You make the content, you take the cut — 10% of all YRAL staked!"
                        </div>

                        <HighlightedButton
                            alt_style=false
                            disabled=false
                            on_click=move || { show.set(false) }
                        >
                            "Keep Playing"
                        </HighlightedButton>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
#[component]
pub fn LowSatsBalancePopup(
    show: RwSignal<bool>,
    navigate_refer_page: Action<bool, ()>,
    claim_airdrop: Action<(), ()>,
    auth: state::canisters::AuthState,
) -> impl IntoView {
    let ev_ctx = auth.event_ctx();

    let status_resource = auth.derive_resource(
        move || show.get(),
        move |auth_cans, showing| async move {
            if !showing {
                return Ok(AirdropStatus::Available);
            }
            let user_canister = auth_cans.user_canister();
            let user_principal = auth_cans.user_principal();
            get_sats_airdrop_status(user_canister, user_principal).await
        },
    );

    view! {
        <ShadowOverlay show=show >
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div style="min-height: 62vh;" class="overflow-hidden h-fit max-w-md items-center cursor-auto bg-neutral-950 rounded-md w-full relative">
                    <button
                        on:click=move |_| {
                            show.set(false);
                        }
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[3] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>

                    <div class="flex z-[2] relative flex-col items-center gap-5 text-white justify-center p-12">
                        <img src="/img/hotornot/sad.webp" class="size-14" />
                        <div class="text-xl text-center font-semibold text-neutral-50">"You're Low on YRAL"</div>

                        <Suspense
                            fallback=move || view! {
                                <div class="flex flex-col items-center justify-center w-full py-16">
                                    <div class="size-12">
                                        <SpinnerFit />
                                    </div>
                                 </div>
                            }
                        >
                            {move || Suspend::new(async move {
                                let airdrop_status = status_resource.await.unwrap_or(AirdropStatus::Available);
                                let is_airdrop_eligible = matches!(airdrop_status, AirdropStatus::Available);

                                if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                                    MixPanelEvent::track_low_on_sats_popup_shown(
                                        global,
                                        is_airdrop_eligible,
                                        "home".to_string(),
                                    );
                                }

                                view! {
                                    {
                                        let refer_reward_text = format!("{REFERRAL_REWARD_SATS} YRAL");
                                        match airdrop_status {
                                            AirdropStatus::Available => view! {
                                                    <div class="text-neutral-300 text-center">"Earn more in two easy ways:"</div>
                                                    <ul class="flex list-disc flex-col gap-5 text-neutral-300">
                                                        <li>"Unlock your daily"<span class="font-semibold">" YRAL "</span>"loot every 24 hours!"</li>
                                                        <li>"Refer & earn "<span class="font-semibold">{refer_reward_text.clone()}</span>" for every friend you invite."</li>
                                                        <li class="font-semibold">"Upload Videos to earn commissions."</li>
                                                    </ul>
                                                }.into_any(),
                                            AirdropStatus::WaitFor(duration) => view! {
                                                    <div class="text-neutral-300 text-center">"Looks like you've already claimed your daily airdrop."</div>
                                                    <AirdropCountdown duration=duration />
                                                    <div class="text-neutral-300 text-center">"Meanwhile, earn "<span class="font-semibold">{refer_reward_text.clone()}</span>" for every friend you refer!"</div>
                                                }.into_any(),
                                            AirdropStatus::Claimed => view! {
                                                    <div class="text-neutral-300 text-center">"Looks like you've already claimed your daily airdrop."</div>
                                                    <div class="text-neutral-300 text-center">"Meanwhile, earn "<span class="font-semibold">{refer_reward_text.clone()}</span>" for every friend you refer!"</div>
                                                }.into_any(),
                                        }
                                    }

                                    {
                                        match airdrop_status {
                                            AirdropStatus::Available => view! {
                                                <HighlightedButton
                                                alt_style=false
                                                disabled=false
                                                on_click=move || {
                                                    show.set(false);
                                                    claim_airdrop.dispatch(());
                                                }
                                                >
                                                "Claim YRAL Airdrop"
                                                </HighlightedButton>
                                                <HighlightedButton
                                                alt_style=true
                                                disabled=false
                                                on_click=move || {
                                                    show.set(false);
                                                    navigate_refer_page.dispatch(is_airdrop_eligible);
                                                }
                                                >
                                                "Refer a friend"
                                                </HighlightedButton>
                                            }.into_any(),
                                            _ => view! {
                                                <HighlightedButton
                                                    alt_style=false
                                                    disabled=false
                                                    on_click=move || {
                                                        show.set(false);
                                                        navigate_refer_page.dispatch(is_airdrop_eligible);
                                                    }
                                                    >
                                                    "Refer a friend"
                                                </HighlightedButton>
                                                <HighlightedButton
                                                    alt_style=true
                                                    disabled=false
                                                    on_click=move || {
                                                        show.set(false);
                                                    }
                                                    >
                                                    "Back to Game"
                                                </HighlightedButton>
                                            }.into_any()
                                        }
                                    }
                                }
                            })}
                        </Suspense>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}

#[component]
fn AirdropCountdown(duration: web_time::Duration) -> impl IntoView {
    use utils::time::to_hh_mm_ss;
    use web_time::Instant;

    let end_time = Instant::now() + duration;
    let (remaining_duration, set_remaining_duration) = signal(duration);

    let _ = use_interval_fn_with_options(
        move || {
            let now = Instant::now();
            let remaining = end_time.saturating_duration_since(now);
            set_remaining_duration(remaining);
        },
        1000,
        UseIntervalFnOptions::default().immediate(true),
    );

    view! {
        <div class="bg-[#444444] rounded-md px-3 py-2">
            <span class="text-white text-sm font-medium">
                "Next Airdrop: "{move || to_hh_mm_ss(remaining_duration())}
            </span>
        </div>
    }
}
