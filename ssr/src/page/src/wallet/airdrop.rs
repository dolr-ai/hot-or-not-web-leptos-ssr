use candid::{Nat, Principal};
use component::{
    back_btn::BackButton,
    buttons::{GradientLinkButton, GradientLinkText, HighlightedButton, HighlightedLinkButton},
    overlay::ShadowOverlay,
    spinner::{SpinnerCircle, SpinnerCircleStyled},
};
use leptos::prelude::*;
use leptos_icons::Icon;
use serde::{Deserialize, Serialize};
use state::canisters::auth_state;
use utils::event_streaming::events::CentsAdded;
use yral_canisters_common::utils::token::{TokenMetadata, TokenOwner};

pub mod dolr_airdrop;
pub mod sats_airdrop;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum AirdropStatus {
    Available,
    Claimed,
    WaitFor(web_time::Duration),
}

#[component]
pub fn AirdropPage(meta: TokenMetadata, airdrop_amount: u64) -> impl IntoView {
    let claimed = RwSignal::new(false);

    let buffer_signal = RwSignal::new(false);

    view! {
        <div
            style="background: radial-gradient(circle, rgba(0,0,0,0) 0%, rgba(0,0,0,0) 75%, rgba(50,0,28,0.5) 100%);"
            class="flex overflow-hidden relative flex-col gap-4 justify-center items-center w-screen h-screen text-white font-kumbh"
        >
            <div class="absolute left-5 top-10 z-40 scale-[1.75]">
                <BackButton fallback="/wallet" />
            </div>
            <img
                alt="bg"
                src="/img/airdrop/bg.webp"
                class="object-cover absolute inset-0 w-full h-full z-1 fade-in"
            />

            {move || {
                view! { <AirdropAnimation claimed=claimed.into() logo=meta.logo_b64.clone() /> }
            }}
            <AirdropButton
                claimed
                airdrop_amount
                name=meta.name
                buffer_signal
                token_owner=meta.token_owner
                root=meta.root
            />
        </div>
    }
}

#[component]
fn AirdropButton(
    claimed: RwSignal<bool>,
    airdrop_amount: u64,
    name: String,
    buffer_signal: RwSignal<bool>,
    token_owner: Option<TokenOwner>,
    root: Option<Principal>,
) -> impl IntoView {
    let name_for_action = name.clone();

    let auth = auth_state();
    let airdrop_action = Action::new_local(move |&()| {
        let token_owner_cans_id = token_owner.clone().unwrap().canister_id;
        let name_c = name_for_action.clone();

        async move {
            if claimed.get() && !buffer_signal.get() {
                return Ok(());
            }
            buffer_signal.set(true);
            let cans = auth.auth_cans().await?;
            let token_owner = cans.individual_user(token_owner_cans_id).await;

            token_owner
                .request_airdrop(
                    root.unwrap(),
                    None,
                    Into::<Nat>::into(airdrop_amount) * 10u64.pow(8),
                    cans.user_canister(),
                )
                .await?;

            let user = cans.individual_user(cans.user_canister()).await;
            user.add_token(root.unwrap()).await?;

            if name_c == "COYNS" || name_c == "CENTS" {
                CentsAdded.send_event(auth.event_ctx(), "airdrop".to_string(), airdrop_amount);
            }

            buffer_signal.set(false);
            claimed.set(true);
            Ok::<_, ServerFnError>(())
        }
    });

    let name_c = name.clone();
    view! {
        <div
            style="--duration:1500ms"
            class="flex flex-col gap-4 justify-center items-center px-8 w-full text-xl font-bold fade-in z-2"
        >
            <Show
                clone:name_c
                when=claimed
                fallback=move || {
                    view! {
                        <div class="text-center">
                            {format!("{} {} Airdrop received", airdrop_amount, name.clone())}
                        </div>
                    }
                }
            >
                <div class="text-center">
                    {format!("{} {}", airdrop_amount, name_c.clone())} <br />
                    <span class="font-normal">"added to wallet"</span>
                </div>
            </Show>

            {move || {
                if buffer_signal.get() {
                    view! {
                        <HighlightedButton
                            classes="max-w-96 mx-auto py-[16px] px-[20px]".to_string()
                            alt_style=false
                            disabled=true
                            on_click=move || {}
                        >
                            <div class="max-w-90">
                                <SpinnerCircle />
                            </div>
                        </HighlightedButton>
                    }
                        .into_any()
                } else if claimed.get() {
                    view! {
                        <HighlightedLinkButton
                            alt_style=true
                            disabled=false
                            classes="max-w-96 mx-auto py-[12px] px-[20px]".to_string()
                            href="/wallet".to_string()
                        >
                            "Go to wallet"
                        </HighlightedLinkButton>
                    }
                        .into_any()
                } else {
                    view! {
                        <HighlightedButton
                            classes="max-w-96 mx-auto py-[12px] px-[20px] w-full".to_string()
                            alt_style=false
                            disabled=false
                            on_click=move || {
                                airdrop_action.dispatch(());
                            }
                        >
                            "Claim Now"
                        </HighlightedButton>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AirdropClaimState {
    Claiming,
    Claimed(u64),
    Failed,
}

#[component]
fn AirdropPopUpButton(state: AirdropClaimState) -> impl IntoView {
    match state {
        AirdropClaimState::Claiming => view! {
            <div class="mt-10 mb-16 max-w-100 scale-[4] z-2">
                <SpinnerCircleStyled />
            </div>
        }
        .into_any(),
        AirdropClaimState::Claimed(..) => view! {
            <GradientLinkButton href="/wallet" classes="py-3 w-full z-2">
                View Wallet
            </GradientLinkButton>
        }
        .into_any(),
        AirdropClaimState::Failed => view! {
            <GradientLinkText href="/wallet" classes="py-3 w-full z-2">
                Try Again
            </GradientLinkText>
        }
        .into_any(),
    }
}

#[component]
fn AirdropPopupMessage(#[prop(into)] name: String, state: AirdropClaimState) -> impl IntoView {
    match state {
        AirdropClaimState::Claiming => view! {
            Claim for
            <span class="mx-2 font-bold">{name}</span>
            is being processed
        }
        .into_any(),
        AirdropClaimState::Claimed(amount) => view! {
            <span class="mx-2 font-bold">{format!("{amount} {name}")}</span>
            credited in your wallet
        }
        .into_any(),
        AirdropClaimState::Failed => view! {
            Claim for
            <span class="mx-2 font-bold">{name}</span>
            failed
        }
        .into_any(),
    }
}

#[component]
pub fn AirdropPopup(
    #[prop(into)] name: String,
    #[prop(into)] logo: String,
    claim_state: ReadSignal<AirdropClaimState>,
    airdrop_popup: WriteSignal<bool>,
) -> impl IntoView {
    let _ = claim_state.get();
    view! {
        <div class="flex overflow-hidden relative flex-col gap-4 justify-center items-center px-8 w-full h-full text-white rounded-lg font-kumbh bg-neutral-900">
            <button
                on:click=move |_| airdrop_popup.set(false)
                class="absolute top-5 right-5 z-40 p-2 rounded-full scale-125 bg-neutral-800"
            >
                <Icon icon=icondata::TbX />
            </button>
            <Show when=move || !matches!(claim_state.get(), AirdropClaimState::Failed)>
                <img
                    alt="bg"
                    src="/img/airdrop/bg.webp"
                    id="gradient-image"
                    class="object-cover absolute inset-0 w-full h-full z-1 fade-in"
                />
            </Show>
            {move || {
                view! {
                    <WalletAirdropAnimation state=claim_state.get() logo=logo.clone() />
                    <div class="z-2">
                        <AirdropPopupMessage name=name.clone() state=claim_state.get() />
                    </div>
                    <AirdropPopUpButton state=claim_state.get() />
                }
            }}
        </div>
    }
}

#[component]
fn WalletAirdropAnimation(state: AirdropClaimState, logo: String) -> impl IntoView {
    let logo = StoredValue::new(logo);
    match state {
        AirdropClaimState::Claiming => view! {
            <div class="relative mb-20 max-h-96 z-2">
                <div
                    style="--y: 50px"
                    class="flex flex-col justify-center items-center airdrop-parachute"
                >
                    <img
                        alt="Parachute"
                        src="/img/airdrop/parachute.webp"
                        class="h-auto max-h-72"
                    />

                    <div class="p-px w-16 h-16 rounded-md rounded-full -translate-y-8">
                        <img
                            alt="Airdrop"
                            src=logo.get_value()
                            class="object-cover w-full h-full rounded-md rounded-full fade-in"
                        />
                    </div>
                </div>
                <img
                    alt="Cloud"
                    src="/img/airdrop/cloud.webp"
                    style="--x: -50px"
                    class="absolute left-0 -top-10 max-w-12 airdrop-cloud"
                />
                <img
                    alt="Cloud"
                    src="/img/airdrop/cloud.webp"
                    style="--x: 50px"
                    class="absolute right-10 bottom-10 max-w-16 airdrop-cloud"
                />
            </div>
        }.into_any(),
        AirdropClaimState::Claimed(..) => view! {
            <div class="flex justify-center items-center mt-12 w-full max-h-96 lg:mb-8 h-[30vh] z-2">
                <div class="relative gap-12 h-[22vh] w-[22vh] lg:h-[27vh] lg:w-[27vh]">
                    <div>
                        <img
                            alt="tick"
                            src="/img/hotornot/tick.webp"
                            class="object-cover w-full h-full rounded-md fade-in"
                        />
                    </div>
                    <div class="absolute right-0 bottom-0 p-px w-16 h-16 rounded-full fade-in bg-black">
                        <img
                            alt="Airdrop"
                            src=logo.get_value()
                            class="object-cover w-full h-full rounded-full fade-in"
                        />
                    </div>
                </div>
            </div>
        }.into_any(),
        AirdropClaimState::Failed => view! {
            <div class="relative mb-20 max-h-96 z-2">
                <div style="--y: 50px" class="flex justify-center items-center airdrop-parachute">
                    <div class="p-px w-16 h-16 bg-black rounded-full rotate-45 translate-x-12 translate-y-4">
                        <img
                            alt="Airdrop"
                            src=logo.get_value()
                            class="object-cover w-full h-full rounded-md rounded-full fade-in"
                        />
                    </div>
                    <img
                        alt="Parachute"
                        src="/img/airdrop/fallen-parachute.webp"
                        class="h-auto w-50"
                    />
                </div>
                <img
                    alt="Cloud"
                    src="/img/airdrop/cloud.webp"
                    style="--x: -50px"
                    class="absolute left-0 -top-10 max-w-12 airdrop-cloud grayscale"
                />
            </div>
        }.into_any(),
    }
}

#[component]
fn AirdropAnimation(claimed: Signal<bool>, logo: String) -> impl IntoView {
    let logo_c = logo.clone();
    view! {
        <Show
            when=claimed
            fallback=move || {
                view! {
                    <div class="flex justify-center items-center mt-12 w-full max-h-96 lg:mb-8 h-[30vh] z-2">
                        <div class="relative gap-12 h-[22vh] w-[22vh] lg:h-[27vh] lg:w-[27vh]">
                            <div>
                                <img
                                    alt="tick"
                                    src="/img/hotornot/tick.webp"
                                    class="object-cover w-full h-full rounded-md fade-in"
                                />
                            </div>
                            <div class="absolute -right-4 -bottom-4 p-px w-16 h-16 rounded-md fade-in">
                                <img
                                    alt="Airdrop"
                                    src=logo_c.clone()
                                    class="object-cover w-full h-full rounded-md fade-in"
                                />
                            </div>
                        </div>
                    </div>
                }
            }
        >
            <div class="relative max-h-96 h-[50vh] z-2">
                <div
                    style="--y: 50px"
                    class="flex flex-col justify-center items-center airdrop-parachute"
                >
                    <img
                        alt="Parachute"
                        src="/img/airdrop/parachute.webp"
                        class="h-auto max-h-72"
                    />

                    <div class="p-px w-16 h-16 rounded-md rounded-full -translate-y-8">
                        <img
                            alt="Airdrop"
                            src=logo.clone()
                            class="object-cover w-full h-full rounded-md rounded-full fade-in"
                        />
                    </div>
                </div>
                <img
                    alt="Cloud"
                    src="/img/airdrop/cloud.webp"
                    style="--x: -50px"
                    class="absolute left-0 -top-10 max-w-12 airdrop-cloud"
                />
                <img
                    alt="Cloud"
                    src="/img/airdrop/cloud.webp"
                    style="--x: 50px"
                    class="absolute right-10 bottom-10 max-w-16 airdrop-cloud"
                />
            </div>
        </Show>
    }
}

#[component]
pub fn AnimatedTick() -> impl IntoView {
    view! {
        <div class="w-full h-full perspective-midrange">
            <div class="relative w-full h-full rounded-full scale-110 animate-coin-spin-horizontal transform-3d before:absolute before:h-full before:w-full before:rounded-full before:bg-linear-to-b before:from-[#FFC6F9] before:via-[#C01271] before:to-[#990D55] before:transform-3d before:[transform:translateZ(1px)]">
                <div class="flex absolute justify-center items-center p-12 w-full h-full text-center rounded-full [transform:translateZ(2rem)] bg-linear-to-br from-[#C01272] to-[#FF48B2]">
                    <div class="relative">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            xmlns:xlink="http://www.w3.org/1999/xlink"
                            class="w-full h-full text-current transform-3d [transform:translateZ(10px)]"
                            viewBox="0 -3 32 32"
                            version="1.1"
                        >
                            <g stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
                                <g
                                    transform="translate(-518.000000, -1039.000000)"
                                    fill="currentColor"
                                >
                                    <path d="M548.783,1040.2 C547.188,1038.57 544.603,1038.57 543.008,1040.2 L528.569,1054.92 L524.96,1051.24 C523.365,1049.62 520.779,1049.62 519.185,1051.24 C517.59,1052.87 517.59,1055.51 519.185,1057.13 L525.682,1063.76 C527.277,1065.39 529.862,1065.39 531.457,1063.76 L548.783,1046.09 C550.378,1044.46 550.378,1041.82 548.783,1040.2"></path>
                                </g>
                            </g>
                        </svg>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StatefulAirdropPopup(
    #[prop(into)] name: String,
    #[prop(into)] logo: String,
    claim_state: ReadSignal<AirdropClaimState>,
    airdrop_popup: RwSignal<bool>,
) -> impl IntoView {
    let name = StoredValue::new(name);
    let logo = StoredValue::new(logo);

    view! {
        <ShadowOverlay show=airdrop_popup>
            <div class="flex justify-center items-center py-6 px-4 w-full h-full">
                <div class="overflow-hidden relative items-center w-full max-w-md rounded-md cursor-auto bg-neutral-950 h-[80vh]">
                    <AirdropPopup
                        name=name.get_value()
                        logo=logo.get_value()
                        claim_state
                        airdrop_popup=airdrop_popup.write_only()
                    />
                </div>
            </div>
        </ShadowOverlay>
    }
}

#[component]
pub fn SatsAirdropPopup(
    show: RwSignal<bool>,
    claimed: ReadSignal<bool>,
    amount_claimed: ReadSignal<u64>,
    error: ReadSignal<bool>,
    try_again: Action<bool, Result<u64, ServerFnError>>,
) -> impl IntoView {
    let img_src = move || {
        if claimed.get() {
            "/img/airdrop/sats-airdrop-success.webp"
        } else if error.get() {
            "/img/airdrop/sats-airdrop-failed.webp"
        } else {
            "/img/airdrop/sats-airdrop.webp"
        }
    };

    let is_connected = auth_state().is_logged_in_with_oauth();

    view! {
        <ShadowOverlay show=show>
            <div class="flex justify-center items-center py-6 px-4 w-full h-full">
                <div class="overflow-hidden relative items-center pt-16 w-full max-w-md rounded-md cursor-auto h-fit bg-neutral-950">
                    <img
                        src="/img/common/refer-bg.webp"
                        class="object-cover absolute inset-0 z-0 w-full h-full opacity-40"
                    />
                    <div
                        style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                        class=format!(
                            "absolute z-[1] -left-1/2 bottom-1/3 size-[32rem] {}",
                            if error.get() { "saturate-0" } else { "saturate-100" },
                        )
                    ></div>
                    <div
                        style="background: radial-gradient(circle, rgba(226, 1, 123, 0.4) 0%, rgba(255,255,255,0) 50%);"
                        class=format!(
                            "absolute z-[1] top-8 -right-1/3 size-72 {}",
                            if error.get() { "saturate-0" } else { "saturate-100" },
                        )
                    ></div>
                    <button
                        on:click=move |_| show.set(false)
                        class="flex absolute top-4 right-4 justify-center items-center text-lg text-center text-white rounded-full md:text-xl size-6 bg-neutral-600 z-[2]"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                    <div class="flex flex-col gap-16 justify-center items-center p-12 text-white z-[2]">
                        <img src=img_src class="h-60" />
                        <div class="flex flex-col gap-6 items-center z-[2]">
                            {move || {
                                if claimed.get() {
                                    view! {
                                        <div class="text-center">
                                            <span class="font-semibold">
                                                {amount_claimed} " Bitcoin (SATS)"
                                            </span>
                                            " credited in your wallet"
                                        </div>
                                        <HighlightedButton
                                            alt_style=false
                                            disabled=false
                                            on_click=move || { show.set(false) }
                                        >
                                            "Keep Playing"
                                        </HighlightedButton>
                                    }
                                        .into_any()
                                } else if error.get() {
                                    view! {
                                        <div class="text-center">
                                            "Claim for "
                                            <span class="font-semibold">"Bitcoin (SATS)"</span>
                                            " failed"
                                        </div>
                                        <HighlightedButton
                                            alt_style=true
                                            disabled=false
                                            on_click=move || {
                                                try_again.dispatch(is_connected.get());
                                            }
                                        >
                                            "Try again"
                                        </HighlightedButton>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div class="text-center">
                                            "Claim for "
                                            <span class="font-semibold">"Bitcoin (SATS)"</span>
                                            " is being processed"
                                        </div>
                                        <div class="w-12 h-12">
                                            <SpinnerCircle />
                                        </div>
                                    }
                                        .into_any()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </ShadowOverlay>
    }
}
