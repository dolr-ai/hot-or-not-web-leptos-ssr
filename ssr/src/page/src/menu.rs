use codee::string::FromToStringCodec;
use component::content_upload::AuthorizedUserToSeedContent;
use component::content_upload::YoutubeUpload;
use component::modal::Modal;
use component::title::TitleText;
use component::toggle::Toggle;
use component::{connect::ConnectLogin, social::*};
use consts::{NOTIFICATIONS_ENABLED_STORE, NSFW_ENABLED_COOKIE};
use leptos::either::Either;
use leptos::html::{Div, Input};
use leptos::portal::Portal;
use leptos::prelude::*;
use leptos::web_sys::{Notification, NotificationPermission};
use leptos_icons::*;
use leptos_meta::*;
use leptos_router::{components::Redirect, hooks::use_navigate, hooks::use_query_map};
use leptos_use::{storage::use_local_storage, use_cookie_with_options, UseCookieOptions};
use state::app_state::AppState;
use state::canisters::auth_state;
use state::content_seed_client::ContentSeedClient;
use utils::mixpanel::mixpanel_events::*;
use utils::send_wrap;
use yral_canisters_common::utils::profile::ProfileDetails;

#[component]
fn MenuItem(
    #[prop(into)] text: String,
    #[prop(into)] href: String,
    #[prop(into)] icon: icondata::Icon,
    #[prop(into, optional)] target: String,
    click_cta_type: MixpanelMenuClickedCTAType,
) -> impl IntoView {
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();
    let track_menu_clicked = move || {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            MixPanelEvent::track_menu_clicked(global, click_cta_type);
        }
    };
    view! {
        <a on:click=move |_| track_menu_clicked.clone()()  href=href class="flex items-center justify-between w-full" target=target>
            <div class="flex flex-row gap-4 items-center">
                <Icon attr:class="text-xl" icon=icon />
                <span class="text-base">{text}</span>
            </div>
            <Icon attr:class="text-xl" icon=icondata::AiRightOutlined />
        </a>
    }
    .into_any()
}

#[component]
fn MenuItemWithToggle(
    #[prop(into)] text: String,
    #[prop(into)] icon: icondata::Icon,
    checked: Signal<bool>,
    node_ref: NodeRef<Input>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between w-full">
            <div class="flex flex-row gap-4 items-center flex-1">
                <Icon attr:class="text-xl flex-shrink-0" icon=icon />
                <span class="text-base">{text}</span>
            </div>
            <div class="flex-shrink-0">
                <Toggle checked=checked node_ref=node_ref />
            </div>
        </div>
    }
}

#[component]
fn SectionHeader(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <div class="font-semibold text-xl text-neutral-50">
            {text}
        </div>
    }
}

#[component]
fn MenuFooter() -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4 items-center pt-10 pb-8 w-full">
            <span class="text-sm text-white/50">Follow us on</span>
            <div class="flex flex-row gap-4">
                <Telegram />
                <Discord />
                <Twitter />
                <IcWebsite />
            </div>
            <svg class="h-14 rounded-md outline outline-primary-600 outline-1" viewBox="0 0 228 49">
                <path
                    fill="#F15A24"
                    d="M51.4 12c-3 0-6.1 1.5-9.5 4.5l-4 4.1 3.5 3.8c1-1.2 2.3-2.8 4-4.2 3-2.7 4.9-3.2 6-3.2 4.2 0 7.5 3.3 7.5 7.5 0 4.1-3.4 7.5-7.5 7.5h-.8c1.2.5 2.6.8 3.8.8 7.7 0 9.2-5 9.3-5.4l.3-3C64 17.7 58.3 12 51.4 12Z"
                ></path>
                <path
                    fill="#9E1EED"
                    d="M24.6 37c3 0 6.1-1.5 9.5-4.5l4-4.1-3.5-3.8c-1 1.2-2.3 2.8-4 4.2-3 2.7-4.9 3.2-6 3.2a7.6 7.6 0 0 1-7.5-7.5c0-4.1 3.4-7.5 7.5-7.5h.8c-1.2-.5-2.6-.8-3.8-.8-7.7 0-9.1 5-9.3 5.4A12.6 12.6 0 0 0 24.6 37Z"
                ></path>
                <path
                    fill="#29ABE2"
                    d="M54.4 32.7c-4 0-8-3.2-8.8-4a207 207 0 0 1-7.5-8c-3.7-4-8.6-8.7-13.5-8.7-6 0-11 4.1-12.3 9.6.1-.4 2.1-5.5 9.3-5.4 4 .1 8 3.3 8.8 4.1 2.2 2 7.1 7.5 7.5 8 3.7 4 8.6 8.7 13.5 8.7 6 0 11-4.1 12.3-9.6-.2.4-2.1 5.5-9.3 5.3Z"
                ></path>
                <path
                    fill="#fff"
                    d="M73 33.8c.3 0 .5-.2.5-.5v-6.6c0-.3-.2-.5-.5-.5h-.5c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h.5ZM83.2 33.8c.3 0 .5-.2.5-.5v-6.6c0-.3-.2-.5-.5-.5h-.5c-.3 0-.5.2-.5.5v4.5l-3-4.6a1 1 0 0 0-.8-.4h-.8c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h.5c.3 0 .5-.2.5-.5v-5l3.4 5.3.4.2h.8ZM92.5 27.6c.2 0 .5-.2.5-.5v-.4c0-.3-.3-.5-.5-.5H87c-.2 0-.5.2-.5.5v.4c0 .3.3.5.5.5h2v5.7c0 .3.2.5.5.5h.5c.3 0 .5-.2.5-.5v-5.7h2ZM100.1 33.8c.3 0 .5-.2.5-.5V33c0-.3-.2-.5-.5-.5h-2.8v-1.8h2.5c.3 0 .5-.2.5-.5v-.3c0-.3-.2-.5-.5-.5h-2.5v-1.7h2.8c.3 0 .5-.3.5-.5v-.4c0-.3-.2-.5-.5-.5h-3.8c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h3.8ZM107.5 33.6l.5.2h.5c.4 0 .7-.4.5-.7l-1.3-2.4c1-.3 1.7-1.1 1.7-2.2 0-1.3-1-2.3-2.5-2.3h-2.5c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h.5c.3 0 .5-.2.5-.5V31h.8l1.3 2.7Zm-2.1-4v-2.1h1.2c.8 0 1.2.4 1.2 1 0 .7-.4 1-1.2 1h-1.2ZM118.6 33.8c.3 0 .5-.2.5-.5v-6.6c0-.3-.2-.5-.5-.5h-.5c-.3 0-.5.2-.5.5v4.5l-3-4.6a1 1 0 0 0-.8-.4h-.8c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h.5c.3 0 .5-.2.5-.5v-5l3.4 5.3.4.2h.8ZM127 33.8c.3 0 .5-.2.5-.5V33c0-.3-.2-.5-.5-.5h-2.8v-1.8h2.5c.3 0 .5-.2.5-.5v-.3c0-.3-.2-.5-.5-.5h-2.5v-1.7h2.8c.3 0 .5-.3.5-.5v-.4c0-.3-.2-.5-.5-.5h-3.8c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h3.8ZM136 27.6c.2 0 .4-.2.4-.5v-.4c0-.3-.2-.5-.5-.5h-5.4c-.3 0-.5.2-.5.5v.4c0 .3.2.5.5.5h2v5.7c0 .3.2.5.5.5h.5c.3 0 .5-.2.5-.5v-5.7h2ZM146.8 34c2.2 0 3.3-1.4 3.6-2.6L149 31c-.2.7-.9 1.5-2.2 1.5-1.2 0-2.4-.9-2.4-2.5 0-1.7 1.2-2.6 2.4-2.6 1.3 0 2 .8 2.1 1.6l1.5-.5c-.4-1.2-1.5-2.5-3.6-2.5-2 0-4 1.6-4 4s1.9 4 4 4ZM154.4 30c0-1.7 1.3-2.6 2.4-2.6 1.2 0 2.5.9 2.5 2.6 0 1.7-1.3 2.5-2.5 2.5-1.1 0-2.4-.8-2.4-2.5Zm-1.5 0c0 2.5 1.8 4 4 4 2 0 4-1.5 4-4s-2-4-4-4c-2.2 0-4 1.5-4 4ZM172 33.8c.4 0 .6-.2.6-.5v-6.6c0-.3-.2-.5-.5-.5h-1.2c-.2 0-.4 0-.5.3l-2.2 5.2-2.1-5.2c-.1-.2-.3-.3-.5-.3h-1.2c-.2 0-.5.2-.5.5v6.6c0 .3.3.5.5.5h.5c.3 0 .5-.2.5-.5v-4.8l2 5c.2.2.3.3.5.3h.6c.2 0 .4 0 .5-.3l2-5v4.8c0 .3.3.5.6.5h.5ZM177.7 29.7v-2.2h1.2c.7 0 1.2.4 1.2 1 0 .7-.5 1.2-1.2 1.2h-1.2Zm1.3 1.2c1.6 0 2.6-1 2.6-2.3 0-1.4-1-2.4-2.6-2.4h-2.4c-.2 0-.5.2-.5.5v6.6c0 .3.3.5.5.5h.5c.3 0 .5-.2.5-.5V31h1.4ZM187.4 34c1.7 0 3-1 3-2.9v-4.4c0-.3-.2-.5-.5-.5h-.5c-.3 0-.5.2-.5.5V31c0 1-.6 1.5-1.5 1.5S186 32 186 31v-4.3c0-.3-.2-.5-.5-.5h-.5c-.2 0-.5.2-.5.5V31c0 1.9 1.4 2.9 3 2.9ZM199 27.6c.4 0 .6-.2.6-.5v-.4c0-.3-.2-.5-.5-.5h-5.4c-.3 0-.5.2-.5.5v.4c0 .3.2.5.5.5h2v5.7c0 .3.1.5.4.5h.5c.3 0 .5-.2.5-.5v-5.7h2ZM206.8 33.8c.2 0 .5-.2.5-.5V33c0-.3-.3-.5-.5-.5h-2.9v-1.8h2.5c.3 0 .5-.2.5-.5v-.3c0-.3-.2-.5-.5-.5H204v-1.7h2.9c.2 0 .5-.3.5-.5v-.4c0-.3-.3-.5-.5-.5h-3.9c-.3 0-.5.2-.5.5v6.6c0 .3.2.5.5.5h3.9ZM214.2 33.6l.4.2h.6c.3 0 .6-.4.4-.7l-1.3-2.4c1-.3 1.7-1.1 1.7-2.2 0-1.3-1-2.3-2.5-2.3h-3v7.1c0 .3.2.5.5.5h.5c.3 0 .5-.2.5-.5V31h.8l1.4 2.7Zm-2.2-4v-2.1h1.2c.8 0 1.2.4 1.2 1 0 .7-.4 1-1.2 1H212ZM73 17v-3h1.7c1 0 1.6.6 1.6 1.5s-.6 1.4-1.6 1.4H73Zm1.8.8c1.4 0 2.4-1 2.4-2.3 0-1.3-1-2.3-2.4-2.3H72V21h.9v-3.2h1.8Zm5.5-2.2c-1.5 0-2.6 1.1-2.6 2.8 0 1.6 1 2.8 2.6 2.8 1.5 0 2.6-1.2 2.6-2.8 0-1.7-1-2.8-2.6-2.8Zm0 .8c1 0 1.8.7 1.8 2 0 1.2-.9 2-1.8 2-1 0-1.8-.8-1.8-2 0-1.3.9-2 1.8-2Zm7-.7L86 20l-1.3-4.2h-1l1.8 5.3h1l1.4-4.2 1.4 4.2h1l1.7-5.3h-1L89.7 20l-1.5-4.2h-.9Zm6.2 2.2c0-.8.7-1.5 1.6-1.5 1 0 1.5.6 1.5 1.5h-3.1Zm3.3 1.3c-.2.7-.7 1.2-1.6 1.2-1 0-1.7-.8-1.7-1.8h4v-.3c0-1.6-.8-2.7-2.4-2.7-1.4 0-2.5 1-2.5 2.7 0 1.8 1.2 2.9 2.6 2.9 1.2 0 2-.8 2.3-1.7l-.7-.3Zm5-3.5h-.5c-.5 0-1.2.2-1.6 1v-1H99V21h.9v-2.7c0-1.2.6-1.7 1.5-1.7h.4v-.9Zm1.4 2.2c0-.8.7-1.5 1.6-1.5 1 0 1.5.6 1.5 1.5h-3.1Zm3.3 1.3c-.2.7-.7 1.2-1.6 1.2-1 0-1.7-.8-1.7-1.8h4v-.3c0-1.6-.8-2.7-2.4-2.7-1.4 0-2.5 1-2.5 2.7 0 1.8 1.2 2.9 2.6 2.9 1.2 0 2-.8 2.3-1.7l-.7-.3Zm5.9 1v.8h1l-.1-1v-7h-1v3.6c0-.5-.6-1-1.6-1-1.5 0-2.5 1.2-2.5 2.8 0 1.5 1 2.8 2.5 2.8.9 0 1.5-.6 1.7-1.1v.1Zm-1.6.2c-1 0-1.7-.9-1.7-2 0-1.2.7-2 1.7-2s1.6.8 1.6 2c0 1.1-.7 2-1.6 2Zm7.8.6v-.9c.3.6 1 1 1.8 1 1.6 0 2.5-1.2 2.5-2.7 0-1.6-.9-2.8-2.4-2.8a2 2 0 0 0-1.8 1V13h-1v8h1Zm1.7-.6c-1 0-1.7-.9-1.7-2 0-1.2.7-2 1.7-2s1.7.8 1.7 2c0 1.1-.7 2-1.7 2Zm5 2.7 3.4-7.4h-1l-1.6 3.8-1.7-3.8h-1l2.3 4.7-1.3 2.7h1Z"
                ></path>
            </svg>
        </div>
    }.into_any()
}

#[component]
fn ProfileLoading() -> impl IntoView {
    view! {
        <div class="rounded-full animate-pulse aspect-square size-12 md:size-14 lg:size-16 overflow-clip bg-white/20"></div>
        <div class="flex flex-col gap-2 animate-pulse w-full">
            <div class="w-full h-7  rounded-full bg-white/20"></div>
            <div class="w-full h-4 md:h-5 rounded-full bg-white/20"></div>
        </div>
    }.into_any()
}

#[component]
fn ProfileLoaded(user_details: ProfileDetails) -> impl IntoView {
    view! {
        <img class="size-12 md:size-14 lg:size-16 rounded-full aspect-square object-cover" src=user_details.profile_pic_or_random() />
        <div class="flex flex-col gap-2 w-full">
            <span class="text-lg md:text-xl text-neutral-50 text-ellipsis line-clamp-1 font-semibold">
                @{user_details.username_or_fallback()}
            </span>
            <span class="text-xs md:text-sm text-neutral-400 line-clamp-1">{user_details.principal()}</span>
        </div>
    }
    .into_any()
}

#[component]
pub fn Menu() -> impl IntoView {
    let query_map = use_query_map();
    let show_content_modal = RwSignal::new(false);
    let is_authorized_to_seed_content: AuthorizedUserToSeedContent = expect_context();

    let auth = auth_state();
    let is_connected = auth.is_logged_in_with_oauth();

    // NSFW cookie management
    let (is_nsfw_enabled, set_nsfw_enabled) = use_cookie_with_options::<bool, FromToStringCodec>(
        NSFW_ENABLED_COOKIE,
        UseCookieOptions::default()
            .path("/")
            .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
            .same_site(leptos_use::SameSite::Lax),
    );

    let nsfw_toggle_ref = NodeRef::<Input>::new();
    let nsfw_enabled_signal = Signal::derive(move || is_nsfw_enabled.get().unwrap_or(false));

    // Handle NSFW toggle
    let _ = leptos_use::use_event_listener(nsfw_toggle_ref, leptos::ev::change, move |_| {
        let new_value = !is_nsfw_enabled.get().unwrap_or(false);
        set_nsfw_enabled(Some(new_value));
    });

    // Notifications state management
    let (notifs_enabled, set_notifs_enabled, _) =
        use_local_storage::<bool, FromToStringCodec>(NOTIFICATIONS_ENABLED_STORE);

    let notifs_enabled_signal = Signal::derive(move || {
        notifs_enabled.get()
            && matches!(Notification::permission(), NotificationPermission::Granted)
    });

    let notifs_toggle_ref = NodeRef::<Input>::new();

    // Handle notifications toggle - simplified version
    // Just toggle the local state, full handling happens in settings page
    let _ = leptos_use::use_event_listener(notifs_toggle_ref, leptos::ev::change, move |_| {
        let current_value = notifs_enabled.get();
        set_notifs_enabled(!current_value);
    });

    Effect::new(move |_| {
        let query_params = query_map.get();
        let url = query_params.get("text")?;
        if !url.is_empty() && is_connected.get() {
            show_content_modal.set(true);
        }
        Some(())
    });

    let authorized_fetch_res = auth.derive_resource(
        move || (),
        move |cans, _| {
            send_wrap(async move {
                let user_principal = cans.user_principal();
                match is_authorized_to_seed_content.0.get_untracked() {
                    Some((auth, principal)) if principal == user_principal => {
                        is_authorized_to_seed_content
                            .0
                            .set(Some((auth, user_principal)))
                    }
                    _ => (),
                }

                let content_seed_client: ContentSeedClient = expect_context();

                let res = content_seed_client
                    .check_if_authorized(user_principal)
                    .await
                    .unwrap_or_default();

                is_authorized_to_seed_content
                    .0
                    .set(Some((res, user_principal)));
                Ok(())
            })
        },
    );

    let app_state = use_context::<AppState>();
    let page_title = app_state.unwrap().name.to_owned() + " - Menu";

    let upload_content_mount_point = NodeRef::<Div>::new();

    let view_profile_clicked = move || {
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx()) {
            MixPanelEvent::track_menu_clicked(global, MixpanelMenuClickedCTAType::ViewProfile);
        }
    };

    view! {
        <Title text=page_title />

        <div class="flex flex-col items-center pt-2 pb-12 w-full min-h-screen text-white bg-black divide-y divide-white/10">
            <div class="flex flex-col gap-20 items-center pb-16 w-full">
                <TitleText justify_center=false>
                    <div class="flex flex-row justify-center">
                        <span class="text-2xl font-bold">Menu</span>
                    </div>
                </TitleText>
                <div class="flex flex-col gap-4 items-center w-11/12 lg:w-8/12">
                    // <a on:click=move |_| view_profile_clicked() href="/profile/posts" class="flex flex-row gap-4 justify-center items-center p-4 w-full bg-neutral-900 rounded-lg">
                    //     <Suspense fallback=ProfileLoading>
                    //         {move || Suspend::new(async move {
                    //             let cans = auth.auth_cans().await;
                    //             cans.map(|c| {
                    //                 view! { <ProfileLoaded user_details=c.profile_details() /> }
                    //             })
                    //         })}
                    //     </Suspense>
                    // </a>
                    <Show when=move || !is_connected()>
                        <div class="px-8 w-full md:w-4/12 xl:w-2/12">
                            <div class="w-full">
                                <ConnectLogin />
                            </div>
                        </div>
                        <div class="px-8 w-full font-sans text-sm text-center text-neutral-400">
                            {r#"Your YRAL account is ready. Log in with Google to save your progress and continue seamlessly."#}
                        </div>
                    </Show>
                    <div node_ref=upload_content_mount_point />
                </div>
            </div>
            <div class="flex flex-col gap-6 py-8 px-8 w-full">

                // Settings section
                <SectionHeader text="Settings" />
                <div class="flex flex-col gap-4">
                    <MenuItemWithToggle
                        text="Show NSFW videos"
                        icon=icondata::AiEyeOutlined
                        checked=nsfw_enabled_signal
                        node_ref=nsfw_toggle_ref
                    />
                    <MenuItemWithToggle
                        text="Enable Notifications"
                        icon=icondata::BiCommentDotsRegular
                        checked=notifs_enabled_signal
                        node_ref=notifs_toggle_ref
                    />
                </div>

                <div class="h-px bg-white/10 w-full" />

                // Legal section
                <SectionHeader text="Legal" />
                <div class="flex flex-col gap-4">
                    <MenuItem click_cta_type=MixpanelMenuClickedCTAType::TermsOfService href="/terms-of-service" text="Terms of service" icon=icondata::TbBook2 />
                    <MenuItem click_cta_type=MixpanelMenuClickedCTAType::PrivacyPolicy href="/privacy-policy" text="Privacy Policy" icon=icondata::TbLock />
                    <MenuItem click_cta_type=MixpanelMenuClickedCTAType::AboutUs href="/about-us" text="About Us" icon=icondata::TbInfoCircle />
                </div>

                <div class="h-px bg-white/10 w-full" />

                // Help section
                <SectionHeader text="Help" />
                <div class="flex flex-col gap-4">
                    <MenuItem
                        click_cta_type=MixpanelMenuClickedCTAType::TalkToTheTeam
                        href=domain_specific_href("TELEGRAM")
                        text="Talk to the team"
                        icon=icondata::BiMessageDetailSolid
                        target="_blank"
                    />
                    <button
                        on:click=move |_| {
                            let nav = use_navigate();
                            nav("/settings/delete", Default::default());
                        }
                        class="flex items-center justify-between w-full"
                    >
                        <div class="flex flex-row gap-4 items-center">
                            <Icon attr:class="text-xl" icon=icondata::RiDeleteBinSystemLine />
                            <span class="text-base">Delete Account</span>
                        </div>
                        <Icon attr:class="text-xl" icon=icondata::AiRightOutlined />
                    </button>
                    <Show when=is_connected>
                        <MenuItem click_cta_type=MixpanelMenuClickedCTAType::LogOut href="/logout" text="Logout" icon=icondata::FiLogOut />
                    </Show>
                </div>
            </div>
            <MenuFooter />
        </div>
    }
}
