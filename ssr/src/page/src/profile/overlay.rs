use leptos::prelude::*;

#[component]
pub fn YourProfileOverlay() -> impl IntoView {
    view! {
        <div class="flex w-full items-center justify-center pt-4 absolute top-0 left-0 bg-transparent z-4">
            <div class="rounded-full p-2 text-white bg-black/20">
                <div class="flex flex-row items-center gap-1 py-2 px-6 rounded-full">
                    <span class="font-sans font-semibold">Your Profile</span>
                </div>
            </div>
        </div>
    }
}
