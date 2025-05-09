use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub mod device_id;

#[wasm_bindgen(module = "/src/notifications/setup-firebase-messaging-inline.js")]
extern "C" {
    #[wasm_bindgen(js_name = default)]
    fn get_token() -> js_sys::Promise;
}

pub async fn get_token_for_principal(principal_id: String) {
    let token_promise = get_token();
    match JsFuture::from(token_promise).await {
        Ok(token_js) => {
            let token: String = token_js.as_string().unwrap_or_default();
            #[cfg(feature = "ga4")]
            {
                use device_id::send_principal_and_token_offchain;
                log::info!("sending offchain with params: {token}, {principal_id}");
                send_principal_and_token_offchain(token.clone(), principal_id)
                    .await
                    .unwrap();
            }
            // Ok(token)
        }
        Err(err) => {
            log::warn!("Failed to get token: {err:?}");
            // Err(())
        }
    }
}
