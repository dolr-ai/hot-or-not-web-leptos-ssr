use leptos::prelude::*;

#[component]
pub fn SearchBar(on_search: impl Fn(String) + 'static) -> impl IntoView {
    let (search_value, set_search_value) = signal(String::new());

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

                // Input field - simple immediate search for now
                <input
                    type="text"
                    class="w-full pl-10 pr-4 py-3 bg-neutral-900 border border-neutral-800 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-pink-500 transition-colors"
                    placeholder="Search by username"
                    value=move || search_value.get()
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        set_search_value.set(value.clone());
                        // Trigger search immediately for now
                        // TODO: Add debouncing when needed
                        on_search(value);
                    }
                />
            </div>
        </div>
    }
}
