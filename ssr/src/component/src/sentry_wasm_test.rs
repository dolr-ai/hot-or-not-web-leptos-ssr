// src/component/sentry_wasm_test.rs
use leptos::prelude::*;
use leptos::server_fn::error::ServerFnError;
use utils::sentry_server_test::trigger_server_error;
use wasm_bindgen::prelude::*;

// Import the WASM functions. Adjust the path if needed.
// If they are part of your main lib crate, you might not need a specific module path.
#[wasm_bindgen]
extern "C" {
    // Make sure these function names match the Rust functions defined above
    fn trigger_wasm_panic();
    fn call_throwing_js_from_wasm();
}


#[component]
pub fn SentryWasmTest() -> impl IntoView {

    let on_panic_click = move |_| {
        log::info!("Button clicked: Triggering WASM panic...");
        // Directly call the exported WASM function
        trigger_wasm_panic();
    };

    let on_js_error_click = move |_| {
        log::info!("Button clicked: Triggering JS error via WASM call...");

        // Define the JS function that will throw the error globally
        // This ensures it's available when the WASM function calls it.
        let _ = js_sys::eval(r#"
            window.executeInternalWasmStuff = function() {
                console.log('---> JS: executeInternalWasmStuff called from WASM');
                throw new Error("Sentry WASM Test: Whoops! Error from JS called by WASM.");
            };
        "#);

        // Call the WASM function which in turn calls the JS function
        call_throwing_js_from_wasm();
    };

    // Action to call the server function
    let trigger_server_error_action = Action::new(|_: &()| async {
        log::info!("Action executing: Triggering server error...");
        // The result here might be an error if the server function itself
        // returns Err, but the panic will likely terminate the request handler
        // before this point. Sentry captures the panic via middleware/tracing.
        match trigger_server_error().await {
            Ok(_) => {
                log::info!("Server function returned Ok (unexpected for panic test)");
            }
            Err(e) => {
                // This log might appear if you use ServerFnError::ServerError return
                log::error!("Server function call returned error: {:?}", e);
            }
        }
    });

    let on_server_error_click = move |_| {
        log::info!("Button clicked: Triggering server error...");
        trigger_server_error_action.dispatch(());
    };

    view! {
        <div>
            <h2>"Sentry WASM Error Test"</h2>
            <p>"Click buttons to trigger errors and check Sentry.io dashboard (Client DSN)."</p>
            <button on:click=on_panic_click class="mr-2">
                "Trigger WASM Panic"
            </button>
            <button on:click=on_js_error_click class="mr-2">
                "Trigger JS Error via WASM"
            </button>
            <p>"Check the browser console and Sentry for errors."</p>

            <h2 class="mt-5">"Sentry Server Error Test"</h2>
            <p>"Click button to trigger server-side error and check Sentry.io dashboard (Server DSN)."</p>
            <button on:click=on_server_error_click>
                "Trigger Server Error (Panic)"
            </button>
            <p>"Check the server logs and Sentry for errors."</p>

            <Show when=move || trigger_server_error_action.value().get().is_some()>
                 <p class="text-red-600">"Server Action Result: " {move || format!("{:?}", trigger_server_error_action.value().get())}</p>
            </Show>
        </div>
    }
}

