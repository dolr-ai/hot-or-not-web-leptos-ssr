use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

#[component]
pub fn TestSentryPage() -> impl IntoView {
    let (error_msg, set_error_msg) = signal(String::new());

    let trigger_rust_panic = move |_| {
        panic!("Test Rust panic from WASM for Sentry!");
    };

    let trigger_js_error = move |_| {
        // This will create a JS error that Sentry should catch
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = ["window"], js_name = "eval")]
            fn eval_js(s: &str);
        }
        
        // This will throw an error that Sentry should catch
        eval_js("throw new Error('Test JS error from Rust/WASM');");
    };

    let trigger_complex_error = move |_| {
        // A more complex error with stack trace
        inner_function_1();
    };

    view! {
        <div class="container mx-auto p-8">
            <h1 class="text-2xl font-bold mb-4">"Sentry Debug Test Page"</h1>
            
            <div class="space-y-4">
                <div>
                    <button
                        on:click=trigger_rust_panic
                        class="bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600"
                    >
                        "Trigger Rust Panic"
                    </button>
                </div>

                <div>
                    <button
                        on:click=trigger_js_error
                        class="bg-orange-500 text-white px-4 py-2 rounded hover:bg-orange-600"
                    >
                        "Trigger JS Error from WASM"
                    </button>
                </div>

                <div>
                    <button
                        on:click=trigger_complex_error
                        class="bg-yellow-500 text-white px-4 py-2 rounded hover:bg-yellow-600"
                    >
                        "Trigger Complex Stack Trace"
                    </button>
                </div>

                <div class="mt-8 p-4 bg-gray-100 rounded">
                    <p class="text-sm">
                        "Click any button above to trigger an error that Sentry should capture."
                    </p>
                    <p class="text-sm mt-2">
                        "With debug symbols enabled, you should see proper Rust source locations in Sentry."
                    </p>
                </div>

                {move || {
                    if !error_msg.get().is_empty() {
                        view! {
                            <div class="mt-4 p-4 bg-red-100 border border-red-400 rounded">
                                <p class="text-red-700">{error_msg.get()}</p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

// Helper functions to create a deeper stack trace
fn inner_function_1() {
    inner_function_2();
}

fn inner_function_2() {
    inner_function_3();
}

fn inner_function_3() {
    panic!("Deep stack trace error from inner_function_3!");
}