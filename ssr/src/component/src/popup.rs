use super::overlay::ShadowOverlay;
use crate::buttons::HighlightedButton;
use leptos::prelude::*;

#[component]
pub fn Popup(#[prop(into)] show: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    view! {
        <ShadowOverlay show>
            <div
                style="min-height: 500px; max-width:40rem;"
                class="flex relative flex-col gap-5 justify-between items-center py-4 mx-auto max-h-full rounded-md cursor-auto px-[20px] bg-neutral-900"
            >
                <div class="flex-1 pb-4 w-full">{children()}</div>
                <div class="flex justify-center items-center px-8 w-full">
                    <HighlightedButton
                        alt_style=true
                        disabled=false
                        on_click=move || show.set(false)
                        classes="w-full".to_string()
                    >
                        "Okay"
                    </HighlightedButton>
                </div>
            </div>
        </ShadowOverlay>
    }
}
