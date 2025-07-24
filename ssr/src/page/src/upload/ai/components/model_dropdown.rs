use leptos::prelude::*;
use leptos_icons::*;
use crate::upload::ai::models::VideoModel;

#[component]
pub fn ModelDropdown(
    selected_model: RwSignal<VideoModel>,
    show_dropdown: RwSignal<bool>,
) -> impl IntoView {
    // Store models in a StoredValue to avoid the closure trait bounds issue
    let models = StoredValue::new(VideoModel::get_models());

    view! {
        <div class="relative w-full">
            <label class="block text-sm font-medium text-white mb-2">Model</label>

            // Selected model display
            <div
                class="flex items-center justify-between p-4 bg-neutral-900 border border-neutral-800 rounded-lg cursor-pointer hover:bg-neutral-800"
                on:click=move |_| show_dropdown.update(|show| *show = !*show)
            >
                <div class="flex items-center gap-3">
                    <div class="w-8 h-8 bg-pink-500 rounded-lg flex items-center justify-center">
                        <span class="text-white font-bold text-sm">"P"</span>
                    </div>
                    <div>
                        <div class="text-white font-medium">{selected_model.get().name}</div>
                        <div class="text-neutral-400 text-sm">{selected_model.get().description}</div>
                    </div>
                </div>
                <Icon
                    icon=icondata::AiDownOutlined
                    attr:class=move || format!("text-white transition-transform {}",
                        if show_dropdown.get() { "rotate-180" } else { "" }
                    )
                />
            </div>

            // Dropdown menu
            <Show when=show_dropdown>
                <div class="absolute top-full left-0 right-0 mt-1 bg-neutral-900 border border-neutral-800 rounded-lg shadow-lg z-10">
                    <For
                        each=move || models.get_value()
                        key=|model| model.id.clone()
                        children=move |model| {
                            let model_id = model.id.clone();
                            let model_name = model.name.clone();
                            let model_description = model.description.clone();
                            let model_duration = model.duration.clone();
                            let model_cost_sats = model.cost_sats;
                            let model_clone = model.clone();

                            let is_selected = Signal::derive(move || selected_model.get().id == model_id);
                            view! {
                                <div
                                    class="flex items-center justify-between p-4 hover:bg-neutral-800 cursor-pointer border-b border-neutral-800 last:border-b-0"
                                    on:click=move |_| {
                                        selected_model.set(model_clone.clone());
                                        show_dropdown.set(false);
                                    }
                                >
                                    <div class="flex items-center gap-3">
                                        <div class=move || format!("w-6 h-6 rounded-full border-2 flex items-center justify-center {}",
                                            if is_selected.get() { "border-pink-500 bg-pink-500" } else { "border-neutral-600" }
                                        )>
                                            <Show when=is_selected>
                                                <div class="w-2 h-2 bg-white rounded-full"></div>
                                            </Show>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            <div class="w-6 h-6 bg-pink-500 rounded flex items-center justify-center">
                                                <span class="text-white font-bold text-xs">"P"</span>
                                            </div>
                                            <div>
                                                <div class="text-white font-medium text-sm">{model_name}</div>
                                                <div class="text-neutral-400 text-xs">{model_description}</div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-4 text-xs">
                                        <div class="flex items-center gap-1">
                                            <Icon icon=icondata::AiClockCircleOutlined attr:class="text-neutral-400" />
                                            <span class="text-neutral-400">{model_duration}</span>
                                        </div>
                                        <div class="flex items-center gap-1">
                                            <span class="text-orange-400">"🪙"</span>
                                            <span class="text-orange-400">{format!("{} SATS", model_cost_sats)}</span>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}