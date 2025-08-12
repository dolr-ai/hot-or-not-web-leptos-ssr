use codee::string::JsonSerdeCodec;
use consts::{auth::REFRESH_MAX_AGE, AUTH_JOURNEY_PAGE};
use global_constants::CoinState;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_location;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use utils::{
    event_streaming::events::EventCtx,
    mixpanel::mixpanel_events::{
        BottomNavigationCategory, MixPanelEvent, MixpanelGlobalProps, StakeType,
    },
};

use crate::{buttons::HighlightedButton, login_icons::*, overlay::ShadowOverlay};

#[component]
pub fn LoginNudgePopup(
    show: RwSignal<bool>,
    coin: RwSignal<CoinState>,
    show_login_popup: RwSignal<bool>,
    ev_ctx: EventCtx,
) -> impl IntoView {
    Effect::new(move |_| {
        if show.get() {
            if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                MixPanelEvent::track_unlock_higher_bets_popup_shown(
                    global,
                    "home".into(),
                    coin.get_untracked().to_cents(),
                    StakeType::Sats,
                );
            }
        }
    });

    let loc = use_location();

    let (_, set_auth_journey_page) =
        use_cookie_with_options::<BottomNavigationCategory, JsonSerdeCodec>(
            AUTH_JOURNEY_PAGE,
            UseCookieOptions::default()
                .path("/")
                .max_age(REFRESH_MAX_AGE.as_millis() as i64),
        );

    let login_click_action = Action::new(move |_: &()| {
        show.set(false);
        show_login_popup.set(true);
        let path = loc.pathname.get_untracked();
        let category: BottomNavigationCategory =
            BottomNavigationCategory::try_from(path.clone()).unwrap_or_default();
        set_auth_journey_page.set(Some(category));
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            let page_name = global.page_name();
            MixPanelEvent::track_signup_clicked(global, page_name);
        }
        async {}
    });
    view! {
        <ShadowOverlay show=show>
            <div class="px-4 py-6 w-full h-full flex items-center justify-center">
                <div class="overflow-hidden h-fit max-w-md items-center pt-16 cursor-auto bg-neutral-950 rounded-md w-full relative">
                    <div
                        style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                        class="absolute z-[1] -left-1/2 bottom-1/3 size-[32rem]" >
                    </div>

                    <button
                        on:click=move |_| {
                            show.set(false);
                        }
                        class="text-white rounded-full flex items-center justify-center text-center size-6 text-lg md:text-xl bg-neutral-600 absolute z-[2] top-4 right-4"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>

                    <div class="flex z-[2] relative flex-col items-center gap-6 text-white justify-center p-12">
                        <img src="/img/login/unlock.png" class="h-24" alt="Unlock Icon" />

                        <div class="text-center text-2xl font-semibold">Unlock Higher Bets</div>

                        <div class="text-center text-sm text-neutral-300">
                            "You're just 1 step away from high-stake fun."<br />
                            Log in to unlock 5 YRAL bets!
                        </div>

                        <div class="flex flex-col px-8 items-start gap-4 w-full max-w-xs text-sm">
                            <span class="text-center text-sm text-neutral-300">Why log in?</span>

                            <div class="flex items-center gap-2 text-neutral-200">
                                <Icon icon=Crown />
                                <span>Higher bets, higher rewards</span>
                            </div>
                            <div class="flex items-center gap-2 text-neutral-200">
                                <Icon icon=Dollar />
                                <span>Enable withdrawals</span>
                            </div>
                            <div class="flex items-center gap-2 text-neutral-200">
                                <Icon icon=Airdrop />
                                <span>Daily DOLR / YRAL airdrops</span>
                            </div>
                        </div>


                        <HighlightedButton
                            alt_style=false
                            disabled=false
                            on_click=move || {
                                login_click_action.dispatch(());
                            }
                        >
                            "Login Now"
                        </HighlightedButton>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
