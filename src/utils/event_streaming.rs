use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = gtag)]
    pub fn gtag(cmd: &str, event_name: &str, params: &JsValue);
}

pub fn send_event(event_name: &str, params: &serde_json::Value) {
    gtag("event", event_name, &JsValue::from_serde(params).unwrap());
}
