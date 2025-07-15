use leptos::prelude::*;

#[component]
pub fn YourProfileOverlay() -> impl IntoView {
    view! {
        <div class="flex absolute top-0 left-0 justify-center items-center pt-4 w-full bg-transparent z-4">
            <div class="p-2 text-white rounded-full bg-black/20">
                <div class="flex flex-row gap-1 items-center py-2 px-6 rounded-full">
                    <span class="font-sans font-semibold">Your Profile</span>
                </div>
            </div>
        </div>
    }
}
