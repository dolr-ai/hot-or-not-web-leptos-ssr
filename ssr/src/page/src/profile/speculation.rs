use candid::Principal;
use hon_worker_common::GameInfo;
use hon_worker_common::GameResV4WithCanister;
use hon_worker_common::GameResult;
use leptos::html;
use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::use_location;
use yral_canisters_common::cursored_data::vote::VotesWithSatsProviderV4;
use yral_canisters_common::utils::token::balance::TokenBalance;
use yral_metadata_client::MetadataClient;

use super::ic::ProfileStream;
use component::profile_placeholders::NoMoreBetsGraphic;
use state::canisters::unauth_canisters;
use utils::{bg_url, send_wrap};
use yral_canisters_common::utils::{posts::PostDetails, profile::ProfileDetails};

#[component]
pub fn ExternalUser(user: Option<ProfileDetails>) -> impl IntoView {
    let propic = user
        .as_ref()
        .map(|u| u.profile_pic_or_random())
        .unwrap_or_default();
    let name = user
        .as_ref()
        .map(|u| u.display_name_or_fallback())
        .unwrap_or_default();

    view! {
        <div class="flex z-20 flex-row gap-1 items-center px-3 pt-3 w-full h-8 text-ellipsis">
            <div class="w-5 h-5 bg-white rounded-full border-2 border-white shrink-0">
                <img class="object-cover object-center rounded-full" src=propic />
            </div>
            <div class="max-w-full text-xs font-semibold truncate">{name}</div>
        </div>
    }
}

#[component]
pub fn ExternalPost(post: Option<PostDetails>) -> impl IntoView {
    let bg_url = post.map(|p| bg_url(p.uid));
    view! {
        <div class="absolute top-0 left-0 z-10 w-full h-full rounded-md overflow-clip">
            {move || {
                bg_url
                    .clone()
                    .map(|bgurl| {
                        view! { <img class="object-cover w-full h-full" src=bgurl.clone() /> }
                    })
            }}

        </div>
    }
}

#[component]
pub fn FallbackUser() -> impl IntoView {
    view! {
        <div
            class="flex flex-row gap-2 items-center p-2 animate-pulse"
            style:animation-delay="-500ms"
        >
            <div class="w-6 h-6 rounded-full bg-white/20"></div>
            <div class="w-20 h-1 rounded-full bg-white/20"></div>
        </div>
    }
}

#[component]
pub fn Speculation(details: GameResV4WithCanister, _ref: NodeRef<html::Div>) -> impl IntoView {
    // TODO: enable scrolling videos for bets
    let profile_post_url = format!(
        "/post/{}/{}",
        details.post_creator_canister, details.post_id
    );

    let bet_canister = details.post_creator_canister;

    let post_id = details.post_id.clone();
    let post_details = Resource::new(
        move || (bet_canister, post_id.clone()),
        move |(canister_id, post_id)| {
            send_wrap(async move {
                let canister = unauth_canisters();
                canister
                    .get_post_details(canister_id, post_id.to_string())
                    .await
                    .ok()
                    .flatten()
            })
        },
    );

    let profile_details = Resource::new(
        move || bet_canister,
        move |canister_id| {
            send_wrap(async move {
                let canister = unauth_canisters();
                let user = canister.individual_user(canister_id).await;
                let profile_details = user.get_profile_details_v_2().await.ok()?;

                let metadata = canister
                    .get_user_metadata(profile_details.principal_id.to_text())
                    .await
                    .ok()??;

                Some(ProfileDetails::from_canister(
                    canister_id,
                    Some(metadata.user_name),
                    profile_details,
                ))
            })
        },
    );

    let details = StoredValue::new(details);
    let (bet_res, amt, icon) = match details.with_value(|d| d.game_info.clone()) {
        GameInfo::CreatorReward(amt) => (
            "RECEIVED",
            amt,
            view! {
                <div class="flex gap-0.5 justify-center items-center w-full h-6 text-white bg-primary-600">
                    <Icon attr:class="text-sm fill-white" icon=icondata::RiTrophyFinanceFill />
                    <span class="text-xs font-medium">Creator Reward</span>
                </div>
            }.into_any(),

        ),
        GameInfo::Vote { vote_amount: _, game_result } => match game_result {
            GameResult::Win { win_amt } => (
                "RECEIVED",
                win_amt,
                view! {
                    <div class="flex gap-0.5 justify-center items-center w-full h-6 text-white bg-primary-600">
                        <Icon attr:class="text-sm fill-white" icon=icondata::RiTrophyFinanceFill />
                        <span class="text-xs font-medium">You Won</span>
                    </div>
                }.into_any(),
            ),
            GameResult::Loss { lose_amt } => (
                "LOST",
                lose_amt,
                view! {
                    <div class="flex justify-center items-center py-2 w-full h-6 text-xs font-medium text-black bg-white">
                        You Lost
                    </div>
                }.into_any(),
            ),
        },
    };

    let amt_render = TokenBalance::new(amt.into(), 0).humanize_float_truncate_to_dp(0);

    view! {
        <div node_ref=_ref class="relative px-1 w-1/2 md:w-1/3 lg:w-1/4">
            <a
                href=profile_post_url
                class="flex relative flex-col justify-between text-white rounded-md aspect-3/5"
            >
                <Suspense fallback=|| {
                    view! {
                        <div class="absolute top-0 left-0 z-10 w-full h-full rounded-md animate-pulse bg-white/10"></div>
                    }
                }>
                    {move || {
                        post_details
                            .get()
                            .map(|post| {
                                view! { <ExternalPost post /> }
                            })
                    }}

                </Suspense>
                <Suspense fallback=FallbackUser>
                    {move || {
                        profile_details
                            .get()
                            .map(|user| {
                                view! { <ExternalUser user /> }
                            })
                    }}

                </Suspense>
                <div class="flex z-20 flex-col gap-y-5">
                    <div class="flex flex-col px-3">
                        <span class="text-xs font-medium uppercase">{bet_res}</span>
                        <span class="text-sm font-semibold md:text-base">{amt_render}Yral</span>
                    </div>
                    {icon}
                </div>
            </a>
        </div>
    }
}

#[component]
pub fn ProfileSpeculations(user_canister: Principal, user_principal: Principal) -> impl IntoView {
    let _ = user_canister;
    let metadata_client = MetadataClient::default();
    let provider = VotesWithSatsProviderV4::new(user_principal, metadata_client);
    let location = use_location();
    let empty_text = if location
        .pathname
        .get_untracked()
        .starts_with(&format!("/profile/{user_principal}"))
    {
        "You haven't placed any votes yet!"
    } else {
        "Not played any games yet!"
    };
    view! {
        <ProfileStream
            provider
            empty_graphic=NoMoreBetsGraphic
            empty_text
            children=move |details, _ref| {
                view! { <Speculation details _ref=_ref.unwrap_or_default() /> }
            }
        />
    }
}
