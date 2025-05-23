use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/src/sentry/sentry-inline.js")]
extern "C" {
    pub fn set_sentry_user(user_principal: Option<String>);
    pub fn set_sentry_user_canister(user_canister: Option<String>);
}
