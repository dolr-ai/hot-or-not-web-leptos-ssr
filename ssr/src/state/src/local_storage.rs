use candid::Principal;
use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;

use consts::REFERRER_STORE;

pub fn use_referrer_store() -> (
    Signal<Option<Principal>>,
    WriteSignal<Option<Principal>>,
    impl Fn() + Clone,
) {
    use_local_storage::<Option<Principal>, JsonSerdeCodec>(REFERRER_STORE)
}

#[derive(Clone, Copy)]
pub struct LocalStorageSyncContext {
    pub account_connected: RwSignal<bool>,
}
