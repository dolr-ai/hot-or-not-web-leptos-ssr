use leptos::prelude::*;
use leptos_icons::*;
use videogen_common::TokenType;

#[component]
pub fn TokenDropdown(
    selected_token: RwSignal<TokenType>,
    show_dropdown: RwSignal<bool>,
) -> impl IntoView {
    let tokens = || vec![
        (TokenType::Sats, "SATS", "🪙"),
        (TokenType::Dolr, "DOLR", "💰"),
    ];

    view! {
        <div class="w-full">
            <label class="block text-sm font-medium text-white mb-2">Payment Token</label>
            <div class="relative">
                <button
                    type="button"
                    class="w-full flex items-center justify-between p-4 bg-neutral-900 border border-neutral-800 rounded-lg text-white hover:bg-neutral-800 transition-colors"
                    on:click=move |_| show_dropdown.update(|v| *v = !*v)
                >
                    <div class="flex items-center gap-3">
                        <span class="text-xl">
                            {move || {
                                let token = selected_token.get();
                                tokens().iter()
                                    .find(|(t, _, _)| *t == token)
                                    .map(|(_, _, emoji)| *emoji)
                                    .unwrap_or("🪙")
                            }}
                        </span>
                        <div>
                            <div class="font-medium">
                                {move || {
                                    let token = selected_token.get();
                                    tokens().iter()
                                        .find(|(t, _, _)| *t == token)
                                        .map(|(_, name, _)| *name)
                                        .unwrap_or("SATS")
                                }}
                            </div>
                        </div>
                    </div>
                    <Icon 
                        icon=Signal::derive(move || if show_dropdown.get() { icondata::AiUpOutlined } else { icondata::AiDownOutlined })
                        attr:class="text-neutral-400"
                    />
                </button>

                // Dropdown menu
                <Show when=move || show_dropdown.get()>
                    <div class="absolute top-full left-0 right-0 mt-2 bg-neutral-900 border border-neutral-800 rounded-lg overflow-hidden z-50">
                        <For
                            each=tokens
                            key=|(token, _, _)| *token
                            children=move |(token, name, emoji)| {
                                let is_selected = move || selected_token.get() == token;
                                view! {
                                    <button
                                        type="button"
                                        class="w-full flex items-center gap-3 px-4 py-3 hover:bg-neutral-800 transition-colors"
                                        class:bg-neutral-800=is_selected
                                        on:click=move |_| {
                                            selected_token.set(token);
                                            show_dropdown.set(false);
                                        }
                                    >
                                        <span class="text-xl">{emoji}</span>
                                        <div class="flex-1 text-left">
                                            <div class="font-medium text-white">{name}</div>
                                        </div>
                                        <Show when=is_selected>
                                            <Icon icon=icondata::AiCheckOutlined attr:class="text-pink-400" />
                                        </Show>
                                    </button>
                                }
                            }
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}