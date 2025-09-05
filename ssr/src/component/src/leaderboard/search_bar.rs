use leptos::prelude::*;
use leptos_use::use_debounce_fn;

#[component]
pub fn SearchBar(on_search: impl Fn(String) + Clone + 'static) -> impl IntoView {
    let (search_value, set_search_value) = signal(String::new());
    
    // Clone the callback for use in different closures
    let on_search_debounced = on_search.clone();
    
    // Debounced search function - triggers after 800ms of no typing
    let debounced_search = use_debounce_fn(
        move || {
            on_search_debounced(search_value.get());
        },
        800.0,
    );

    view! {
        <div class="relative w-full mb-6">
            <div class="relative">
                // Search icon
                <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
                    <svg
                        class="w-5 h-5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        />
                    </svg>
                </div>

                // Input field with debounced search
                <input
                    type="text"
                    class="w-full pl-10 pr-4 py-3 bg-neutral-900 border border-neutral-800 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-pink-500 transition-colors"
                    placeholder="Search by username"
                    value=move || search_value.get()
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_search_value.set(value.clone());
                        
                        if value.is_empty() {
                            // Clear search immediately when input is empty
                            on_search(String::new());
                        } else {
                            // Trigger debounced search for non-empty input
                            debounced_search();
                        }
                    }
                />
            </div>
        </div>
    }
}
