use leptos::prelude::*;
use leptos_icons::*;
use state::canisters::auth_state;
use utils::mixpanel::mixpanel_events::{MixPanelEvent, MixpanelGlobalProps};
use videogen_common::ProviderInfo;

#[component]
pub fn ModelDropdown(
    selected_provider: RwSignal<Option<ProviderInfo>>,
    show_dropdown: RwSignal<bool>,
    providers: Vec<ProviderInfo>,
) -> impl IntoView {
    let auth = auth_state();
    let ev_ctx = auth.event_ctx();

    let providers = StoredValue::new(providers);

    // Create derived signals for the selected provider properties
    let provider_name = Signal::derive(move || {
        selected_provider
            .get()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Select Model".to_string())
    });
    let provider_description = Signal::derive(move || {
        selected_provider
            .get()
            .map(|p| p.description.clone())
            .unwrap_or_else(|| "Choose a model to generate video".to_string())
    });
    let provider_icon =
        Signal::derive(move || selected_provider.get().and_then(|p| p.model_icon.clone()));

    view! {
        <div class="relative w-full">
            <label class="block text-sm font-medium text-white mb-2">Model</label>

            // Selected model display
            <div
                class="flex items-center justify-between p-4 bg-neutral-900 border border-neutral-800 rounded-lg cursor-pointer hover:bg-neutral-800"
                on:click=move |_| show_dropdown.update(|show| *show = !*show)
            >
                <div class="flex items-center gap-3">
                    <Show
                        when=move || provider_icon.get().is_some()
                        fallback=move || view! {
                            <div class="w-8 h-8 bg-pink-500 rounded-lg flex items-center justify-center">
                                <span class="text-white font-bold text-sm">"AI"</span>
                            </div>
                        }
                    >
                        <img
                            src=move || provider_icon.get().unwrap_or_default()
                            alt="Model icon"
                            class="w-8 h-8"
                        />
                    </Show>
                    <div>
                        <div class="text-white font-medium">{provider_name}</div>
                        <div class="text-neutral-400 text-sm">{provider_description}</div>
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
                // max-h-[264px] is calculated for ~3 items (each item ~84px + padding)
                <div class="absolute top-full left-0 right-0 mt-1 bg-[#212121] border border-neutral-800 rounded-lg shadow-lg z-50 py-1 max-h-[264px] overflow-y-auto scrollbar-thin scrollbar-thumb-neutral-600 scrollbar-track-transparent">
                    <For
                        each=move || providers.get_value()
                        key=|provider| provider.id.clone()
                        children=move |provider| {
                            let provider_id = provider.id.clone();
                            let provider_name = provider.name.clone();
                            let provider_description = provider.description.clone();
                            let provider_duration = format_duration(provider.default_duration);
                            let _provider_cost_usd_cents = provider.cost.usd_cents;
                            let provider_icon = provider.model_icon.clone();
                            let is_available = provider.is_available;
                            let provider_clone = provider.clone();

                            let is_selected = Signal::derive(move || {
                                selected_provider
                                    .get()
                                    .map(|p| p.id == provider_id)
                                    .unwrap_or(false)
                            });
                            view! {
                                <div
                                    class=move || format!("flex items-center gap-4 px-4 py-2.5 rounded-lg {}",
                                        if is_available { "hover:bg-neutral-800 cursor-pointer" } else { "opacity-50 cursor-not-allowed" }
                                    )
                                    on:click=move |_| {
                                        if is_available {
                                            selected_provider.set(Some(provider_clone.clone()));
                                            show_dropdown.set(false);

                                            // Track provider selection
                                            if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                                                MixPanelEvent::track_video_generation_model_selected(
                                                    global,
                                                    provider_clone.name.clone()
                                                );
                                            }
                                        }
                                    }
                                >
                                    // Radio button on the left
                                    <div class=move || format!("w-[18px] h-[18px] rounded-full border-2 flex items-center justify-center shrink-0 {}",
                                        if is_selected.get() { "border-pink-500 bg-pink-500" } else { "border-neutral-600" }
                                    )>
                                        <Show when=is_selected>
                                            <div class="w-2 h-2 bg-white rounded-full"></div>
                                        </Show>
                                    </div>

                                    // Provider info container
                                    <div class="flex items-start gap-2.5 flex-1">
                                        // Provider icon
                                        {
                                            match provider_icon.clone() {
                                                Some(icon_path) => view! {
                                                    <img
                                                        src=icon_path
                                                        alt="Model icon"
                                                        class="w-[30px] h-[30px] shrink-0"
                                                    />
                                                }.into_any(),
                                                None => view! {
                                                    <div class="w-[30px] h-[30px] bg-pink-500 rounded flex items-center justify-center shrink-0">
                                                        <span class="text-white font-bold text-sm">"AI"</span>
                                                    </div>
                                                }.into_any()
                                            }
                                        }

                                        // Provider details
                                        <div class="flex flex-col gap-1">
                                            <div class="text-neutral-50 text-sm leading-tight">{provider_name}</div>
                                            <div class="text-neutral-600 text-xs leading-tight">{provider_description}</div>

                                            // Duration and cost badges or Coming Soon
                                            <div class="flex items-center gap-2.5 mt-1">
                                                {
                                                    if is_available {
                                                        view! {
                                                            // Duration badge
                                                            <div class="flex items-center gap-1 px-1 py-1 rounded border border-neutral-700">
                                                                <Icon icon=icondata::AiClockCircleOutlined attr:class="text-neutral-400 w-4 h-4" />
                                                                <span class="text-neutral-400 text-xs">{provider_duration}</span>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            // Coming Soon badge
                                                            <div class="flex items-center gap-1 px-2 py-1 rounded bg-neutral-700/50">
                                                                <span class="text-neutral-500 text-xs font-medium">"Coming Soon"</span>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }
                                            </div>
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

// Helper function to format duration display
fn format_duration(duration_seconds: Option<u8>) -> String {
    match duration_seconds {
        Some(seconds) => {
            if seconds < 60 {
                format!("{seconds} Sec")
            } else {
                format!("{} Min", seconds / 60)
            }
        }
        None => "Variable".to_string(), // For providers where duration depends on input
    }
}
