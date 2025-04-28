use candid::Principal;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[derive(Default, Clone)]
pub struct PostViewState {
    state: RwSignal<BTreeMap<(Principal, u64), u64>>,
}

impl PostViewState {
    pub fn register_global() -> Self {
        let this: Self = Self::default();
        provide_context(this.clone());
        this
    }

    fn get() -> Self {
        let this = use_context();
        match this {
            Some(this) => this,
            None => Self::register_global(),
        }
    }

    pub fn set_view_count(canister_id: Principal, post_id: u64, count: u64) {
        let this = Self::get();
        this.state.update(|state| {
            let count = match state.get(&(canister_id, post_id)) {
                Some(existing_count) => {
                    if *existing_count > count {
                        *existing_count
                    } else {
                        count
                    }
                }
                _ => count,
            };
            state.insert((canister_id, post_id), count);
        });
    }

    pub fn get_view_count(canister_id: Principal, post_id: u64) -> impl Fn() -> u64 {
        let this = Self::get();
        move || {
            this.state
                .get()
                .get(&(canister_id, post_id))
                .map_or(0, |count| *count)
        }
    }

    pub fn get_view_count_untracked(canister_id: Principal, post_id: u64) -> impl Fn() -> u64 {
        let this = Self::get();
        move || {
            this.state
                .get_untracked()
                .get(&(canister_id, post_id))
                .map_or(0, |count| *count)
        }
    }

    pub fn increament_view_count(canister_id: Principal, post_id: u64) -> u64 {
        let count = Self::get_view_count_untracked(canister_id, post_id)() + 1;
        let this = Self::get();
        this.state.update(|state| {
            state.insert((canister_id, post_id), count);
        });
        count
    }

    pub fn decreament_view_count(canister_id: Principal, post_id: u64) -> u64 {
        let count = Self::get_view_count_untracked(canister_id, post_id)() - 1;
        let this = Self::get();
        this.state.update(|state| {
            state.insert((canister_id, post_id), count);
        });
        count
    }
}
