use component::back_btn::BackButton;
use component::login_modal::LoginModal;
use component::notification_toggle::NotificationToggle;
use component::overlay::ShadowOverlay;
use component::social::*;
use component::spinner::FullScreenSpinner;
use component::title::TitleText;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::components::Redirect;
use leptos_router::hooks::use_navigate;
use leptos_router::{hooks::use_params, params::Params};
use state::canisters::auth_state;
use utils::mixpanel::mixpanel_events::*;
use utils::send_wrap;
use yral_canisters_common::utils::profile::ProfileDetails;

mod delete_user;

#[derive(Params, PartialEq, Clone)]
struct SettingsParams {
    action: String,
}

#[component]
#[allow(dead_code)]
fn MenuItem(
    #[prop(into)] _text: String,
    #[prop(into)] _href: String,
    #[prop(into)] _icon: icondata::Icon,
    #[prop(into, optional)] _target: String,
) -> impl IntoView {
    view! {
        <a href=_href class="grid grid-cols-3 items-center w-full" target=_target>
            <div class="flex flex-row col-span-2 gap-4 items-center">
                <Icon attr:class="text-2xl" icon=_icon />
                <span class="text-wrap">{_text}</span>
            </div>
            <Icon attr:class="text-2xl justify-self-end" icon=icondata::AiRightOutlined />
        </a>
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
    }
}

#[component]
fn ProfileLoading() -> impl IntoView {
    view! {
        <div class="rounded-full animate-pulse basis-4/12 aspect-square overflow-clip bg-white/20"></div>
        <div class="flex flex-col gap-2 animate-pulse basis-8/12">
            <div class="w-full h-4 rounded-full bg-white/20"></div>
            <div class="w-full h-4 rounded-full bg-white/20"></div>
        </div>
    }
}

#[component]
fn ProfileLoaded(user_details: ProfileDetails) -> impl IntoView {
    let auth_state = auth_state();
    let is_connected = auth_state.is_logged_in_with_oauth();

    view! {
        <div class="rounded-full basis-4/12 aspect-square overflow-clip">
            <img class="object-cover w-full h-full" src=user_details.profile_pic_or_random() />
        </div>
        <div
            class="flex flex-col basis-8/12"
            class=("w-12/12", move || !is_connected())
            class=("sm:w-5/12", move || !is_connected())
        >
            <span class="text-xl text-white text-ellipsis line-clamp-1">
                {user_details.display_name_or_fallback()}
            </span>
            <a class="text-primary-600 text-md" href="/profile/posts">
                View Profile
            </a>
        </div>
    }
}

#[component]
fn ProfileInfo() -> impl IntoView {
    let auth = auth_state();
    view! {
        <Suspense fallback=ProfileLoading>
            {move || Suspend::new(async move {
                let res = auth.auth_cans().await;
                match res {
                    Ok(cans) => {
                        let user_details = cans.profile_details();
                        Either::Left(view! { <ProfileLoaded user_details /> })
                    }
                    Err(e) => Either::Right(view! { <Redirect path=format!("/error?err={e}") /> }),
                }
            })}
        </Suspense>
    }
}

#[component]
fn EnableNotifications() -> impl IntoView {
    view! {
        <NotificationToggle
            show_icon=true
            show_label=true
        />
    }
}

#[component]
fn DeleteAccountPopup(show_delete_popup: RwSignal<bool>) -> impl IntoView {
    let auth = auth_state();
    let navigate = use_navigate();
    let (is_deleting, set_is_deleting) = signal(false);
    let global = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx());

    let handle_delete = Action::new(move |&()| {
        set_is_deleting(true);
        if let Some(global) = global.clone() {
            MixPanelEvent::track_delete_account_confirmed(global, "settings".into());
        }
        let navigate = navigate.clone();

        let value = global.clone();
        send_wrap(async move {
            let value = value.clone();
            match auth.user_identity.await {
                Ok(id) => match delete_user::initiate_delete_user(id.id_wire).await {
                    Ok(_) => {
                        if let Some(props) = value.clone() {
                            MixPanelEvent::track_account_deleted(props, "settings".into());
                        }
                        navigate("/logout", Default::default());
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to delete account: {e:?}");
                        navigate(&format!("/error?err={e}"), Default::default());
                    }
                },
                Err(e) => {
                    leptos::logging::error!("Failed to get auth canisters: {e:?}");
                    navigate(&format!("/error?err={e}"), Default::default());
                }
            }
        })
    });

    view! {
        <ShadowOverlay show=show_delete_popup>
            <div class="flex justify-center items-center py-6 px-4 w-full h-full cursor-auto">
                <div class="relative w-full max-w-md rounded-md bg-neutral-900 text-white p-6">
                    <button
                        on:click=move |_| show_delete_popup.set(false)
                        class="absolute top-4 right-4 text-white rounded-full bg-neutral-600 hover:bg-neutral-700 size-6 flex items-center justify-center"
                        disabled=move || is_deleting.get()
                    >
                        <Icon attr::class="w-4 h-4" icon=icondata::ChCross />
                    </button>

                    <h2 class="text-lg font-bold mb-4 text-center">"Delete your account"</h2>

                    <p class="text-sm text-neutral-300 mb-6 text-center">
                        <span class="font-medium">Are you sure you want to delete your account?</span>
                        <br/><br/>
                        This action is permanent and cannot be undone.
                        <br/><br/>
                        "All your data — including your Bitcoin and token balances — will be permanently removed from the platform."
                    </p>

                    <div class="flex justify-center gap-4">
                        <button
                            class="flex-1 px-4 py-2 rounded-md bg-neutral-700 hover:bg-neutral-600 text-white text-sm disabled:opacity-50"
                            on:click=move |_| show_delete_popup.set(false)
                            disabled=move || is_deleting.get()
                        >
                            "No, take me back"
                        </button>
                        <button
                            class="flex-1 px-4 py-2 rounded-md bg-red-600 hover:bg-red-700 text-white text-sm font-semibold disabled:opacity-50 flex items-center justify-center gap-2"
                            on:click=move |_| {
                                handle_delete.dispatch(());
                                leptos::logging::log!("Delete account button clicked");
                            }
                            disabled=move || is_deleting.get()
                        >
                            <Show
                                when=move || is_deleting.get()
                                fallback=|| "Yes, Delete"
                            >
                                <div class="w-4 h-4 rounded-full border-2 border-white border-solid animate-spin border-t-transparent"></div>
                                "Deleting..."
                            </Show>
                        </button>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}

#[component]
fn DeleteAccount(show_popup: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="flex items-center justify-between w-full"
            on:click=move |_| {
                leptos::logging::log!("Delete account button clicked");
                show_popup.set(true);
            }
        >
            <div class="flex flex-row gap-4 items-center flex-1">
                <Icon icon=icondata::RiDeleteBinSystemLine attr:class="text-2xl flex-shrink-0" />
                <span class="text-wrap">Delete account</span>
            </div>
            <Icon attr:class="text-2xl flex-shrink-0 hover:text-primary-600 transition-colors cursor-pointer" icon=icondata::AiRightOutlined />
        </button>
    }
}

#[component]
fn DeleteAccountFlow(show_popup: RwSignal<bool>, is_authenticated: bool) -> impl IntoView {
    let auth = auth_state();
    let global = MixpanelGlobalProps::from_ev_ctx(auth.event_ctx());
    Effect::new(move || {
        if show_popup.get() {
            if let Some(props) = global.clone() {
                MixPanelEvent::track_delete_account_clicked(props, "settings".into());
            }
        }
    });
    view! {
        <Show when=move || show_popup.get()>
            {
                if is_authenticated {
                    Either::Left(view! { <DeleteAccountPopup show_delete_popup=show_popup /> })
                } else {
                    Either::Right(view! { <LoginModal show=show_popup redirect_to=Some("/settings/delete".to_string()) /> })
                }
            }
        </Show>
    }
}

#[component]
pub fn Settings() -> impl IntoView {
    // Handle route parameters
    let params = use_params::<SettingsParams>();
    let action = Signal::derive(move || {
        params
            .get()
            .map(|p| p.action)
            .unwrap_or_else(|_| String::new())
    });

    let show_popup = RwSignal::new(action.get_untracked() == "delete");

    Effect::new(move || {
        let current_action = action.get();
        show_popup.set(current_action == "delete");
    });

    let auth = auth_state();

    let is_authenticated = Resource::new_blocking(
        move || auth.is_logged_in_with_oauth().get(),
        move |is_auth| async move { is_auth },
    );

    view! {
        <Suspense fallback=FullScreenSpinner>
            {move || Suspend::new(async move {
                let is_auth = is_authenticated.await;

                view! {
                    <div class="flex flex-col items-center pt-2 pb-12 w-full min-h-screen text-white bg-black divide-y divide-white/10">
                        <div class="flex flex-col gap-20 items-center pb-16 w-full">
                            <TitleText justify_center=false>
                                <div class="flex flex-row justify-between">
                                    <BackButton fallback="/menu".to_string() />
                                    <span class="text-2xl font-bold">Settings</span>
                                    <div></div>
                                </div>
                            </TitleText>
                        </div>
                        <div class="flex flex-col gap-8 py-12 px-8 w-full text-lg">
                            <EnableNotifications />
                            <DeleteAccount show_popup />
                        </div>
                        <MenuFooter />
                        <DeleteAccountFlow show_popup is_authenticated=is_auth />
                    </div>
                }
            })}
        </Suspense>
    }
}
