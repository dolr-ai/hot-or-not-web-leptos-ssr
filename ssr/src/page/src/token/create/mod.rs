#[cfg(feature = "ssr")]
mod server_impl;
use component::{
    back_btn::BackButton, show_any::ShowAny, title::TitleText,
    token_logo_sanitize::TokenLogoSanitize,
};
use server_fn::codec::Json;
use state::canisters::authenticated_canisters;

use candid::Principal;
use leptos::{
    ev,
    html::{Input, Textarea},
    prelude::*,
};
use leptos_meta::*;
use std::env;
use utils::{
    event_streaming::events::{
        auth_canisters_store, TokenCreationCompleted, TokenCreationFailed, TokenCreationStarted,
    },
    token::{nsfw::NSFWInfo, DeployedCdaoCanisters},
    web::FileWithUrl,
};
use yral_canisters_common::{utils::profile::ProfileDetails, Canisters, CanistersAuthWire};

use leptos::html;
use sns_validation::humanize::parse_tokens;
use sns_validation::{
    humanize::{format_duration, format_percentage, format_tokens},
    pbs::sns_pb::SnsInitPayload,
};

use super::{popups::TokenCreationPopup, sns_form::SnsFormState};

use icp_ledger::AccountIdentifier;

#[server]
async fn is_server_available() -> Result<(bool, AccountIdentifier), ServerFnError> {
    server_impl::is_server_available().await
}

pub struct DeployedCdaoCanistersRes {
    pub deploy_cdao_canisters: DeployedCdaoCanisters,
    pub token_nsfw_info: NSFWInfo,
}

#[server(
    input = Json
)]
async fn deploy_cdao_canisters(
    cans_wire: CanistersAuthWire,
    create_sns: SnsInitPayload,
    profile_details: ProfileDetails,
    canister_id: Principal,
) -> Result<DeployedCdaoCanisters, ServerFnError> {
    let res = server_impl::deploy_cdao_canisters(cans_wire, create_sns.clone()).await;

    match res {
        Ok(c) => {
            TokenCreationCompleted
                .send_event(
                    create_sns,
                    c.deploy_cdao_canisters.root,
                    profile_details,
                    canister_id,
                    c.token_nsfw_info,
                )
                .await;
            Ok(c.deploy_cdao_canisters)
        }
        Err(e) => {
            TokenCreationFailed
                .send_event(e.to_string(), create_sns, profile_details, canister_id)
                .await;
            Err(e)
        }
    }
}

#[component]
fn TokenImage() -> impl IntoView {
    let ctx = expect_context::<CreateTokenCtx>();
    let img_file = RwSignal::new_local(None::<FileWithUrl>);
    let fstate = ctx.form_state;

    // let img_file = RwSignal::new(None::<FileWithUrl>);
    let (logo_b64, set_logo_b64) = slice!(fstate.logo_b64);

    let on_file_input = move |ev: ev::Event| {
        _ = ev.target().and_then(|_target| {
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::JsCast;
                use web_sys::HtmlInputElement;

                let input = _target.dyn_ref::<HtmlInputElement>()?;
                let file = input.files()?.get(0)?;

                img_file.set(Some(FileWithUrl::new(file.into())));
            }
            Some(())
        })
    };

    let file_input_ref: NodeRef<html::Input> = NodeRef::<html::Input>::new();

    let on_edit_click = move |_| {
        // Trigger the file input click
        if let Some(input) = file_input_ref.get() {
            input.click();
            // input.click();
        }
    };

    let border_class = move || match logo_b64.with(|u| u.is_none()) {
        true => "relative w-20 h-20 rounded-full border-2 border-white/20".to_string(),
        _ => "relative w-20 h-20 rounded-full border-2 border-primary-600".to_string(),
    };

    view! {
        <div class="flex flex-col space-y-4  rounded-lg text-white">

            <div class="flex items-center space-x-4">
                <div class=border_class>

                    <div class="flex items-center justify-center w-full h-full rounded-full">
                        <span class="text-xs text-center text-gray-400 font-medium">
                            "Add custom logo"
                        </span>
                    </div>

                    <input
                        type="file"
                        node_ref=file_input_ref
                        on:change=on_file_input
                        id="dropzone-logo"
                        accept="image/*"
                        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
                    />
                    <div class="absolute bottom-0 right-0 p-1 rounded-full bg-white ">
                        <img src="/img/icpump/upload.svg" class="bg-white" />
                    </div>
                    <ShowAny
                        when=move || logo_b64.with(|u| u.is_some())
                        fallback=|| view! { <div></div> }
                    >
                        <img
                            class="absolute top-0 object-conver h-full w-full rounded-full"
                            src=move || logo_b64().unwrap()
                        />
                        <div class="absolute bottom-0 right-0 p-1 rounded-full bg-white ">
                            <button
                                on:click=on_edit_click
                                class="w-4 h-4 flex items-center justify-center rounded-full bg-white"
                            >
                                <img src="/img/icpump/edit.svg" class="bg-white w-4 h-4 rounded-full" />
                            </button>
                        </div>
                    </ShowAny>

                </div>

            </div>
        </div>
        <TokenLogoSanitize img_file=img_file output_b64=set_logo_b64 />
    }
}

macro_rules! input_element {
    (
        textarea,
        $node_ref:ident,
        $value:ident,
        $on_input:ident,
        $placeholder:ident,
        $class:ident,
        $kind:ident
    ) => {
        view! {
            <textarea
                node_ref=$node_ref
                on:input=move |_| $on_input()
                placeholder=$placeholder
                class=move || $class()
            />
        }
    };
    (
        input,
        $node_ref:ident,
        $value:ident,
        $on_input:ident,
        $placeholder:ident,
        $class:ident,
        $kind:ident
    ) => {
        view! {
            <input
                node_ref=$node_ref
                value={$value.unwrap_or_default()}
                on:input=move |_| $on_input()
                placeholder=$placeholder
                class=move || $class()
                type=$kind.unwrap_or_else(|| "text".into())
            />
        }
    };
}

#[allow(dead_code)]
#[allow(unused_variables)]
macro_rules! input_component {
    ($name:ident, $input_element:ident, $input_type:ident, $attrs:expr) => {
        #[component]
        #[allow(dead_code)]
        #[allow(unused_variables)]
        fn $name<T: 'static, U: Fn(T) + 'static + Copy, V: Fn(String) -> Option<T> + 'static + Copy>(
            #[prop(into)] heading: String,
            #[prop(into)] placeholder: String,
            #[prop(optional)] initial_value: Option<String>,
            #[prop(optional, into)] _input_type: Option<String>,
            #[prop(default = false)] _disabled: bool,
            updater: U,
            validator: V,
        ) -> impl IntoView {
            let ctx: CreateTokenCtx = expect_context();
            let error = RwSignal::new(initial_value.is_none());
            let show_error = RwSignal::new(false);
            if error.get_untracked() {
                ctx.invalid_cnt.update(|c| *c += 1);
            }
            let input_ref = NodeRef::<$input_type>::new();
            let on_input = move || {
                let Some(input) = input_ref.get() else {
                    return;
                };
                let value = input.value();
                match validator(value) {
                    Some(v) => {
                        if error.get_untracked() {
                            ctx.invalid_cnt.update(|c| *c -= 1);
                        }
                        error.set(false);
                        updater(v);
                    },
                    None => {
                        show_error.set(true);
                        if error.get_untracked() {
                            return;
                        }
                        error.set(true);
                        ctx.invalid_cnt.update(|c| *c += 1);
                        }
                    }
            };
            Effect::new(move |prev: Option<()>| {
                ctx.on_form_reset.track();
                // Do not trigger on render
                if prev.is_none() {
                    return;
                }
                let cur_show_err = show_error.get_untracked();
                on_input();
                // this is necessary
                // if the user had not previously input anything,
                // we don't want to show an error
                if !cur_show_err {
                    show_error.set(false);
                }
            });

            let input_class =move ||  match show_error() && error() {
                false => format!("w-full p-3  md:p-4 md:py-5 text-white outline-none bg-white/10 border-2 border-solid border-white/20 text-xs  rounded-xl placeholder-neutral-600"),
                _ =>  format!("w-full p-3  md:p-4 md:py-5 text-white outline-none bg-white/10 border-2 border-solid border-red-500 text-xs  rounded-xl placeholder-neutral-600")
            };
            view! {
                <div class="flex flex-col grow gap-y-1 text-sm md:text-base">
                     <span class="text-white font-semibold">{heading.clone()}</span>
                     {input_element! {
                        $input_element,
                        input_ref,
                        initial_value,
                        on_input,
                        placeholder,
                        input_class,
                        _input_type
                     }}
                    <span class="text-red-500 font-semibold">
                        <Show when=move || show_error() && error()>
                                "Invalid "
                        </Show>
                    </span>
                </div>
            }
        }
    }
}

fn non_empty_string_validator(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}

fn non_empty_string_validator_for_u64(s: String) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    s.parse().ok()
}

input_component!(InputBox, input, Input, {});
input_component!(InputArea, textarea, Textarea, rows = 4);
input_component!(InputField, textarea, Textarea, rows = 1);

#[derive(Clone, Copy, Default)]
pub struct CreateTokenCtx {
    form_state: RwSignal<SnsFormState>,
    invalid_cnt: RwSignal<u32>,
    on_form_reset: Trigger,
}

impl CreateTokenCtx {
    pub fn reset() {
        let ctx: Self = expect_context();

        ctx.form_state.set(SnsFormState::default());
        ctx.invalid_cnt.set(0);
    }
}

#[component]
pub fn CreateToken() -> impl IntoView {
    let auth_cans = auth_canisters_store();

    let ctx: CreateTokenCtx = expect_context();

    let set_token_name = move |name: String| {
        ctx.form_state.update(|f| f.name = Some(name));
    };
    let set_token_symbol = move |symbol: String| {
        ctx.form_state.update(|f| f.symbol = Some(symbol));
    };
    let set_token_desc = move |desc: String| {
        ctx.form_state.update(|f| f.description = Some(desc));
    };
    let set_total_distribution = move |total: u64| {
        ctx.form_state.update(|f| {
            (*f).try_update_total_distribution_tokens(
                parse_tokens(&format!("{} tokens", total)).unwrap(),
            );
        });
    };

    let cans_wire_res = authenticated_canisters();

    let create_action = Action::new(move |&()| {
        let cans_wire_res = cans_wire_res;
        async move {
            let cans_wire = cans_wire_res.await.map_err(|e| e.to_string())?;
            let cans = Canisters::from_wire(cans_wire.clone(), expect_context())
                .map_err(|_| "Unable to authenticate".to_string())?;

            let canister_id = cans.user_canister();
            let profile_details = cans.profile_details();

            let sns_form = ctx.form_state.get_untracked();
            let sns_config = sns_form.try_into_config(&cans)?;

            let create_sns = sns_config.try_convert_to_sns_init_payload()?;
            let server_available = is_server_available().await.map_err(|e| e.to_string())?;
            log::debug!(
                "Server details: {}, {}",
                server_available.0,
                server_available.1
            );
            if !server_available.0 {
                return Err("Server is not available".to_string());
            }

            TokenCreationStarted.send_event(create_sns.clone(), auth_cans);

            let _deployed_cans_response =
                deploy_cdao_canisters(cans_wire, create_sns.clone(), profile_details, canister_id)
                    .await
                    .map_err(|e| e.to_string())?;

            Ok(())
        }
    });
    let creating = create_action.pending();

    let create_disabled = Memo::new(move |_| {
        creating()
            || auth_cans.with(|c| c.is_none())
            || ctx.form_state.with(|f| f.logo_b64.is_none())
            || ctx.form_state.with(|f: &SnsFormState| f.name.is_none())
            || ctx
                .form_state
                .with(|f: &SnsFormState| f.description.is_none())
            || ctx.form_state.with(|f| f.symbol.is_none())
            || ctx.invalid_cnt.get() != 0
    });

    view! {
        <Title text="ICPump - Create token" />
        <div class="w-dvw min-h-dvh bg-black pt-4 flex flex-col gap-4" style="padding-bottom:6rem">
            <TitleText justify_center=false>
                <div class="flex justify-between w-full">
                    <div></div>
                    <span class="font-bold justify-self-center">Create Meme Token</span>
                    <a href="/token/create/faq">
                        <img src="/img/icpump/info.svg" />
                    </a>
                </div>
            </TitleText>
            <div class="flex flex-col w-full px-6 md:px-8 gap-2 md:gap-8">
                <div class="flex flex-row w-full gap-4  justify-between items-center">
                    <TokenImage />
                    <InputBox
                        heading="Token name"
                        placeholder="Add a name to your crypto currency"
                        updater=set_token_name
                        validator=non_empty_string_validator
                        initial_value=ctx
                            .form_state
                            .with_untracked(|f| f.name.clone())
                            .unwrap_or_default()
                    />
                </div>
                <InputArea
                    heading="Description"
                    placeholder="Fun & friendly internet currency inspired by the legendary Shiba Inu dog 'Kabosu'"
                    updater=set_token_desc
                    validator=non_empty_string_validator
                    initial_value=ctx
                        .form_state
                        .with_untracked(|f| f.description.clone())
                        .unwrap_or_default()
                />

                <InputBox
                    heading="Token Symbol"
                    placeholder="Eg. DODGE"
                    updater=set_token_symbol
                    validator=non_empty_string_validator
                    initial_value=ctx
                        .form_state
                        .with_untracked(|f| f.symbol.clone())
                        .unwrap_or_default()
                />

                <InputBox
                    heading="Distribution"
                    placeholder="Distribution Tokens"
                    _input_type="number"
                    updater=set_total_distribution
                    // initial_value="100000000".into()
                    initial_value=(ctx
                        .form_state
                        .with_untracked(|f| {
                            f.total_distrubution().e8s.unwrap_or_else(|| 1000000 * 1e8 as u64)
                                / 1e8 as u64
                        }))
                        .to_string()
                    validator=non_empty_string_validator_for_u64
                />

                <div class="w-full flex justify-center">
                    <button
                        on:click=move |_| {create_action.dispatch(());}
                        disabled=create_disabled
                        class="text-white disabled:text-neutral-500 md:text-xl py-4 md:py-4 font-bold w-full md:w-1/2 lg:w-1/3 rounded-full bg-primary-600 disabled:bg-primary-500/30"
                    >
                        {move || if creating() { "Creating..." } else { "Create" }}
                    </button>
                </div>

                <div class="w-full flex justify-center underline text-sm text-white my-4 ">
                    <a href="/token/create/settings">View advanced settings</a>
                </div>
            </div>
            <TokenCreationPopup
                creation_action=create_action
                img_url=Signal::derive(move || {
                    ctx.form_state.with(|f| f.logo_b64.clone()).unwrap()
                })

                token_name=Signal::derive(move || {
                    ctx.form_state.with(|f| f.name.clone()).unwrap_or_default()
                })
            />

        </div>
    }.into_any()
}

#[component]
fn CommingSoonBanner() -> impl IntoView {
    view! {
        <div class="with-gradient relative bg-black rounded-md p-3 bg-no-repeat bg-cover bg-left" style="background-image: url('/img/gradient-backdrop.png')">
            <div class="flex flex-col gap-1">
                <h2 class="text-white font-bold">Want to raise ICP?</h2>
                <span class="text-font bg-clip-text text-transparent bg-gradient-to-r from-[#FFFFFF80] to-[#882B5E80]">Coming Soon...</span>
            </div>
            <img src="/img/coming-soon-art.svg" class="absolute right-0 bottom-0" />
        </div>
    }
}

#[component]
fn AdvanceSettingCard(
    #[prop(into)] heading: String,
    #[prop(into)] description: String,
    #[prop(into)] value: String,
) -> impl IntoView {
    view! {
        <div class="bg-neutral-900 flex flex-col rounded-md p-3 gap-1">
            <div class="flex justify-between">
                <h2 class="text-neutral-400">{heading}</h2>
                <div class="relative group">
                    <div class="tooltip-trigger cursor-pointer bg-neutral-800 rounded-full size-6 grid">
                        <span class="text-neutral-400 text-center text-xs grid place-items-center font-bold">i</span>
                    </div>
                    <div class="w-max max-w-[85vw] md:max-w-[400px] absolute pointer-events-none duration-150 rounded-md top-0 right-0 mt-8 z-50 opacity-0 group-hover:opacity-100 bg-[#EAC9DB] text-[#A00157] p-4">
                        {description}
                        <div class="absolute right-0 mr-1 -translate-x-1/2 bottom-full h-0 w-0 border-r-4 border-l-4 border-b-4 border-l-transparent border-r-transparent border-b-[#EAC9DB]"></div>
                    </div>
                </div>
            </div>
            <div class="text-neutral-600">{value}</div>
        </div>
    }
}

#[component]
fn AdvanceSettings(
    #[prop()] items: Vec<(String, String, String)>, // heading, value, description
) -> impl IntoView {
    view! {
        <div id="advanced-settings" class="flex flex-col gap-3 pb-8">
            {
                items.into_iter().map(|(h, v, d)| {
                    view! { <AdvanceSettingCard heading=h value=v description=d /> }
                }).collect_view()
            }
        </div>
    }
    .into_any()
}

#[component]
pub fn CreateTokenSettings() -> impl IntoView {
    let fallback_url = "/token/create";
    let ctx: CreateTokenCtx = use_context().unwrap_or_else(|| {
        let ctx = CreateTokenCtx::default();
        provide_context(ctx);
        ctx
    });
    let fstate = ctx.form_state;

    let (transaction_fee, _) = slice!(fstate.transaction_fee);
    let (rejection_fee, _) = slice!(fstate.proposals.rejection_fee);

    let (initial_voting_period, _) = slice!(fstate.proposals.initial_voting_period);
    let (max_wait_deadline_extension, _) =
        slice!(fstate.proposals.maximum_wait_for_quiet_deadline_extension);
    let (min_creation_stake, _) = slice!(fstate.neurons.minimum_creation_stake);
    let (min_dissolve_delay, _) = slice!(fstate.voting.minimum_dissolve_delay);
    let (age, _) = slice!(fstate.voting.maximum_voting_power_bonuses.age.duration);

    let (age_bonus, _) = slice!(fstate.voting.maximum_voting_power_bonuses.age.bonus);
    let (min_participants, _) = slice!(fstate.swap.minimum_participants);

    let (min_direct_participants_icp, _) = slice!(fstate.swap.minimum_direct_participation_icp);
    let (max_direct_participants_icp, _) = slice!(fstate.swap.maximum_direct_participation_icp);
    let (min_participants_icp, _) = slice!(fstate.swap.minimum_participant_icp);
    let (max_participants_icp, _) = slice!(fstate.swap.maximum_participant_icp);

    // let set_restricted_country = move |value: String| {
    //     ctx.form_state.update(|f| {
    //         f.sns_form_setting.restricted_country = Some(value);
    //     });
    // };

    let items = vec![
        (
            "Transaction Fee (e8s)".into(),
            transaction_fee.get_untracked().e8s.unwrap_or(1).to_string(),
            "Fee for sending, receiving token post creation (canister to canister sending)".into(),
        ),
        (
            "Rejection Fee (Token)".into(),
            format_tokens(&rejection_fee.get_untracked()),
            "Fee for proposal rejection once we raised the SNS proposal".into(),
        ),
        (
            "Initial Voting Period (days)".into(),
            format_duration(&initial_voting_period.get_untracked()),
            "Duration for which the proposal remains live".into(),
        ),
        (
            "Maximum wait for quiet deadline extention (days)".into(),
            format_duration(&max_wait_deadline_extension.get_untracked()),
            "Till how far into the sns swap process you can increase the duration for the swap"
                .into(),
        ),
        (
            "Minimum creation stake (token)".into(),
            format_tokens(&min_creation_stake.get_untracked()),
            "Minimum amount of tokens (e8s) to stake in each neuron".into(),
        ),
        (
            "Minimum dissolve delay (months)".into(),
            format_duration(&min_dissolve_delay.get_untracked()),
            "Time commitment you give that by when will you get the liquid token".into(),
        ),
        (
            "Age (duration in years)".into(),
            format_duration(&age.get_untracked()),
            "Age at which participants will earn full bonus".into(),
        ),
        (
            "Age (bonus %)".into(),
            format_percentage(&age_bonus.get_untracked()),
            "% reward post max. age is hit".into(),
        ),
        (
            "Minimum participants".into(),
            min_participants.get_untracked().to_string(),
            "Min number of participant required for execution of SNS".into(),
        ),
        (
            "Minimum direct participant icp".into(),
            min_direct_participants_icp
                .with_untracked(|p| p.as_ref().map(format_tokens))
                .unwrap_or_default(),
            "Min token when direct participant is taking part in swap".into(),
        ),
        (
            "Maximum direct participant icp".into(),
            max_direct_participants_icp
                .with_untracked(|p| p.as_ref().map(format_tokens))
                .unwrap_or_default(),
            "Max token when direct participant is taking part in swap".into(),
        ),
        (
            "Minimum participant icp".into(),
            format_tokens(&min_participants_icp.get_untracked()),
            "Min. ICP taken from treasury of YRAL".into(),
        ),
        (
            "Maximum participant icp".into(),
            format_tokens(&max_participants_icp.get_untracked()),
            "Max. ICP token from treasury of YRAL".into(),
        ),
    ];

    view! {
        <Title text="ICPump - Create token" />
        <div
            class="w-dvw min-h-dvh bg-black pt-4 flex flex-col gap-3 px-4"
            style="padding-bottom:5rem;"
        >
            <TitleText justify_center=false>
                <div class="flex justify-between w-full" style="background: black">
                    <BackButton fallback=fallback_url />
                    <span class="font-bold justify-self-center">Settings</span>
                    <a href="/token/create/faq">
                        <img src="/img/icpump/info.svg" />
                    </a>
                </div>
            </TitleText>
            <CommingSoonBanner />
            <AdvanceSettings items=items />
        </div>
    }
    .into_any()
}
