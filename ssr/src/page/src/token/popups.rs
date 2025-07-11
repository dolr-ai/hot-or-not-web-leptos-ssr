use component::{buttons::GradientLinkButton, overlay::ActionTrackerPopup};
use leptos::{either::Either, prelude::*};
use leptos_icons::*;
use yral_canisters_common::utils::token::balance::TokenBalance;

#[component]
fn SuccessPopup<ImgIV: IntoView, Img: Fn() -> ImgIV, TxtIV: IntoView, Txt: Fn() -> TxtIV>(
    img: Img,
    text: Txt,
    #[prop(into)] previous_link: Signal<String>,
    #[prop(into)] previous_text: String,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6 items-center w-full h-full">
            {img()} <span class="text-base text-center text-neutral-400">{text()}</span>
            <GradientLinkButton href=previous_link() classes="w-3/4">
                {previous_text}
            </GradientLinkButton>
        </div>
    }
}

#[component]
fn ErrorPopup<HeadIV: IntoView, Head: Fn() -> HeadIV>(
    error: String,
    header: Head,
    #[prop(into)] previous_link: Signal<String>,
    #[prop(into)] previous_text: String,
    close_popup: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-6 items-center w-full h-full">
            <div class="flex flex-row justify-center items-center p-3 text-2xl text-orange-400 bg-amber-100 rounded-full md:text-3xl">
                <Icon icon=icondata::BsExclamationTriangle />
            </div>
            <span class="text-2xl font-bold text-center md:text-3xl">{header()}</span>
            <textarea
                prop:value=error
                disabled
                rows=3
                class="p-2 w-full text-xs text-red-500 resize-none md:w-2/3 md:text-sm bg-black/10"
            ></textarea>
            <button
                on:click=move |_| close_popup.set(true)
                class="py-3 w-full text-lg text-center text-white rounded-full md:text-xl bg-primary-600"
            >
                Retry
            </button>
            <a
                href=previous_link
                class="py-3 w-full text-lg text-center text-black bg-white rounded-full border border-black md:text-xl"
            >
                {previous_text}
            </a>
        </div>
    }
}

#[component]
fn TokenTransferSuccessPopup(
    #[prop(into)] token_name: String,
    amount: TokenBalance,
) -> impl IntoView {
    let amount_str = amount.humanize_float();
    view! {
        <SuccessPopup
            img=|| view! { <img src="/img/hotornot/tick.webp" class="max-w-44" /> }
            text=move || {
                format!("You’ve successfully sent {amount_str} {token_name} to your wallet.")
            }

            previous_link="/wallet"
            previous_text="Back to wallet"
        />
    }
}

#[component]
fn TokenTransferErrorPopup(
    #[prop(into)] error: String,
    #[prop(into)] token_name: String,
    close_popup: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <ErrorPopup
            error
            header=move || {
                view! {
                    Failed to transfer
                    <span class="text-primary-600">{format!(" {token_name} ")}</span>
                    token
                }
            }

            previous_link="/wallet"
            previous_text="Back to wallet"
            close_popup
        />
    }
}

#[component]
pub fn TokenTransferPopup(
    transfer_action: Action<(), Result<TokenBalance, ServerFnError>>,
    #[prop(into)] token_name: Signal<String>,
) -> impl IntoView {
    let close_popup = RwSignal::new(false);

    view! {
        <ActionTrackerPopup
            action=transfer_action
            loading_message="Token transfer in progress"
            classes="bg-neutral-900"
            modal=move |res| match res {
                Ok(amount) => {
                    Either::Left(
                        view! {
                            <TokenTransferSuccessPopup
                                token_name=token_name.get_untracked().clone()
                                amount
                            />
                        },
                    )
                }
                Err(e) => {
                    Either::Right(
                        view! {
                            <TokenTransferErrorPopup
                                error=e.to_string()
                                token_name=token_name.get_untracked().clone()
                                close_popup=close_popup.write_only()
                            />
                        },
                    )
                }
            }

            close=close_popup
        />
    }
}
