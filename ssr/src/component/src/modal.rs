use super::overlay::ShadowOverlay;
use leptos::prelude::*;
use leptos_icons::*;

#[component]
pub fn Modal(#[prop(into)] show: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    view! {
        <ShadowOverlay show>
            <div class="flex flex-col justify-around items-center py-4 mx-4 max-w-full max-h-full rounded-md cursor-auto px-[20px] bg-neutral-900">
                <div class="flex justify-end items-center w-full">
                    <button
                        on:click=move |_| show.set(false)
                        class="p-1 text-lg text-center text-white rounded-full md:text-xl bg-neutral-600"
                    >
                        <Icon icon=icondata::ChCross />
                    </button>
                </div>
                <div class="pb-4 w-full">{children()}</div>
            </div>
        </ShadowOverlay>
    }.into_any()
}
