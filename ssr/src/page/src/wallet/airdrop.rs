use std::time::{Duration, SystemTime, UNIX_EPOCH};

use candid::{Nat, Principal};
use component::{
    back_btn::BackButton,
    buttons::{HighlightedButton, HighlightedLinkButton},
    spinner::{SpinnerCircle, SpinnerCircleStyled},
};
use consts::SATS_AIRDROP_LIMIT_RANGE;
use hon_worker_common::{ClaimRequest, VerifiableClaimRequest, WORKER_URL};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_location;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use reqwest::Url;
use state::{canisters::authenticated_canisters, server::HonWorkerJwt};
use utils::event_streaming::events::CentsAdded;
use utils::host::get_host;
use yral_canisters_client::individual_user_template::{Result9, SessionType};
use yral_canisters_common::{
    utils::token::{TokenMetadata, TokenOwner},
    Canisters,
};
use yral_identity::Signature;

pub async fn is_airdrop_claimed(user_principal: Principal) -> Result<bool, ServerFnError> {
    let req_url: Url = WORKER_URL.parse().expect("url to be valid");
    let req_url = req_url
        .join(&format!("/last_airdrop_claimed_at/{user_principal}"))
        .expect("url to be valid");

    let response: Option<u64> = reqwest::get(req_url).await?.json().await?;

    // user has never claimed airdrop before
    let Some(last_airdrop_timestamp) = response else {
        return Ok(false);
    };
    let last_airdrop_timestamp: u128 = last_airdrop_timestamp.into();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let duration_24h = Duration::from_secs(24 * 60 * 60).as_millis();

    // user is blocked for 24h since last airdrop claim
    let blocked_window = last_airdrop_timestamp..(last_airdrop_timestamp + duration_24h);
    Ok(blocked_window.contains(&now))
}

#[server(input = server_fn::codec::Json)]
pub async fn claim_sats_airdrop(
    user_canister: Principal,
    request: ClaimRequest,
    signature: Signature,
) -> Result<u64, ServerFnError> {
    use hon_worker_common::WORKER_URL;

    // TODO: yral-auth-v2, we can do this verification with a JWT
    let cans: Canisters<false> = expect_context();
    let user_principal = request.user_principal;

    let user = cans.individual_user(user_canister).await;
    let profile_owner = user.get_profile_details_v_2().await?;
    if profile_owner.principal_id != user_principal {
        log::error!(
            "Not allowed to claim due to principal mismatch: owner={} != receiver={user_principal}",
            profile_owner.principal_id,
        );
        return Err(ServerFnError::new("Not allowed to claim"));
    }

    let sess = user.get_session_type().await?;
    if !matches!(sess, Result9::Ok(SessionType::RegisteredSession)) {
        log::error!("Not allowed to claim due to invalid session: {sess:?}");
        return Err(ServerFnError::new("Not allowed to claim"));
    }

    let is_airdrop_claimed = is_airdrop_claimed(user_principal).await?;

    if is_airdrop_claimed {
        log::error!("Not allowed to claim as user has already claimed airdrop");
        return Err(ServerFnError::new("Not allowed to claim"));
    }

    // small rng should be sufficient for this usecase
    let mut rng = SmallRng::from_os_rng();
    let amount = rng.random_range(SATS_AIRDROP_LIMIT_RANGE);
    let worker_req = VerifiableClaimRequest {
        sender: user_principal,
        amount,
        request,
        signature,
    };

    let req_url: Url = WORKER_URL.parse().expect("url to be valid");
    let req_url = req_url
        .join(&format!("/claim_airdrop/{user_principal}"))
        .expect("url to be valid");
    let client = reqwest::Client::new();
    let jwt = expect_context::<HonWorkerJwt>();
    let res = client
        .post(req_url)
        .json(&worker_req)
        .header("Authorization", format!("Bearer {}", jwt.0))
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(ServerFnError::new(format!(
            "worker error[{}]: {}",
            res.status().as_u16(),
            res.text().await?
        )));
    }

    Ok(amount)
}

#[component]
pub fn AirdropPage(meta: TokenMetadata, airdrop_amount: u64) -> impl IntoView {
    let claimed = RwSignal::new(false);

    let buffer_signal = RwSignal::new(false);

    view! {
        <div
            style="background: radial-gradient(circle, rgba(0,0,0,0) 0%, rgba(0,0,0,0) 75%, rgba(50,0,28,0.5) 100%);"
            class="h-screen w-screen relative items-center justify-center text-white font-kumbh flex flex-col overflow-hidden gap-4"
        >
            <div class="absolute z-40 left-5 top-10 scale-[1.75]">
                <BackButton fallback="/wallet" />
            </div>
            <img
                alt="bg"
                src="/img/airdrop/bg.webp"
                class="absolute inset-0 z-[1] fade-in w-full h-full object-cover"
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
    let cans_res = authenticated_canisters();
    let name_for_action = name.clone();

    let airdrop_action = Action::new_local(move |&()| {
        let cans_res = cans_res;
        let token_owner_cans_id = token_owner.clone().unwrap().canister_id;
        let name_c = name_for_action.clone();
        async move {
            if claimed.get() && !buffer_signal.get() {
                return Ok(());
            }
            buffer_signal.set(true);
            let cans_wire = cans_res.await?;
            let cans = Canisters::from_wire(cans_wire, expect_context())?;
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
                CentsAdded.send_event("airdrop".to_string(), airdrop_amount);
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
            class="fade-in flex text-xl font-bold z-[2] w-full flex-col gap-4 items-center justify-center px-8"
        >
            <Show clone:name_c when=claimed fallback=move ||view! {
                    <div class="text-center">
                        {format!("{} {} Airdrop received", airdrop_amount, name.clone())}
                    </div>
                }>
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

struct PopUpButtonTextRedirection {
    href: String,
    text: String,
}

fn pop_up_button_href(host: String, pathname: String) -> PopUpButtonTextRedirection {
    if pathname.starts_with("/board") {
        PopUpButtonTextRedirection {
            href: "/wallet".to_string(),
            text: "View Wallet".to_string(),
        }
    } else if host.contains("icpump") {
        PopUpButtonTextRedirection {
            href: "/".to_string(),
            text: "Explore more Tokens".to_string(),
        }
    } else if host.contains("pumpdump") {
        PopUpButtonTextRedirection {
            href: "/play".to_string(),
            text: "Continue Playing".to_string(),
        }
    } else if host.contains("yral") {
        PopUpButtonTextRedirection {
            href: "/".to_string(),
            text: "Watch more Videos".to_string(),
        }
    } else {
        PopUpButtonTextRedirection {
            href: "/wallet".to_string(),
            text: "View Wallet".to_string(),
        }
    }
}

#[component]
fn AirdropPopUpButton(
    claimed: RwSignal<bool>,
    name: String,
    buffer_signal: RwSignal<bool>,
) -> impl IntoView {
    let host = get_host();
    let pathname = use_location();
    let name_c = name.clone();
    let name_c2 = name.clone();
    view! {
        <div
            style="--duration:1500ms"
            class="fade-in flex text-xl font-bold z-[2] w-full flex-col gap-4 items-center justify-center px-8"
        >
            <Show when=claimed fallback=move || view! {
                <div class="text-center font-normal"><span class="font-semibold">100 {name_c.clone()}</span> successfully claimed and added to your wallet!</div>
            }.into_view()>
                <div class="text-center">
                    {format!("100 {}", name_c2.clone())} <br />
                    <span class="text-center font-normal">Claim for <span class="font-semibold">100 {name_c2.clone()}</span> is being processed</span>
                </div>
            </Show>
            {move || {
                if buffer_signal.get() {
                    Some(view! {
                        <div class="max-w-100 mt-10 mb-16 scale-[4] ">
                            <SpinnerCircleStyled/>
                        </div>
                    }
                        .into_any())
                } else if claimed.get() {
                    let host = host.clone();
                    let PopUpButtonTextRedirection { href, text } = pop_up_button_href(host, pathname.pathname.get());
                    Some(view! {
                        <div class="mt-10 mb-16">
                            <HighlightedLinkButton
                                alt_style=true
                                disabled=false
                                classes="max-w-96 mx-auto py-[12px] px-[20px] w-full".to_string()
                                href=href
                            >
                                {text}
                            </HighlightedLinkButton>
                        </div>

                    }
                        .into_any())
                } else {
                    None
                }
            }}
        </div>
    }
}

#[component]
pub fn AirdropPopup(
    name: String,
    logo: String,
    buffer_signal: RwSignal<bool>,
    claimed: RwSignal<bool>,
    airdrop_popup: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div
            style="background: radial-gradient(circle, rgba(0,0,0,0) 0%, rgba(0,0,0,0) 75%, rgba(50,0,28,0.5) 100%);"
            class="h-full w-full relative items-center justify-center text-white font-kumbh flex flex-col overflow-hidden gap-4 rounded-lg"
        >
            <button on:click=move |_| airdrop_popup.set(false) class="absolute z-40 right-5 top-5 scale-125 p-2 rounded-full bg-neutral-800">
                <Icon icon=icondata::TbX />
            </button>
            <img
                alt="bg"
                src="/img/airdrop/bg.webp"
                class="absolute inset-0 z-[1] fade-in w-full h-full object-cover"
            />
            <AirdropAnimation claimed=claimed.into() logo=logo.clone()/>
            <AirdropPopUpButton
                claimed
                name
                buffer_signal
            />
        </div>
    }
}

#[component]
fn AirdropAnimation(claimed: Signal<bool>, logo: String) -> impl IntoView {
    let logo_c = logo.clone();
    view! {
        <Show when=claimed fallback=move || view! {
            <div class="h-[30vh] max-h-96 w-full flex items-center justify-center z-[2] lg:mb-8 mt-12">
                <div class="h-[22vh] w-[22vh] lg:h-[27vh] lg:w-[27vh] relative gap-12">
                    <AnimatedTick />
                    <div
                        style="--duration:1500ms; background: radial-gradient(circle, rgba(27,0,15,1) 0%, rgba(0,0,0,1) 100%); box-shadow: 0px 0px 3.43px 0px #FFFFFF29;"
                        class="p-[1px] fade-in absolute w-16 h-16 -bottom-4 -right-4 rounded-md"
                    >
                        <img
                            alt="Airdrop"
                            src=logo_c.clone()
                            class="w-full fade-in rounded-md h-full object-cover"
                        />
                    </div>
                </div>
            </div>
        }>
        <div class="relative h-[50vh] max-h-96 z-[2]">
        <div
            style="--y: 50px"
            class="flex flex-col items-center justify-center airdrop-parachute"
        >
            <img
                alt="Parachute"
                src="/img/airdrop/parachute.webp"
                class="h-auto max-h-72"
            />

            <div
                style="background: radial-gradient(circle, rgb(244 141 199) 0%, rgb(255 255 255) 100%); box-shadow: 0px 0px 3.43px 0px #FFFFFF29;"
                class="p-[1px] w-16 h-16 -translate-y-8 rounded-md"
            >
                <img
                    alt="Airdrop"
                    src=logo.clone()
                    class="w-full fade-in rounded-md h-full object-cover"
                />
            </div>
        </div>
        <img
            alt="Cloud"
            src="/img/airdrop/cloud.webp"
            style="--x: -50px"
            class="max-w-12 absolute -top-10 left-0 airdrop-cloud"
        />
        <img
            alt="Cloud"
            src="/img/airdrop/cloud.webp"
            style="--x: 50px"
            class="max-w-16 absolute bottom-10 right-10 airdrop-cloud"
        />
    </div>
        </Show>
    }
}

#[component]
pub fn AnimatedTick() -> impl IntoView {
    view! {
        <div class="h-full w-full [perspective:800px]">
            <div class="relative h-full w-full scale-110 animate-coin-spin-horizontal rounded-full [transform-style:preserve-3d] before:absolute before:h-full before:w-full before:rounded-full
            before:bg-gradient-to-b before:from-[#FFC6F9] before:via-[#C01271] before:to-[#990D55] before:[transform-style:preserve-3d] before:[transform:translateZ(1px)]">
                <div class="absolute flex h-full w-full items-center justify-center rounded-full text-center [transform:translateZ(2rem)] p-12
                bg-gradient-to-br from-[#C01272] to-[#FF48B2]">
                    <div class="relative">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            xmlns:xlink="http://www.w3.org/1999/xlink"
                            class="h-full w-full text-current [transform-style:preserve-3d] [transform:translateZ(10px)]"
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
