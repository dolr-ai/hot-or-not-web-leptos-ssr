use leptos::prelude::*;

#[component]
pub fn PaginationControls(
    has_more: ReadSignal<bool>,
    is_loading: ReadSignal<bool>,
    trigger_load: WriteSignal<()>,
) -> impl IntoView {
    view! {
        <div class="w-full flex justify-center py-8">
            <Show
                when=move || has_more.get()
                fallback=move || view! {
                    <div class="text-neutral-500 text-sm">
                        "No more entries to load"
                    </div>
                }
            >
                <Show
                    when=move || is_loading.get()
                    fallback=move || view! {
                        <button
                            class="px-6 py-3 bg-pink-600 hover:bg-pink-700 text-white font-medium rounded-lg transition-colors"
                            on:click=move |_| trigger_load.set(())
                        >
                            "Load More"
                        </button>
                    }
                >
                    <div class="flex items-center gap-2 text-neutral-400">
                        // Loading spinner
                        <svg
                            class="animate-spin h-5 w-5"
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                        >
                            <circle
                                class="opacity-25"
                                cx="12"
                                cy="12"
                                r="10"
                                stroke="currentColor"
                                stroke-width="4"
                            />
                            <path
                                class="opacity-75"
                                fill="currentColor"
                                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            />
                        </svg>
                        <span>"Loading more..."</span>
                    </div>
                </Show>
            </Show>
        </div>
    }
}
