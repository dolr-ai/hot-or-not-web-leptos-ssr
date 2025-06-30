use codee::string::{FromToStringCodec, JsonSerdeCodec};
use component::buttons::HighlightedButton;
use component::overlay::ShadowOverlay;
use component::{hn_icons::HomeFeedShareIcon, modal::Modal, option::SelectOption};

use consts::{UserOnboardingStore, NSFW_TOGGLE_STORE, USER_ONBOARDING_STORE_KEY};
use gloo::timers::callback::Timeout;
use leptos::html::Audio;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::*;
use leptos_use::storage::use_local_storage;
use leptos_use::use_window;
use state::canisters::{auth_state, unauth_canisters};
use utils::host::show_nsfw_content;
use utils::{
    event_streaming::events::{LikeVideo, ShareVideo},
    report::ReportOption,
    send_wrap,
    web::{copy_to_clipboard, share_url},
};

use yral_canisters_common::utils::posts::PostDetails;

use utils::mixpanel::mixpanel_events::*;

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

    let post_canister = post.canister_id;
    let post_id = post.post_id;
    let initial_liked = (post.liked_by_user, post.likes);

    let auth: state::canisters::AuthState = auth_state();
    let is_logged_in = auth.is_logged_in_with_oauth();
    let ev_ctx = auth.event_ctx();

    let like_toggle = Action::new(move |&()| {
        let post_details = post.clone();
        let video_id = post.uid.clone();
        send_wrap(async move {
            let Ok(canisters) = auth.auth_cans(unauth_canisters()).await else {
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
                LikeVideo.send_event(ev_ctx, post_details.clone(), likes);

                let is_logged_in = is_logged_in.get_untracked();
                let global = MixpanelGlobalProps::try_get(&canisters, is_logged_in);
                let is_hot_or_not = true;
                MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                    user_id: global.user_id,
                    visitor_id: global.visitor_id,
                    is_logged_in: global.is_logged_in,
                    canister_id: global.canister_id,
                    is_nsfw_enabled: global.is_nsfw_enabled,
                    is_nsfw: post.is_nsfw,
                    is_game_enabled: is_hot_or_not,
                    publisher_user_id: post.poster_principal.to_text(),
                    game_type: MixpanelPostGameType::HotOrNot,
                    cta_type: MixpanelVideoClickedCTAType::Like,
                    video_id,
                    view_count: post.views,
                    like_count: post.likes,
                });
            } else {
                likes.update(|l| *l -= 1);
            }

            let individual = canisters.individual_user(post_canister).await;
            match individual
                .update_post_toggle_like_status_by_caller(post_id)
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

    let liked_fetch = auth.derive_resource(
        || (),
        move |cans, _| {
            send_wrap(async move {
                let result = if let Some(liked) = initial_liked.0 {
                    (liked, initial_liked.1)
                } else {
                    match cans.post_like_info(post_canister, post_id).await {
                        Ok(liked) => liked,
                        Err(e) => {
                            log::warn!("faild to fetch likes {e}");
                            (false, likes.try_get_untracked().unwrap_or_default())
                        }
                    }
                };
                Ok::<_, ServerFnError>(result)
            })
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
    prev_post: Option<PostDetails>,
    win_audio_ref: NodeRef<Audio>,
) -> impl IntoView {
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
    let video_url = move || {
        base_url()
            .map(|b| format!("{b}/hot-or-not/{}/{}", post.canister_id, post.post_id))
            .unwrap_or_default()
    };

    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    let post_details_share = post.clone();
    let track_video_id = post.uid.clone();

    if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
        MixPanelEvent::track_video_impression(MixpanelVideoViewedProps {
            user_id: global.user_id,
            visitor_id: global.visitor_id,
            is_logged_in: global.is_logged_in,
            canister_id: global.canister_id,
            is_nsfw_enabled: global.is_nsfw_enabled,
            publisher_user_id: post.poster_principal.to_text(),
            video_id: track_video_id.clone(),
            is_nsfw: post.is_nsfw,
            game_type: MixpanelPostGameType::HotOrNot,
            like_count: post.likes,
            view_count: post.views,
            is_game_enabled: true,
        });
    }

    let track_video_clicked = move |cta_type: MixpanelVideoClickedCTAType| {
        let video_id = track_video_id.clone();
        let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
            return;
        };
        let is_hot_or_not = true;
        MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
            user_id: global.user_id,
            visitor_id: global.visitor_id,
            is_logged_in: global.is_logged_in,
            is_nsfw: post.is_nsfw,
            canister_id: global.canister_id,
            is_nsfw_enabled: global.is_nsfw_enabled,
            is_game_enabled: is_hot_or_not,
            publisher_user_id: post.poster_principal.to_text(),
            game_type: MixpanelPostGameType::HotOrNot,
            cta_type,
            video_id,
            view_count: post.views,
            like_count: post.likes,
        });
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

    let profile_url = format!("/profile/{}/tokens", post.poster_principal.to_text());
    let post_c = post.clone();

    let click_copy = move |text: String| {
        _ = copy_to_clipboard(&text);
        show_copied_popup.set(true);
        Timeout::new(1200, move || show_copied_popup.set(false)).forget();
    };

    let post_details_report = post.clone();
    let profile_click_video_id = post.uid.clone();
    let report_video_click_id = post.uid.clone();
    let click_report = Action::new(move |()| {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_video_reported(MixpanelVideoReportedProps {
                user_id: global.user_id,
                visitor_id: global.visitor_id,
                is_logged_in: global.is_logged_in,
                canister_id: global.canister_id,
                is_nsfw_enabled: global.is_nsfw_enabled,
                publisher_user_id: post.poster_principal.to_text(),
                video_id: report_video_click_id.clone(),
                is_nsfw: post.is_nsfw,
                is_game_enabled: true,
                game_type: MixpanelPostGameType::HotOrNot,
                report_reason: report_option.get_untracked(),
            });
        }
        #[cfg(feature = "ga4")]
        {
            use utils::report::send_report_offchain;

            let post_details = post_details_report.clone();
            let base = unauth_canisters();

            spawn_local(async move {
                let cans = auth.auth_cans(base).await.unwrap();
                let details = cans.profile_details();
                send_report_offchain(
                    details.principal(),
                    post_details.poster_principal.to_string(),
                    post_details.canister_id.to_string(),
                    post_details.post_id.to_string(),
                    post_details.uid,
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

    let (nsfw_enabled, set_nsfw_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NSFW_TOGGLE_STORE);
    let nsfw_enabled_with_host = Signal::derive(move || {
        if show_nsfw_content() {
            true
        } else {
            nsfw_enabled()
        }
    });
    let click_nsfw = Action::new(move |()| {
        let video_id = post.uid.clone();
        async move {
            if show_nsfw_content() {
                return;
            }

            if !nsfw_enabled() && !show_nsfw_permission() {
                show_nsfw_permission.set(true);
                if let Some(global) = MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(ev_ctx, false)
                {
                    let is_hot_or_not = true;
                    MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                        user_id: global.user_id,
                        visitor_id: global.visitor_id,
                        is_logged_in: global.is_logged_in,
                        is_nsfw: post.is_nsfw,
                        canister_id: global.canister_id,
                        is_nsfw_enabled: global.is_nsfw_enabled,
                        is_game_enabled: is_hot_or_not,
                        publisher_user_id: post.poster_principal.to_text(),
                        game_type: MixpanelPostGameType::HotOrNot,
                        cta_type: MixpanelVideoClickedCTAType::NsfwToggle,
                        video_id,
                        view_count: post.views,
                        like_count: post.likes,
                    });
                }
            } else {
                if !nsfw_enabled() && show_nsfw_permission() {
                    show_nsfw_permission.set(false);
                    if let Some(global) =
                        MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(ev_ctx, false)
                    {
                        MixPanelEvent::track_nsfw_true(MixpanelNsfwToggleProps {
                            user_id: global.user_id,
                            visitor_id: global.visitor_id,
                            is_logged_in: global.is_logged_in,
                            canister_id: global.canister_id,
                            is_nsfw_enabled: global.is_nsfw_enabled,
                            publisher_user_id: post.poster_principal.to_text(),
                            video_id,
                            is_nsfw: post.is_nsfw,
                        });
                    }
                    set_nsfw_enabled(true);
                } else {
                    set_nsfw_enabled(false);
                    if let Some(global) =
                        MixpanelGlobalProps::from_ev_ctx_with_nsfw_info(ev_ctx, false)
                    {
                        let is_hot_or_not = true;
                        MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                            user_id: global.user_id,
                            visitor_id: global.visitor_id,
                            is_logged_in: global.is_logged_in,
                            is_nsfw: post.is_nsfw,
                            canister_id: global.canister_id,
                            is_nsfw_enabled: global.is_nsfw_enabled,
                            is_game_enabled: is_hot_or_not,
                            publisher_user_id: post.poster_principal.to_text(),
                            game_type: MixpanelPostGameType::HotOrNot,
                            cta_type: MixpanelVideoClickedCTAType::NsfwToggle,
                            video_id,
                            view_count: post.views,
                            like_count: post.likes,
                        });
                    }
                }
                // using set_href to hard reload the page
                let window = window();
                let _ = window
                    .location()
                    .set_href(&format!("/?nsfw={}", nsfw_enabled.get_untracked()));
            }
        }
    });

    let mixpanel_track_profile_click = move || {
        let video_id = profile_click_video_id.clone();
        let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) else {
            return;
        };
        let is_hot_or_not = true;
        MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
            user_id: global.user_id,
            visitor_id: global.visitor_id,
            is_logged_in: global.is_logged_in,
            is_nsfw: post.is_nsfw,
            canister_id: global.canister_id,
            is_nsfw_enabled: global.is_nsfw_enabled,
            is_game_enabled: is_hot_or_not,
            publisher_user_id: post.poster_principal.to_text(),
            game_type: MixpanelPostGameType::HotOrNot,
            cta_type: MixpanelVideoClickedCTAType::CreatorProfile,
            video_id,
            view_count: post.views,
            like_count: post.likes,
        });
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

    view! {
        <div class="flex absolute bottom-0 left-0 flex-col flex-nowrap justify-between pt-5 pb-20 w-full h-full text-white bg-transparent pointer-events-none px-[16px] z-4 md:px-[16px]">
            <div class="flex flex-row justify-between items-center w-full pointer-events-auto">
                <div class="flex flex-row gap-2 items-center p-2 w-9/12 rounded-s-full bg-linear-to-r from-black/25 via-80% via-black/10">
                    <div class="flex w-fit">
                        <a
                            href=profile_url.clone()
                            class="w-10 h-10 rounded-full border-2 md:w-12 md:h-12 overflow-clip border-primary-600"
                        >
                            <img class="object-cover w-full h-full" src=post.propic_url />
                        </a>
                    </div>
                    <div class="flex flex-col justify-center min-w-0">
                        <div class="flex flex-row gap-1 items-center text-xs md:text-sm lg:text-base">
                            <span class="font-semibold truncate">
                                <a
                                    on:click=move |_| mixpanel_track_profile_click()
                                    href=profile_url
                                >
                                    {post.display_name}
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
                            if nsfw_enabled_with_host() {
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
                    <HNGameOverlay post=post_c prev_post=prev_post win_audio_ref show_tutorial />
                </div>
            </div>
        </div>
        <Modal show=show_share>
            <div class="flex flex-col gap-4 justify-center items-center text-white">
                <span class="text-lg">Share</span>
                <div class="flex flex-row gap-2 w-full">
                    <p class="overflow-x-scroll p-2 max-w-full whitespace-nowrap rounded-full text-md bg-white/10">
                        {video_url}
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
pub fn HotOrNotTutorialOverlay(
    show: RwSignal<bool>,
    close_action: Action<(), ()>,
) -> impl IntoView {
    view! {
        <ShadowOverlay show=show >
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div class="overflow-hidden h-fit max-w-md items-center cursor-auto bg-neutral-950 rounded-md w-full relative">
                    <img src="/img/common/refer-bg.webp" class="absolute inset-0 z-0 w-full h-full object-cover opacity-40" />
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
                        <div class="font-bold text-yellow-500 pb-4">"Stake Bitcoin (SATS) to vote HOT or NOT."</div>
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
                                <div class="font-semibold">"You scored it right. Bitcoin coming your way!"</div>
                            </div>
                            <div class="text-sm text-neutral-400"><span class="font-bold text-neutral-300">"Note: "</span>"First video results are random."</div>
                        </div>
                        <div class="text-yellow-500 font-bold text-center py-4">
                            "You make the content, you take the cut — 10% of all SATS staked!"
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
