use crate::overlay::*;
use leptos::{ev, prelude::*, reactive::wrappers::write::SignalSetter};
use leptos_icons::*;
use state::app_state::AppState;
use utils::{
    host::get_host,
    web::{copy_to_clipboard, share_url},
};

use crate::icons::share_icon::ShareIcon;
#[component]
pub fn ShareContent(
    share_link: String,
    message: String,
    #[prop(into)] show_popup: SignalSetter<bool>,
) -> impl IntoView {
    // let has_share_support = check_share_support();

    let share_link_social = share_link.clone();

    // Encode the message for URLs
    let copy = share_link.clone();
    let copy_clipboard = move |_| {
        copy_to_clipboard(&copy);
    };

    let app_state = use_context::<AppState>();
    view! {
        <div class="flex flex-col gap-6 items-center p-6 w-full h-full bg-white rounded-lg shadow-lg">
            <div class="flex flex-col gap-2 items-center">
                <img
                    class="w-16 h-16 md:w-20 md:h-20"
                    src=format!("/{}/favicon.svg", app_state.clone().unwrap().asset_path())
                    alt=format!("{} Logo", app_state.clone().unwrap().name)
                />

                <span class="text-xl font-semibold text-center md:text-2xl">Share this app</span>
            </div>
            <SocialShare message=message.clone() share_link=share_link_social.clone() />
            <div class="flex overflow-x-auto justify-center items-center px-10 mx-1 space-x-2 w-full rounded-xl border-2 border-neutral-700 h-10 md:h-20">
                <span class="text-lg text-black md:text-xl truncate">{share_link.clone()}</span>
                <button on:click=copy_clipboard>
                    <Icon
                        attr:class="w-6 h-6 text-black cursor-pointer"
                        icon=icondata::BiCopyRegular
                    />
                </button>
            </div>
            <button
                on:click=move |_| show_popup.set(false)
                class="py-4 w-3/4 text-lg text-center text-white rounded-full bg-primary-600"
            >
                Back
            </button>

        </div>
    }.into_any()
}

#[component]
fn SocialShare(share_link: String, message: String) -> impl IntoView {
    let encoded_message = urlencoding::encode(&message);
    // let encoded_link = urlencoding::encode(&profile_link);

    // Facebook share URL using Dialog API
    let fb_url =
        format!("http://www.facebook.com/share.php?u={share_link}&quote={encoded_message}");

    // WhatsApp share URL
    let whatsapp_url = format!("https://wa.me/?text={encoded_message}");

    // Twitter share URL
    let twitter_url = format!("https://twitter.com/intent/tweet?text={encoded_message}");

    let telegram_url = format!("https://telegram.me/share/url?url={}", &share_link);

    // LinkedIn share URL
    let linkedin_url = format!(
        "https://linkedin.com/sharing/share-offsite/?url={}&title={}",
        &share_link, encoded_message
    );

    view! {
        <div class="flex gap-4">
            // Facebook button
            <a href=fb_url target="_blank">
                <Icon
                    attr:class="text-3xl md:text-4xl text-primary-600"
                    icon=icondata::BsFacebook
                />
            </a>

            // Twitter button
            <a href=twitter_url target="_blank">
                <Icon
                    attr:class="text-3xl md:text-4xl text-primary-600"
                    icon=icondata::BsTwitterX
                />
            </a>

            // WhatsApp button
            <a href=whatsapp_url target="_blank">
                <Icon
                    attr:class="text-3xl md:text-4xl text-primary-600"
                    icon=icondata::FaSquareWhatsappBrands
                />
            </a>

            // LinkedIn button
            <a href=linkedin_url target="_blank">
                <Icon
                    attr:class="text-3xl md:text-4xl text-primary-600"
                    icon=icondata::TbBrandLinkedin
                />
            </a>
            <a href=telegram_url target="_blank">
                <Icon
                    attr:class="text-3xl md:text-4xl text-primary-600"
                    icon=icondata::TbBrandTelegram
                />
            </a>
        </div>
    }
}

#[component]
pub fn ShareButtonWithFallbackPopup(
    share_link: String,
    message: String,
    #[prop(optional)] style: String,
) -> impl IntoView {
    let base_url = get_host();
    let show_fallback = RwSignal::new(false);
    let share_link_c = share_link.clone();
    let on_share_click = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        if share_url(&share_link_c).is_none() {
            show_fallback.set(true);
        }
    };

    let class = format!(
        "text-white text-center text-lg md:text-xl flex items-center justify-center {style}",
    );

    view! {
        <button on:click=on_share_click class=class>
            <Icon attr:class="h-6 w-6 text-neutral-300" icon=ShareIcon />
        </button>
        <PopupOverlay show=show_fallback>
            <ShareContent
                share_link=format!("{base_url}{share_link}")
                message=message.clone()
                show_popup=show_fallback
            />
        </PopupOverlay>
    }
    .into_any()
}
