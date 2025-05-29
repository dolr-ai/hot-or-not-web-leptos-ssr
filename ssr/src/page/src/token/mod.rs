pub mod context;
pub mod create;
pub mod create_token_faq;
pub mod icpump_sunset_popup;
pub mod info;
mod popups;
mod sns_form;
pub mod transfer;
pub mod types;

use leptos::prelude::*;
use leptos_router::params::Params;
use yral_canisters_common::utils::token::RootType;

#[derive(Params, PartialEq, Clone)]
struct TokenParams {
    token_root: RootType,
}

#[derive(Params, PartialEq, Clone)]
struct TokenInfoParams {
    token_root: RootType,
}
