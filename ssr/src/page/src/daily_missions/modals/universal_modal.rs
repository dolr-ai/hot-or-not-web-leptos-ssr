use super::{CloseButton, ModalOverlay};
use leptos::prelude::*;

pub struct ButtonConfig {
    pub text: String,
    pub style: ButtonStyle,
    pub on_click: Box<dyn Fn() + 'static + Send + Sync>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonStyle {
    Primary,   // Gradient pink-to-purple button
    Secondary, // White button with purple text
}

#[component]
pub fn UniversalModal(
    show: RwSignal<bool>,
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(optional)] progress_bar: Option<(u32, u32)>,
    svg_content: fn() -> AnyView,
    buttons: Vec<ButtonConfig>,
    on_close: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <ModalOverlay show=show>
            <div class="relative w-80 mx-4 bg-gradient-to-b from-purple-900 to-purple-800 rounded-2xl p-6 text-white shadow-2xl">
                <CloseButton on_close=on_close />

                // Centered SVG Icon
                <div class="flex justify-center mb-6 mt-4">
                    {(svg_content)()}
                </div>

                // Title
                <h2 class="text-2xl font-bold text-center mb-4">
                    {title}
                </h2>

                // Optional Progress Bar
                {progress_bar.map(|(current, total)| {
                    view! {
                        <div class="mb-4">
                            <div class="flex gap-1 w-full">
                                {(0..total as i32).map(|i| {
                                    let is_active = (i as u32) < current;
                                    view! {
                                        <div class=format!(
                                            "flex-1 h-2 rounded-full {}",
                                            if is_active {
                                                "bg-gradient-to-r from-green-400 to-green-500"
                                            } else {
                                                "bg-neutral-700"
                                            }
                                        )></div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                })}

                // Description
                <div
                    class="text-center text-purple-100 mb-6 leading-relaxed"
                    inner_html=description
                ></div>

                // Buttons
                <div class="space-y-3">
                    {buttons.into_iter().map(|button| {
                        let button_class = match button.style {
                            ButtonStyle::Primary => {
                                "w-full bg-gradient-to-r from-pink-500 to-purple-600 text-white font-semibold py-3 px-6 rounded-xl hover:from-pink-600 hover:to-purple-700 transition-all duration-200 flex items-center justify-center gap-2"
                            },
                            ButtonStyle::Secondary => {
                                "w-full bg-white text-purple-900 font-semibold py-3 px-6 rounded-xl hover:bg-gray-100 transition-colors"
                            }
                        };

                        let on_click = button.on_click;
                        view! {
                            <button
                                class=button_class
                                on:click=move |_| (on_click)()
                            >
                                {if button.style == ButtonStyle::Primary {
                                    view! {
                                        <span>{button.text}</span>
                                        <div class="w-5 h-5 bg-yellow-400 rounded-full flex items-center justify-center">
                                            <span class="text-xs text-yellow-900 font-bold">!</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span>{button.text}</span> }.into_any()
                                }}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </ModalOverlay>
    }
}

// SVG Icon Components
pub mod icons {
    use leptos::prelude::*;

    // Video Cards Icon
    pub fn video_cards_icon() -> AnyView {
        view! {
            <div class="relative">
                // Back cards
                <div class="absolute -left-3 -top-2 w-16 h-12 bg-gradient-to-br from-pink-400 to-pink-500 rounded-lg transform rotate-12 opacity-80"></div>
                <div class="absolute -right-2 -top-1 w-16 h-12 bg-gradient-to-br from-blue-400 to-blue-500 rounded-lg transform -rotate-6 opacity-60"></div>

                // Main video card
                <div class="relative w-20 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-lg flex items-center justify-center shadow-lg">
                    <div class="w-8 h-8 bg-gradient-to-br from-pink-500 to-red-500 rounded-full flex items-center justify-center">
                        <svg class="w-4 h-4 text-white ml-0.5" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M8 5v14l11-7z"/>
                        </svg>
                    </div>
                </div>
            </div>
        }.into_any()
    }

    // Video Cards with Checkmark
    pub fn video_cards_complete_icon() -> AnyView {
        view! {
            <div class="relative">
                // Back cards
                <div class="absolute -left-3 -top-2 w-16 h-12 bg-gradient-to-br from-pink-400 to-pink-500 rounded-lg transform rotate-12 opacity-80"></div>
                <div class="absolute -right-2 -top-1 w-16 h-12 bg-gradient-to-br from-blue-400 to-blue-500 rounded-lg transform -rotate-6 opacity-60"></div>

                // Main video card
                <div class="relative w-20 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-lg flex items-center justify-center shadow-lg">
                    <div class="w-8 h-8 bg-gradient-to-br from-pink-500 to-red-500 rounded-full flex items-center justify-center">
                        <svg class="w-4 h-4 text-white ml-0.5" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M8 5v14l11-7z"/>
                        </svg>
                    </div>
                </div>

                // Green checkmark
                <div class="absolute -bottom-2 -right-2 w-8 h-8 bg-green-500 rounded-full flex items-center justify-center shadow-lg">
                    <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/>
                    </svg>
                </div>
            </div>
        }.into_any()
    }

    // Medal Icon
    pub fn medal_icon() -> AnyView {
        view! {
            <div class="relative">
                // Medal circle with star
                <div class="w-20 h-20 bg-gradient-to-br from-pink-400 to-pink-500 rounded-full flex items-center justify-center shadow-lg">
                    <div class="w-16 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-full flex items-center justify-center">
                        // Star icon
                        <svg class="w-8 h-8 text-yellow-400 fill-current" viewBox="0 0 24 24">
                            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                        </svg>
                    </div>
                </div>

                // Blue ribbons
                <div class="absolute -bottom-2 left-1/2 transform -translate-x-1/2">
                    <div class="flex space-x-1">
                        <div class="w-3 h-8 bg-blue-600 transform -rotate-12 rounded-sm"></div>
                        <div class="w-3 h-8 bg-blue-700 transform rotate-12 rounded-sm"></div>
                    </div>
                </div>
            </div>
        }.into_any()
    }

    // Target/Bullseye Icon
    pub fn target_icon() -> AnyView {
        view! {
            <div class="relative">
                // Outer target ring
                <div class="w-20 h-20 bg-gradient-to-br from-pink-400 to-pink-500 rounded-full flex items-center justify-center shadow-lg">
                    // Middle ring
                    <div class="w-16 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-full flex items-center justify-center">
                        // Inner bullseye
                        <div class="w-10 h-10 bg-gradient-to-br from-pink-500 to-red-500 rounded-full flex items-center justify-center">
                            <div class="w-6 h-6 bg-white rounded-full"></div>
                        </div>
                    </div>
                </div>

                // Arrow hitting the target
                <div class="absolute -right-2 top-1/2 transform -translate-y-1/2">
                    <div class="relative">
                        // Arrow shaft
                        <div class="w-6 h-1 bg-blue-600 rounded-full"></div>
                        // Arrow point
                        <div class="absolute right-0 top-1/2 transform -translate-y-1/2">
                            <div class="w-0 h-0 border-l-4 border-l-blue-600 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                        </div>
                        // Arrow fletching
                        <div class="absolute left-0 top-1/2 transform -translate-y-1/2 -translate-x-1">
                            <div class="w-2 h-2 bg-blue-400 transform rotate-45"></div>
                        </div>
                    </div>
                </div>
            </div>
        }.into_any()
    }

    // Target with Checkmark
    pub fn target_complete_icon() -> AnyView {
        view! {
            <div class="relative">
                // Outer target ring
                <div class="w-20 h-20 bg-gradient-to-br from-pink-400 to-pink-500 rounded-full flex items-center justify-center shadow-lg">
                    // Middle ring
                    <div class="w-16 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-full flex items-center justify-center">
                        // Inner bullseye
                        <div class="w-10 h-10 bg-gradient-to-br from-pink-500 to-red-500 rounded-full flex items-center justify-center">
                            <div class="w-6 h-6 bg-white rounded-full"></div>
                        </div>
                    </div>
                </div>

                // Arrow hitting the target
                <div class="absolute -right-2 top-1/2 transform -translate-y-1/2">
                    <div class="relative">
                        // Arrow shaft
                        <div class="w-6 h-1 bg-blue-600 rounded-full"></div>
                        // Arrow point
                        <div class="absolute right-0 top-1/2 transform -translate-y-1/2">
                            <div class="w-0 h-0 border-l-4 border-l-blue-600 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                        </div>
                        // Arrow fletching
                        <div class="absolute left-0 top-1/2 transform -translate-y-1/2 -translate-x-1">
                            <div class="w-2 h-2 bg-blue-400 transform rotate-45"></div>
                        </div>
                    </div>
                </div>

                // Green checkmark
                <div class="absolute -bottom-2 -right-2 w-8 h-8 bg-green-500 rounded-full flex items-center justify-center shadow-lg">
                    <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/>
                    </svg>
                </div>
            </div>
        }.into_any()
    }

    // Flame Icon
    pub fn flame_icon() -> AnyView {
        view! {
            <div class="w-20 h-20 relative">
                // Flame base (orange/yellow gradient)
                <div class="absolute bottom-0 w-16 h-16 bg-gradient-to-t from-orange-600 via-orange-400 to-yellow-400 rounded-full transform scale-110"></div>

                // Flame tip (yellow)
                <div class="absolute top-2 left-1/2 transform -translate-x-1/2 w-8 h-12 bg-gradient-to-t from-yellow-400 to-yellow-200 rounded-full"></div>

                // Inner flame (brighter)
                <div class="absolute bottom-1 left-1/2 transform -translate-x-1/2 w-10 h-10 bg-gradient-to-t from-red-500 via-orange-300 to-yellow-200 rounded-full"></div>

                // Flame highlights
                <div class="absolute top-4 left-1/2 transform -translate-x-1/2 w-4 h-8 bg-gradient-to-t from-yellow-300 to-white opacity-80 rounded-full"></div>
            </div>
        }.into_any()
    }

    // Lightning Coin Icon
    pub fn lightning_coin_icon() -> AnyView {
        view! {
            <div class="relative">
                // Outer golden ring
                <div class="w-20 h-20 bg-gradient-to-br from-yellow-400 to-yellow-600 rounded-full flex items-center justify-center shadow-lg">
                    // Inner coin face
                    <div class="w-16 h-16 bg-gradient-to-br from-yellow-300 to-yellow-500 rounded-full flex items-center justify-center">
                        // Lightning bolt
                        <svg class="w-8 h-8 text-purple-900 fill-current" viewBox="0 0 24 24">
                            <path d="M13 0L3 14h6l-2 10 10-14h-6l2-10z"/>
                        </svg>
                    </div>
                </div>

                // Shine effect
                <div class="absolute top-2 left-2 w-4 h-4 bg-white opacity-60 rounded-full blur-sm"></div>
                <div class="absolute top-1 right-3 w-2 h-2 bg-white opacity-40 rounded-full blur-sm"></div>
            </div>
        }.into_any()
    }

    // Lightning Coin with Flame
    pub fn lightning_coin_flame_icon() -> AnyView {
        view! {
            <div class="relative">
                // Main golden coin
                <div class="w-20 h-20 bg-gradient-to-br from-yellow-400 to-yellow-600 rounded-full flex items-center justify-center shadow-lg">
                    // Inner coin face
                    <div class="w-16 h-16 bg-gradient-to-br from-yellow-300 to-yellow-500 rounded-full flex items-center justify-center">
                        // Lightning bolt
                        <svg class="w-8 h-8 text-purple-900 fill-current" viewBox="0 0 24 24">
                            <path d="M13 0L3 14h6l-2 10 10-14h-6l2-10z"/>
                        </svg>
                    </div>
                </div>

                // Flame accent on the side
                <div class="absolute -right-2 -top-2">
                    <div class="w-8 h-10 relative">
                        // Flame base
                        <div class="absolute bottom-0 w-6 h-6 bg-gradient-to-t from-orange-600 via-orange-400 to-yellow-400 rounded-full"></div>
                        // Flame tip
                        <div class="absolute top-1 left-1/2 transform -translate-x-1/2 w-3 h-6 bg-gradient-to-t from-yellow-400 to-yellow-200 rounded-full"></div>
                        // Inner flame
                        <div class="absolute bottom-0 left-1/2 transform -translate-x-1/2 w-4 h-4 bg-gradient-to-t from-red-500 to-yellow-300 rounded-full"></div>
                    </div>
                </div>

                // Coin shine effects
                <div class="absolute top-2 left-2 w-4 h-4 bg-white opacity-60 rounded-full blur-sm"></div>
                <div class="absolute top-1 right-3 w-2 h-2 bg-white opacity-40 rounded-full blur-sm"></div>
            </div>
        }.into_any()
    }

    // Megaphone Icon
    pub fn megaphone_icon() -> AnyView {
        view! {
            <div class="w-20 h-14 relative">
                // Megaphone cone (pink gradient)
                <div class="absolute left-0 top-1/2 transform -translate-y-1/2 w-12 h-8 bg-gradient-to-r from-pink-400 to-pink-500 rounded-l-full flex items-center justify-center shadow-lg">
                    // Inner cone
                    <div class="w-8 h-5 bg-gradient-to-r from-pink-300 to-pink-400 rounded-l-full"></div>
                </div>

                // Handle
                <div class="absolute right-2 top-1/2 transform -translate-y-1/2 translate-y-1 w-4 h-6 bg-gradient-to-b from-pink-500 to-pink-600 rounded"></div>

                // Sound waves/effects
                <div class="absolute -right-2 top-1/2 transform -translate-y-1/2">
                    <div class="w-3 h-1 bg-yellow-400 rounded-full mb-1 transform rotate-12"></div>
                    <div class="w-4 h-1 bg-orange-400 rounded-full mb-1 transform -rotate-12"></div>
                    <div class="w-3 h-1 bg-yellow-300 rounded-full transform rotate-6"></div>
                </div>
            </div>
        }.into_any()
    }

    // Megaphone with Checkmark
    pub fn megaphone_complete_icon() -> AnyView {
        view! {
            <div class="relative">
                // Main megaphone body
                <div class="w-20 h-14 relative">
                    // Megaphone cone (pink gradient)
                    <div class="absolute left-0 top-1/2 transform -translate-y-1/2 w-12 h-8 bg-gradient-to-r from-pink-400 to-pink-500 rounded-l-full flex items-center justify-center shadow-lg">
                        // Inner cone
                        <div class="w-8 h-5 bg-gradient-to-r from-pink-300 to-pink-400 rounded-l-full"></div>
                    </div>

                    // Handle
                    <div class="absolute right-2 top-1/2 transform -translate-y-1/2 translate-y-1 w-4 h-6 bg-gradient-to-b from-pink-500 to-pink-600 rounded"></div>

                    // Sound waves/effects
                    <div class="absolute -right-2 top-1/2 transform -translate-y-1/2">
                        <div class="w-3 h-1 bg-yellow-400 rounded-full mb-1 transform rotate-12"></div>
                        <div class="w-4 h-1 bg-orange-400 rounded-full mb-1 transform -rotate-12"></div>
                        <div class="w-3 h-1 bg-yellow-300 rounded-full transform rotate-6"></div>
                    </div>
                </div>

                // Green checkmark overlay
                <div class="absolute -bottom-2 -right-2 w-8 h-8 bg-green-500 rounded-full flex items-center justify-center shadow-lg">
                    <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/>
                    </svg>
                </div>
            </div>
        }.into_any()
    }

    // Target with Blue Card (for "Almost There")
    pub fn target_with_card_icon() -> AnyView {
        view! {
            <div class="relative">
                // Blue notification card behind target
                <div class="absolute -left-4 -bottom-2 w-8 h-6 bg-gradient-to-br from-blue-400 to-blue-500 rounded transform -rotate-12"></div>

                // Main target
                <div class="w-20 h-20 bg-gradient-to-br from-pink-400 to-pink-500 rounded-full flex items-center justify-center shadow-lg">
                    // Middle ring
                    <div class="w-16 h-16 bg-gradient-to-br from-gray-100 to-gray-200 rounded-full flex items-center justify-center">
                        // Inner bullseye
                        <div class="w-10 h-10 bg-gradient-to-br from-pink-500 to-red-500 rounded-full flex items-center justify-center">
                            <div class="w-6 h-6 bg-white rounded-full"></div>
                        </div>
                    </div>
                </div>

                // Arrow hitting the target
                <div class="absolute -right-2 top-1/2 transform -translate-y-1/2">
                    <div class="relative">
                        // Arrow shaft
                        <div class="w-6 h-1 bg-blue-600 rounded-full"></div>
                        // Arrow point
                        <div class="absolute right-0 top-1/2 transform -translate-y-1/2">
                            <div class="w-0 h-0 border-l-4 border-l-blue-600 border-t-2 border-t-transparent border-b-2 border-b-transparent"></div>
                        </div>
                        // Arrow fletching
                        <div class="absolute left-0 top-1/2 transform -translate-y-1/2 -translate-x-1">
                            <div class="w-2 h-2 bg-blue-400 transform rotate-45"></div>
                        </div>
                    </div>
                </div>
            </div>
        }.into_any()
    }
}
