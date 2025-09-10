use crate::nav_icons::*;
use candid::Principal;
use codee::string::FromToStringCodec;
use consts::{
    ACCOUNT_CONNECTED_STORE, AUTH_UTIL_COOKIES_MAX_AGE_MS, NSFW_ENABLED_COOKIE,
    USER_CANISTER_ID_STORE, USER_PRINCIPAL_STORE,
};
use leptos::{either::Either, prelude::*};
use leptos_icons::*;
use leptos_router::hooks::use_location;
use leptos_use::{use_cookie, use_cookie_with_options, UseCookieOptions};
use utils::{
    mixpanel::mixpanel_events::{BottomNavigationCategory, MixPanelEvent, MixpanelGlobalProps},
    types::PostParams,
};

#[derive(Clone)]
struct NavItem {
    render_data: NavItemRenderData,
    cur_selected: Signal<bool>,
}

#[derive(Debug, Clone)]
enum NavItemRenderData {
    Icon {
        icon: icondata_core::Icon,
        filled_icon: Option<icondata_core::Icon>,
        href: Signal<String>,
    },
    Upload,
}

fn yral_nav_items() -> Vec<NavItem> {
    let cur_location = use_location();
    let path = cur_location.pathname;
    let (user_principal, _) = use_cookie_with_options::<Principal, FromToStringCodec>(
        USER_PRINCIPAL_STORE,
        UseCookieOptions::default()
            .path("/")
            .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
    );

    vec![
        NavItem {
            render_data: NavItemRenderData::Icon {
                icon: HomeSymbol,
                filled_icon: Some(HomeSymbolFilled),
                href: "/".into(),
            },
            cur_selected: Signal::derive(move || {
                matches!(path.get().as_str(), "/") || path.get().contains("/hot-or-not")
            }),
        },
        NavItem {
            render_data: NavItemRenderData::Icon {
                icon: WalletSymbol,
                filled_icon: Some(WalletSymbolFilled),
                href: "/wallet".into(),
            },
            cur_selected: Signal::derive(move || {
                // is selected only if the user is viewing their own wallet
                let Some(user_principal) = user_principal.get() else {
                    return false;
                };
                path.get().starts_with(&format!("/wallet/{user_principal}"))
            }),
        },
        NavItem {
            render_data: NavItemRenderData::Upload,
            cur_selected: Signal::derive(move || matches!(path.get().as_str(), "/upload")),
        },
        NavItem {
            render_data: NavItemRenderData::Icon {
                icon: ProfileIcon,
                filled_icon: Some(ProfileIconFilled),
                href: "/profile/token".into(),
            },
            cur_selected: Signal::derive(move || {
                // is selected only if the user is viewing their own profile
                let Some(user_principal) = user_principal.get() else {
                    return false;
                };
                path.get()
                    .starts_with(&format!("/profile/{user_principal}"))
            }),
        },
        NavItem {
            render_data: NavItemRenderData::Icon {
                icon: MenuSymbol,
                filled_icon: None,
                href: "/menu".into(),
            },
            cur_selected: Signal::derive(move || matches!(path.get().as_str(), "/menu")),
        },
    ]
}

fn get_nav_items() -> Vec<NavItem> {
    yral_nav_items()
}

#[component]
pub fn NavBar() -> impl IntoView {
    let items = get_nav_items();

    view! {
        <Suspense>
            <div class="flex fixed bottom-0 left-0 z-50 flex-row justify-between items-center px-6 w-full bg-black/80">
                {items
                    .iter()
                    .map(|item| {
                        let cur_selected = item.cur_selected;
                        match item.render_data.clone() {
                            NavItemRenderData::Icon { icon, filled_icon, href } => {
                                Either::Left(
                                    view! { <NavIcon href icon filled_icon cur_selected /> },
                                )
                            }
                            NavItemRenderData::Upload => {
                                Either::Right(view! { <UploadIcon cur_selected /> })
                            }
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </Suspense>
    }
}

#[component]
fn NavIcon(
    #[prop(into)] href: Signal<String>,
    #[prop(into)] icon: icondata_core::Icon,
    #[prop(into)] filled_icon: Option<icondata_core::Icon>,
    #[prop(into)] cur_selected: Signal<bool>,
) -> impl IntoView {
    let (user_principal, _) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);

    let (user_canister, _) = use_cookie::<Principal, FromToStringCodec>(USER_CANISTER_ID_STORE);
    let (is_connected, _) = use_cookie::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
    let (is_nsfw_enabled, _) = use_cookie_with_options::<bool, FromToStringCodec>(
        NSFW_ENABLED_COOKIE,
        UseCookieOptions::default()
            .path("/")
            .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
            .same_site(leptos_use::SameSite::Lax),
    );

    let on_click = move |_| {
        if let (Some(user), Some(canister)) = (
            user_principal.get_untracked(),
            user_canister.get_untracked(),
        ) {
            let connected = is_connected.get_untracked().unwrap_or(false);
            let category = BottomNavigationCategory::try_from(href.get_untracked());
            if let Ok(category_name) = category {
                let global = MixpanelGlobalProps::new(
                    user,
                    canister,
                    connected,
                    is_nsfw_enabled.get_untracked().unwrap_or(false),
                    None,
                );
                MixPanelEvent::track_bottom_navigation_clicked(global, category_name);
            }
        }
    };
    view! {
        <a href=href on:click=on_click class="flex justify-center items-center">
            <Show
                when=move || cur_selected()
                fallback=move || {
                    view! {
                        <div class="py-5">
                            <Icon icon=icon attr:class="text-2xl text-white md:text-3xl" />
                        </div>
                    }
                }
            >

                <div class="py-5 border-t-2 border-t-pink-500">
                    <Icon
                        icon=filled_icon.unwrap_or(icon)
                        attr:class="text-2xl text-white md:text-3xl aspect-square"
                    />
                </div>
            </Show>
        </a>
    }
}

#[component]
fn UploadIcon(#[prop(into)] cur_selected: Signal<bool>) -> impl IntoView {
    let (user_principal, _) = use_cookie::<Principal, FromToStringCodec>(USER_PRINCIPAL_STORE);

    let (user_canister, _) = use_cookie::<Principal, FromToStringCodec>(USER_CANISTER_ID_STORE);
    let (is_connected, _) = use_cookie::<bool, FromToStringCodec>(ACCOUNT_CONNECTED_STORE);
    let (is_nsfw_enabled, _) = use_cookie_with_options::<bool, FromToStringCodec>(
        NSFW_ENABLED_COOKIE,
        UseCookieOptions::default()
            .path("/")
            .max_age(consts::auth::REFRESH_MAX_AGE.as_secs() as i64)
            .same_site(leptos_use::SameSite::Lax),
    );

    let on_click = move |_| {
        if let (Some(user), Some(canister)) = (
            user_principal.get_untracked(),
            user_canister.get_untracked(),
        ) {
            let connected = is_connected.get_untracked().unwrap_or(false);
            let global = MixpanelGlobalProps::new(
                user,
                canister,
                connected,
                is_nsfw_enabled.get_untracked().unwrap_or(false),
                None,
            );
            MixPanelEvent::track_bottom_navigation_clicked(
                global,
                BottomNavigationCategory::UploadVideo,
            );
        }
    };
    view! {
        <a
            href="/upload-options"
            on:click=on_click
            class="flex justify-center items-center text-white rounded-full"
        >
            <Show
                when=move || cur_selected()
                fallback=move || {
                    view! {
                        <Icon
                            icon=icondata::AiPlusOutlined
                            attr:class="p-2 w-10 h-10 bg-transparent rounded-full border-2"
                        />
                    }
                }
            >

                <div class="border-t-2 border-transparent">
                    <Icon
                        icon=icondata::AiPlusOutlined
                        attr:class="p-2 w-10 h-10 rounded-full bg-primary-600 aspect-square"
                    />
                    <div class="absolute bottom-0 w-10 bg-primary-600 blur-md"></div>
                </div>
            </Show>
        </a>
    }
}
