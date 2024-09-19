use std::collections::HashMap;

use leptos::*;

use crate::component::spinner::FullScreenSpinner;
use crate::consts::ICPUMP_LISTING_PAGE_SIZE;
use crate::utils::token::icpump::get_paginated_token_list;
use crate::utils::token::icpump::TokenListItem;

#[component]
pub fn TokenListing(details: TokenListItem) -> impl IntoView {
    view! {
        <div class="max-h-[300px] overflow-hidden h-fit p-2 flex border hover:border-white gap-2 w-full  border-transparent">
            <div class="min-w-32 relative self-start">
            <img class="mr-4 w-32 h-auto" src={details.logo} alt={details.token_name.clone()}/>
            </div>
            <div class="gap-1 grid h-fit">
                <span class="text-sm text-gray-500 font-bold">"$" {details.token_symbol}</span>
                <span class="text-sm text-gray-500">"Name: " {details.token_name}</span>
                <span class="text-sm text-gray-500">{details.description}</span>
                <span class="text-xs text-gray-500">{details.formatted_created_at}</span>
                <span class="text-xs text-gray-500">"Created by: " {details.user_id}</span>
            </div>
        </div>
    }
}

#[component]
pub fn ICPumpListing() -> impl IntoView {
    let page = create_rw_signal(1);
    let token_list: RwSignal<Vec<TokenListItem>> = create_rw_signal(vec![]);
    let end_of_list = create_rw_signal(false);
    let cache = create_rw_signal(HashMap::<u64, Vec<TokenListItem>>::new());

    let act = create_resource(page, move |page| async move {
        if let Some(cached) = cache.with_untracked(|c| c.get(&page).cloned()) {
            return cached.clone();
        }

        get_paginated_token_list(page as u32).await.unwrap()
    });

    view! {

        <div class="flex flex-col justify-center mt-10 mb-10">
            <Suspense fallback=FullScreenSpinner>
                {move || {
                    let _ = act.get().map(|res| {
                            if res.len() < ICPUMP_LISTING_PAGE_SIZE {
                                end_of_list.set(true);
                            }
                            update!(move |token_list, cache| {
                                *token_list = res.clone();
                                cache.insert(page.get_untracked(), res.clone());
                            });

                    });

                    view!{
                            <div class="grid grid-col-1 md:grid-cols-2 lg:grid-cols-3 text-gray-400 gap-4">
                                <For
                                    each=move || token_list.get()
                                    key=|t| t.token_symbol.clone()
                                    children=move |token: TokenListItem| {
                                    view! {
                                        <TokenListing details=token />
                                    }
                                    }
                                />
                            </div>

                            <div class="flex flex-row justify-center mt-5">
                                <button on:click={
                                    move |_| {
                                        page.update(|page| *page -= 1);
                                        end_of_list.set(false);
                                    }
                                }
                                disabled=move || page.get()==1> {"[ << ]"} </button>
                                <span class="mx-2"> {page} </span>
                                <button on:click={
                                    move |_| {
                                        page.update(|page| *page += 1);
                                    }
                                }
                                disabled=move || end_of_list.get()
                                > {"[ >> ]"} </button>
                            </div>
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
pub fn ICPumpLanding() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-black text-white overflow-y-scroll pt-5 pb-12">
            <div class="flex ml-4 space-x-2">
                <div class="text-white"> <a href="https://twitter.com/Yral_app" target="_blank"> [twitter] </a> </div>
                <div class="text-white"> <a href="https://www.instagram.com/yral_app/" target="_blank"> [instagram] </a> </div>
                <div class="text-white"> <a href="https://t.me/+c-LTX0Cp-ENmMzI1" target="_blank"> [telegram] </a> </div>
            </div>
            <div class="flex justify-center items-center">
                <div class="font-bold text-3xl hover:font-extrabold"> <a href="/your-profile?tab=tokens"> [start a new coin] </a> </div>
            </div>
            <ICPumpListing />
        </div>
    }
}
