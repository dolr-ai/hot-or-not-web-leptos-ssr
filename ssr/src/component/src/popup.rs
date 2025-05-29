use super::overlay::ShadowOverlay;
use leptos::prelude::*;
use crate::buttons::HighlightedButton;

#[component]
pub fn Popup(#[prop(into)] show: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    view! {
        <ShadowOverlay show>
            <div class="mx-4 py-4 px-[20px] max-w-full relative max-h-full items-center cursor-auto flex-col flex justify-between bg-neutral-900 rounded-md">
                <div class="pb-4 w-full flex-1">{children()}</div>
                <div class="flex justify-center w-full items-center">
                    <HighlightedButton
                        alt_style=false
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
