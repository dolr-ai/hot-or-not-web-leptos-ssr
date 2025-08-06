use leptos::prelude::*;
use leptos_icons::*;
use videogen_common::TokenType;

#[component]
pub fn TokenDropdown(
    selected_token: RwSignal<TokenType>,
    show_dropdown: RwSignal<bool>,
    show_free_option: bool,
) -> impl IntoView {
    let tokens = move || {
        let mut token_list = vec![
            (TokenType::Sats, "SATS", "/img/hotornot/sats.svg"),
            (TokenType::Dolr, "DOLR", "/img/common/dolr.svg"),
        ];

        if show_free_option {
            token_list.push((TokenType::Free, "YRAL", "/img/yral/favicon.svg"));
        }

        token_list
    };

    view! {
        <div class="relative">
            <button
                type="button"
                class="flex items-center gap-2 p-2 bg-neutral-700 rounded-lg text-white hover:bg-neutral-600 transition-colors"
                on:click=move |_| show_dropdown.update(|v| *v = !*v)
            >
                <div class="flex items-center gap-2">
                    {move || {
                        let token = selected_token.get();
                        let (_, name, icon_path) = tokens().iter()
                            .find(|(t, _, _)| *t == token)
                            .cloned()
                            .unwrap_or((TokenType::Sats, "SATS", "/img/hotornot/sats.svg"));

                        view! {
                            <>
                                <img
                                    src=icon_path
                                    alt=name
                                    class="w-5 h-5 object-contain"
                                />
                                <span class="font-bold text-sm">{name}</span>
                            </>
                        }
                    }}
                </div>
                <Icon
                    icon=Signal::derive(move || if show_dropdown.get() { icondata::AiUpOutlined } else { icondata::AiDownOutlined })
                    attr:class="text-neutral-400 text-sm"
                />
            </button>

            // Dropdown menu
            <Show when=move || show_dropdown.get()>
                <div class="absolute top-full right-0 mt-1 bg-neutral-900 border border-neutral-800 rounded-lg overflow-hidden z-50 min-w-[140px]">
                    <For
                        each=tokens
                        key=|(token, _, _)| *token
                        children=move |(token, name, icon_path)| {
                            let is_selected = move || selected_token.get() == token;
                            view! {
                                <button
                                    type="button"
                                    class="w-full flex items-center gap-2 px-3 py-2 hover:bg-neutral-800 transition-colors"
                                    class:bg-neutral-800=is_selected
                                    on:click=move |_| {
                                        selected_token.set(token);
                                        show_dropdown.set(false);
                                    }
                                >
                                    <img
                                        src=icon_path
                                        alt=name
                                        class="w-5 h-5 object-contain"
                                    />
                                    <div class="flex-1 text-left">
                                        <div class="font-medium text-white text-sm">{name}</div>
                                    </div>
                                    <Show when=is_selected>
                                        <Icon icon=icondata::AiCheckOutlined attr:class="text-pink-400 text-sm" />
                                    </Show>
                                </button>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
