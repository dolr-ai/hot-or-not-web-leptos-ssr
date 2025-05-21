use codee::string::FromToStringCodec;
use component::buttons::HighlightedButton;
use component::{
    canisters_prov::with_cans, hn_icons::HomeFeedShareIcon, modal::Modal, option::SelectOption,
};

use consts::NSFW_TOGGLE_STORE;
use gloo::timers::callback::Timeout;
use leptos::html::Audio;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::*;
use leptos_use::storage::use_local_storage;
use leptos_use::use_window;
use utils::event_streaming::events::auth_canisters_store;
use utils::host::show_nsfw_content;
use utils::{
    event_streaming::events::{LikeVideo, ShareVideo},
    report::ReportOption,
    route::failure_redirect,
    send_wrap,
    user::UserDetails,
    web::{copy_to_clipboard, share_url},
};

use yral_canisters_common::{utils::posts::PostDetails, Canisters};

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
    let canisters = auth_canisters_store();
    let (is_connected, _, _) =
        use_local_storage::<bool, FromToStringCodec>(consts::ACCOUNT_CONNECTED_STORE);

    let like_toggle = Action::new_local(move |&()| {
        let post_details = post.clone();
        let canister_store = canisters;
        let video_id = post.uid.clone();

        async move {
            let Some(canisters) = canisters.get_untracked() else {
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
                LikeVideo.send_event(post_details.clone(), likes, canister_store);
                let is_logged_in = is_connected.get_untracked();
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
        }
    });

    let liked_fetch = with_cans(move |cans: Canisters<true>| {
        send_wrap(async move {
            let result = if let Some(liked) = initial_liked.0 {
                (liked, initial_liked.1)
            } else {
                match cans.post_like_info(post_canister, post_id).await {
                    Ok(liked) => liked,
                    Err(e) => {
                        failure_redirect(e);
                        (false, likes.try_get_untracked().unwrap_or_default())
                    }
                }
            };
            Ok::<_, ServerFnError>(result)
        })
    });

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
pub fn VideoDetailsOverlay(post: PostDetails, win_audio_ref: NodeRef<Audio>) -> impl IntoView {
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

    let (is_connected, _, _) =
        use_local_storage::<bool, FromToStringCodec>(consts::ACCOUNT_CONNECTED_STORE);

    let post_details_share = post.clone();
    let canisters = auth_canisters_store();
    let canisters_copy = canisters;
    let share_video_id = post.uid.clone();
    let report_video_id = post.uid.clone();

    let share = move || {
        let video_id = share_video_id.clone();
        let post_details = post_details_share.clone();
        let url = video_url();
        if share_url(&url).is_some() {
            return;
        }
        show_share.set(true);
        ShareVideo.send_event(post_details, canisters);
        if let Some(cans) = canisters.get() {
            let is_logged_in = is_connected.get_untracked();
            let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
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
                cta_type: MixpanelVideoClickedCTAType::Share,
                video_id,
                view_count: post.views,
                like_count: post.likes,
            });
        }
    };

    let mixpanel_track_refer = move || {
        let video_id = report_video_id.clone();
        if let Some(cans) = canisters.get() {
            let is_logged_in = is_connected.get_untracked();
            let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
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
                cta_type: MixpanelVideoClickedCTAType::ReferAndEarn,
                video_id,
                view_count: post.views,
                like_count: post.likes,
            });
        }
    };

    let profile_url = format!("/profile/{}/tokens", post.poster_principal.to_text());
    let post_c = post.clone();

    let click_copy = move |text: String| {
        _ = copy_to_clipboard(&text);
        show_copied_popup.set(true);
        Timeout::new(1200, move || show_copied_popup.set(false)).forget();
    };

    let post_details_report = post.clone();
    let report_video_id = post.uid.clone();
    let profile_click_video_id = post.uid.clone();
    let click_report = Action::new(move |()| {
        let video_id = report_video_id.clone();
        #[cfg(feature = "ga4")]
        {
            use utils::report::send_report_offchain;

            let post_details = post_details_report.clone();
            let user_details = UserDetails::try_get_from_canister_store(canisters_copy).unwrap();

            spawn_local(async move {
                send_report_offchain(
                    user_details.details.principal.to_string(),
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

        if let Some(cans) = canisters.get() {
            let is_logged_in = is_connected.get_untracked();
            let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
            let is_hot_or_not = true;
            MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                user_id: global.user_id,
                visitor_id: global.visitor_id,
                is_logged_in: global.is_logged_in,
                canister_id: global.canister_id,
                is_nsfw: post.is_nsfw,

                is_nsfw_enabled: global.is_nsfw_enabled,
                is_game_enabled: is_hot_or_not,
                publisher_user_id: post.poster_principal.to_text(),
                game_type: MixpanelPostGameType::HotOrNot,
                cta_type: MixpanelVideoClickedCTAType::Report,
                video_id,
                view_count: post.views,
                like_count: post.likes,
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
            } else {
                if !nsfw_enabled() && show_nsfw_permission() {
                    show_nsfw_permission.set(false);
                    if let Some(cans) = canisters.get() {
                        let is_logged_in = is_connected.get_untracked();
                        let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
                        let is_hot_or_not = true;
                        MixPanelEvent::track_video_clicked(MixpanelVideoClickedProps {
                            user_id: global.user_id,
                            visitor_id: global.visitor_id,
                            is_logged_in: global.is_logged_in,
                            canister_id: global.canister_id,
                            is_nsfw: post.is_nsfw,
                            is_nsfw_enabled: global.is_nsfw_enabled,
                            is_game_enabled: is_hot_or_not,
                            publisher_user_id: post.poster_principal.to_text(),
                            game_type: MixpanelPostGameType::HotOrNot,
                            cta_type: MixpanelVideoClickedCTAType::NsfwTrue,
                            video_id,
                            view_count: post.views,
                            like_count: post.likes,
                        });
                    }
                    set_nsfw_enabled(!nsfw_enabled());
                } else {
                    set_nsfw_enabled(!nsfw_enabled());
                    if let Some(cans) = canisters.get() {
                        let is_logged_in = is_connected.get_untracked();
                        let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
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
                            cta_type: MixpanelVideoClickedCTAType::NsfwFalse,
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
                    .set_href(&format!("/?nsfw={}", nsfw_enabled()));
            }
        }
    });

    let mixpanel_track_profile_click = move || {
        let video_id = profile_click_video_id.clone();
        if let Some(cans) = canisters.get() {
            let is_logged_in = is_connected.get_untracked();
            let global = MixpanelGlobalProps::try_get(&cans, is_logged_in);
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
        }
    };

    view! {
        <div class="flex flex-col pointer-events-none flex-nowrap h-full justify-between pt-5 pb-20 px-[16px] md:px-[16px] w-full text-white absolute bottom-0 left-0 bg-transparent z-[4]">
            <div class="flex pointer-events-auto flex-row justify-between w-full items-center">
                <div class="flex flex-row gap-2 w-9/12 rounded-s-full bg-gradient-to-r from-black/25 via-80% via-black/10 items-center p-2">
                    <div class="w-fit flex">
                        <a
                            href=profile_url.clone()
                            class="w-10 md:w-12 h-10 md:h-12 overflow-clip rounded-full border-primary-600 border-2"
                        >
                            <img class="h-full w-full object-cover" src=post.propic_url />
                        </a>
                    </div>
                    <div class="flex flex-col justify-center min-w-0">
                        <div class="flex flex-row text-xs md:text-sm lg:text-base gap-1 items-center">
                            <span class="font-semibold truncate">
                                <a on:click=move|_| mixpanel_track_profile_click() href=profile_url>{post.display_name}</a>
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
                <button class="pointer-events-auto py-2">
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
                        class="w-[76px] h-[36px] object-contain"
                        alt="NSFW Toggle"
                    />
                </button>
            </div>
            <div class="flex flex-col gap-2 w-full">
                <div class="flex flex-col pointer-events-auto gap-6 self-end items-end text-2xl md:text-3xl lg:text-4xl">
                    <button on:click=move |_| show_report.set(true)>
                        <Icon attr:class="drop-shadow-lg" icon=icondata::TbMessageReport />
                    </button>
                    <a on:click=move|_| mixpanel_track_refer()  href="/refer-earn">
                        <Icon attr:class="drop-shadow-lg" icon=icondata::AiGiftFilled />
                    </a>
                    <LikeAndAuthCanLoader post=post_c.clone() />
                    <button on:click=move |_| share()>
                        <Icon attr:class="drop-shadow-lg" icon=HomeFeedShareIcon />
                    </button>
                </div>
                <div class="w-full bg-transparent pointer-events-auto">
                    <HNGameOverlay post=post_c win_audio_ref />
                </div>
            </div>
        </div>
        <Modal show=show_share>
            <div class="flex flex-col justify-center items-center gap-4 text-white">
                <span class="text-lg">Share</span>
                <div class="flex flex-row w-full gap-2">
                    <p class="text-md max-w-full bg-white/10 rounded-full p-2 overflow-x-scroll whitespace-nowrap">
                        {video_url}
                    </p>
                    <button on:click=move |_| click_copy(video_url())>
                        <Icon attr:class="text-xl" icon=icondata::FaCopyRegular />
                    </button>
                </div>
            </div>

            <Show when=show_copied_popup>
                <div class="flex flex-col justify-center items-center">
                    <span class="absolute mt-80 flex flex-row justify-center items-center bg-white/90 rounded-md h-10 w-28 text-center shadow-lg">
                        <p>Link Copied!</p>
                    </span>
                </div>
            </Show>
        </Modal>
        <Modal show=show_report>
            <div class="flex flex-col justify-center items-center gap-4 text-white">
                <span class="text-lg">Report Post</span>
                <span class="text-lg">Please select a reason:</span>
                <div class="max-w-full text-md text-black">
                    <select
                        class="p-2 w-full block rounded-lg text-sm"
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
                    <div class="rounded-lg bg-pink-500 p-1">Submit</div>
                </button>
            </div>
        </Modal>
        <Modal show=show_nsfw_permission>
            <div class="flex flex-col justify-center items-center gap-4 text-white">
                <img class="h-32 w-32 object-contain" src="/img/yral/nsfw/nsfw-modal-logo.svg" />
                <h1 class="text-xl font-bold font-kumbh">Enable NSFW Content?</h1>
                <span class="text-sm w-50 md:w-80 text-center font-kumbh font-thin">
                    By enabling NSFW content, you confirm that you are 18 years or older and consent to viewing content that may include explicit, sensitive, or mature material. This content is intended for adult audiences only and may not be suitable for all viewers. Viewer discretion is advised.
                </span>
                <div class="flex flex-col w-full gap-4 items-center">
                    <a
                        class="text-[#E2017B] font-bold text-sm text-center font-kumbh"
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
    }.into_any()
}

#[component]
fn ExpandableText(description: String) -> impl IntoView {
    let truncated = RwSignal::new(true);

    view! {
        <span
            class="text-xs md:text-sm lg:text-base w-full"
            class:truncate=truncated

            on:click=move |_| truncated.update(|e| *e = !*e)
        >
            {description}
        </span>
    }
}
