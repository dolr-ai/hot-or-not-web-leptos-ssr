use leptos::prelude::*;

#[component]
pub fn BulletLoader() -> impl IntoView {
    view! {
        <div class="flex justify-center w-full h-full basis-full">
            <div class="flex flex-row gap-2">
                <div class="w-4 h-4 rounded-full animate-bounce bg-white/50"></div>
                <div
                    class="w-4 h-4 rounded-full animate-bounce bg-white/50"
                    style:animation-delay="-300ms"
                ></div>
                <div
                    class="w-4 h-4 rounded-full animate-bounce bg-white/50"
                    style:animation-delay="-500ms"
                ></div>
                <div
                    class="w-4 h-4 rounded-full animate-bounce bg-white/50"
                    style:animation-delay="-700ms"
                ></div>
            </div>
        </div>
    }
}
