use super::ic_symbol::IcSymbol;
use leptos::prelude::*;
use leptos_icons::*;
use utils::host::show_pnd_page;

#[component]
fn FollowItem(#[prop(into)] href: String, #[prop(into)] icon: icondata::Icon) -> impl IntoView {
    view! {
        <a
            href=href
            target="_blank"
            class="h-12 w-12 text-2xl rounded-full grid place-items-center aspect-square border border-primary-600"
        >
            <Icon icon />
        </a>
    }
}

pub fn domain_specific_href(base: &str) -> &'static str {
    match base {
        "TELEGRAM" => {
            if show_pnd_page() {
                consts::social::TELEGRAM_PND
            } else {
                consts::social::TELEGRAM_YRAL
            }
        }
        "TWITTER" => {
            if show_pnd_page() {
                consts::social::TWITTER_PND
            } else {
                consts::social::TWITTER_YRAL
            }
        }
        "DISCORD" => consts::social::DISCORD, // Same for both
        "IC_WEBSITE" => consts::social::IC_WEBSITE, // Same for both
        _ => panic!("Unknown base name"),
    }
}

macro_rules! social_button {
    // Regular (non-domain specific)
    ($name:ident, $icon:expr, $href:ident) => {
        #[component]
        pub fn $name() -> impl IntoView {
            view! {
                <FollowItem href=consts::social::$href icon=$icon />
            }
        }
    };

    // Domain-specific version (true/false flag)
    ($name:ident, $icon:expr, $href:ident, true) => {
        #[component]
        pub fn $name() -> impl IntoView {
            let href = domain_specific_href(stringify!($href));
            view! {
                <FollowItem href=href icon=$icon />
            }
        }
    };
}

social_button!(Telegram, icondata::TbBrandTelegram, TELEGRAM, true);
social_button!(Discord, icondata::BiDiscordAlt, DISCORD);
social_button!(Twitter, icondata::BiTwitter, TWITTER, true);
social_button!(IcWebsite, IcSymbol, IC_WEBSITE);
