use super::{ModelDropdown, TokenDropdown};
use crate::upload::ai::server::fetch_video_providers;
use crate::upload::ai::token_balance::load_token_balance;
use crate::upload::ai::types::VideoGenerationParams;
use codee::string::JsonSerdeCodec;
use component::{back_btn::BackButton, buttons::GradientButton, login_modal::LoginModal};
use consts::auth::REFRESH_MAX_AGE;
use consts::AUTH_JOURNEY_PAGE;
use leptos::{html::Input, prelude::*};

use leptos::{ev::Event, web_sys};
use leptos_icons::*;
use leptos_router::hooks::use_location;
use leptos_use::{use_cookie_with_options, UseCookieOptions};
use state::canisters::auth_state;
use utils::event_streaming::events::VideoUploadInitiated;
use utils::mixpanel::mixpanel_events::{
    BottomNavigationCategory, MixPanelEvent, MixpanelGlobalProps,
};
use utils::send_wrap;
use videogen_common::{ProviderInfo, TokenType};
use wasm_bindgen::{prelude::*, JsCast};
use yral_canisters_common::utils::token::balance::TokenBalance;

// Component for error display
#[component]
fn ErrorDisplay(generation_error: Signal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || generation_error.get().is_some()>
            <div class="p-3 bg-red-900/20 border border-red-500/30 rounded-lg">
                <div class="text-red-400 text-sm">
                    {move || generation_error.get().unwrap_or_default()}
                </div>
            </div>
        </Show>
    }
}

// Component for image upload section
#[component]
fn ImageUploadSection(
    selected_provider: Signal<Option<ProviderInfo>>,
    uploaded_image: RwSignal<Option<String>>,
    image_input_ref: NodeRef<Input>,
) -> impl IntoView {
    // Handle image upload
    let handle_image_upload = move |event: Event| {
        let input: web_sys::HtmlInputElement = event_target(&event);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let file_reader = web_sys::FileReader::new().unwrap();
                let file_reader_clone = file_reader.clone();

                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    if let Ok(result) = file_reader_clone.result() {
                        if let Ok(data_url) = result.dyn_into::<js_sys::JsString>() {
                            uploaded_image.set(Some(data_url.as_string().unwrap()));
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader
                    .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();

                let _ = file_reader.read_as_data_url(&file);
            }
        }
    };
    view! {
        <Show when=move || selected_provider.get().map(|p| p.supports_image).unwrap_or(false)>
            <div class="w-full">
                <div class="flex items-center gap-2 mb-2">
                    <label class="block text-sm font-medium text-white">Image</label>
                    <span class="text-xs text-neutral-400">(Optional)</span>
                    <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                </div>

                <div class="relative">
                    <input
                        type="file"
                        accept="image/*"
                        node_ref=image_input_ref
                        on:change=handle_image_upload
                        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                    />
                    <div class="flex flex-col items-center justify-center p-12 bg-neutral-900 border border-neutral-800 rounded-lg hover:bg-neutral-800 transition-colors cursor-pointer">
                        <Show
                            when=move || uploaded_image.get().is_some()
                            fallback=move || view! {
                                <div class="flex flex-col items-center gap-3">
                                    <Icon icon=icondata::AiPictureOutlined attr:class="text-neutral-500 text-3xl" />
                                    <span class="text-neutral-500 text-sm">"Click to upload an image"</span>
                                </div>
                            }
                        >
                            <img
                                src=move || uploaded_image.get().unwrap_or_default()
                                class="max-w-full max-h-32 object-contain rounded"
                                alt="Uploaded preview"
                            />
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// Component for audio upload section
#[component]
fn AudioUploadSection(
    selected_provider: Signal<Option<ProviderInfo>>,
    uploaded_audio: RwSignal<Option<String>>,
    audio_input_ref: NodeRef<Input>,
) -> impl IntoView {
    // Handle audio upload
    let handle_audio_upload = move |event: Event| {
        let input: web_sys::HtmlInputElement = event_target(&event);
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                let file_reader = web_sys::FileReader::new().unwrap();
                let file_reader_clone = file_reader.clone();
                let _file_name = file.name();

                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    if let Ok(result) = file_reader_clone.result() {
                        if let Ok(data_url) = result.dyn_into::<js_sys::JsString>() {
                            uploaded_audio.set(Some(data_url.as_string().unwrap()));
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader
                    .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
                    .unwrap();
                closure.forget();

                let _ = file_reader.read_as_data_url(&file);
            }
        }
    };

    view! {
        <Show when=move || selected_provider.get().map(|p| p.supports_audio_input).unwrap_or(false)>
            <div class="w-full">
                <div class="flex items-center gap-2 mb-2">
                    <label class="block text-sm font-medium text-white">Audio</label>
                    <span class="text-xs text-neutral-400">(Required for Talking Head)</span>
                    <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                </div>

                <div class="relative">
                    <input
                        type="file"
                        accept="audio/*"
                        node_ref=audio_input_ref
                        on:change=handle_audio_upload
                        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                    />
                    <div class="flex flex-col items-center justify-center p-12 bg-neutral-900 border border-neutral-800 rounded-lg hover:bg-neutral-800 transition-colors cursor-pointer">
                        <Show
                            when=move || uploaded_audio.get().is_some()
                            fallback=move || view! {
                                <div class="flex flex-col items-center gap-3">
                                    <Icon icon=icondata::AiAudioOutlined attr:class="text-neutral-500 text-3xl" />
                                    <span class="text-neutral-500 text-sm">"Click to upload audio"</span>
                                </div>
                            }
                        >
                            <div class="flex flex-col items-center gap-2">
                                <Icon icon=icondata::AiAudioFilled attr:class="text-green-500 text-3xl" />
                                <span class="text-green-500 text-sm">"Audio uploaded"</span>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// Component for prompt section
#[component]
fn PromptSection(prompt_text: RwSignal<String>, character_count: Signal<usize>) -> impl IntoView {
    view! {
        <div class="w-full">
            <label class="block text-sm font-medium text-white mb-2">Prompt</label>
            <div class="relative">
                <textarea
                    class="w-full p-4 bg-neutral-900 border border-neutral-800 rounded-lg text-white placeholder:text-neutral-500 resize-none focus:outline-none focus:border-pink-400 transition-colors"
                    rows=6
                    placeholder="Enter the Prompt here..."
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        if value.len() <= 500 {
                            prompt_text.set(value);
                        }
                    }
                    prop:value=move || prompt_text.get()
                ></textarea>

                // Character counter
                <div class="absolute bottom-3 right-3 text-xs text-neutral-400">
                    {move || format!("{}/500", character_count.get())}
                </div>
            </div>
        </div>
    }
}

// Component for credits section
#[component]
fn CreditsSection(
    selected_provider: Signal<Option<ProviderInfo>>,
    selected_token: RwSignal<TokenType>,
    show_token_dropdown: RwSignal<bool>,
    is_logged_in: Signal<bool>,
    rate_limit_resource: LocalResource<Result<bool, ServerFnError>>,
    balance_resource: LocalResource<Result<TokenBalance, ServerFnError>>,
    has_sufficient_balance: RwSignal<bool>,
    locked_rate_limit_status: RwSignal<Option<bool>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-2">
            <span class="text-sm font-medium text-neutral-300">Credits Required</span>
            <div class="flex items-center justify-between px-2.5 py-2 bg-neutral-900 border border-neutral-800 rounded-lg">
                <div class="text-base font-semibold text-neutral-300">
                    <Suspense fallback=move || view! { <span>"..."</span> }>
                        {move || Suspend::new(async move {
                            let token = selected_token.get();
                            let provider_opt = selected_provider.get();

                            // If no provider selected yet, show loading
                            let provider = match provider_opt {
                                Some(p) => p,
                                None => return view! { <span>"Loading..."</span> }.into_any(),
                            };

                            // Check if user can use free generation and lock in the status
                            let can_use_free = if is_logged_in.get() {
                                match rate_limit_resource.await {
                                    Ok(can_use) => {
                                        // Lock in the rate limit status for consistency
                                        locked_rate_limit_status.set(Some(can_use));
                                        can_use
                                    },
                                    Err(_) => {
                                        locked_rate_limit_status.set(Some(false));
                                        false
                                    },
                                }
                            } else {
                                // Non-logged-in users get free tier
                                locked_rate_limit_status.set(Some(true));
                                true
                            };

                            // Get cost from provider based on token type
                            let cost = match token {
                                TokenType::Sats => provider.cost.sats,
                                TokenType::Dolr => provider.cost.dolr,
                                _ => 0,
                            };
                            let humanized = match token {
                                TokenType::Sats => TokenBalance::new(cost.into(), 0).humanize_float_truncate_to_dp(0),
                                TokenType::Dolr => TokenBalance::new(cost.into(), 8).humanize_float_truncate_to_dp(2),
                                _ => "0".to_string(),
                            };

                            if can_use_free {
                                // Show 0 with strikethrough original price
                                view! {
                                    <div class="flex items-center gap-2">
                                        <span>"0"</span>
                                        <span class="line-through text-neutral-500">{humanized}</span>
                                    </div>
                                }.into_any()
                            } else {
                                // Show regular price
                                view! {
                                    <div class="flex items-center gap-2">
                                        <span>{humanized}</span>
                                    </div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
                // Always show SATS/DOLR dropdown
                <TokenDropdown
                    selected_token=selected_token
                    show_dropdown=show_token_dropdown
                />
            </div>

            // Current Balance
            <Suspense fallback=move || view! {
                <div class="flex items-center gap-2 text-xs text-neutral-400">
                    <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                    <span>"Current balance: Loading..."</span>
                </div>
            }>
                {move || Suspend::new(async move {
                    match balance_resource.await {
                        Ok(balance) => {
                            // Check balance sufficiency
                            let provider_opt = selected_provider.get();

                            // If no provider selected yet, show loading
                            let provider = match provider_opt {
                                Some(p) => p,
                                None => {
                                    has_sufficient_balance.set(false);
                                    return view! {
                                        <div class="flex items-center gap-2 text-xs text-neutral-400">
                                            <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                                            <span>"Loading providers..."</span>
                                        </div>
                                    }.into_any()
                                }
                            };

                            let token_type = selected_token.get();
                            let required_amount = match token_type {
                                TokenType::Sats => provider.cost.sats,
                                TokenType::Dolr => provider.cost.dolr,
                                _ => 0,
                            };

                            // Check if user can use free generation (use locked status if available)
                            let can_use_free = if !is_logged_in.get() {
                                true // Non-logged-in users always see free tier message
                            } else {
                                locked_rate_limit_status.get().unwrap_or(false)
                            };

                            let is_sufficient = if can_use_free {
                                true // Free generation always has sufficient "balance"
                            } else {
                                match token_type {
                                    TokenType::Sats => balance.e8s >= required_amount,
                                    TokenType::Dolr => balance.e8s >= required_amount,
                                    _ => false,
                                }
                            };

                            // Update the balance sufficiency signal
                            has_sufficient_balance.set(is_sufficient);

                            if can_use_free {
                                // Show green message for free generation
                                view! {
                                    <div class="flex items-center gap-2 text-xs" style="color: #1ec981;">
                                        <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-sm" attr:style="color: #1ec981;" />
                                        <span>"Enjoy 1 free AI video per day. Use credits for more."</span>
                                    </div>
                                }.into_any()
                            } else {
                                // Show regular balance for paid generation
                                let balance_text = match token_type {
                                    TokenType::Sats => {
                                        let formatted_balance = balance.humanize_float_truncate_to_dp(0);
                                        format!("Current balance: {formatted_balance} YRAL")
                                    },
                                    TokenType::Dolr => {
                                        let formatted_balance = balance.humanize_float_truncate_to_dp(2);
                                        format!("Current balance: {formatted_balance} DOLR")
                                    },
                                    _ => "Current balance: Unknown token type".to_string(),
                                };
                                view! {
                                    <div class="flex items-center gap-2 text-xs text-neutral-400">
                                        <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                                        <span>{balance_text}</span>
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(_) => {
                            has_sufficient_balance.set(false);
                            view! {
                                <div class="flex items-center gap-2 text-xs text-neutral-400">
                                    <Icon icon=icondata::AiInfoCircleOutlined attr:class="text-neutral-400 text-sm" />
                                    <span>{"Current balance: Error loading".to_string()}</span>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
pub fn PreUploadAiView(
    generate_action: Action<VideoGenerationParams, Result<String, String>>,
    set_stored_params: WriteSignal<VideoGenerationParams>,
) -> impl IntoView {
    // Form state
    // Use Resource to fetch providers from API
    let providers_resource = LocalResource::new(move || async move {
        match fetch_video_providers().await {
            Ok(providers) => providers,
            Err(e) => {
                leptos::logging::error!("Failed to fetch providers: {}", e);
                // Return empty vector as fallback
                Vec::new()
            }
        }
    });

    // Default provider will be set once providers are loaded
    let selected_provider = RwSignal::new(None::<ProviderInfo>);
    let show_dropdown = RwSignal::new(false);
    let selected_token = RwSignal::new(TokenType::Sats);
    let show_token_dropdown = RwSignal::new(false);
    let prompt_text = RwSignal::new(String::new());
    let character_count = Signal::derive(move || prompt_text.get().len());
    let uploaded_image = RwSignal::new(None::<String>);
    let uploaded_audio = RwSignal::new(None::<String>);

    // Set default provider once loaded
    Effect::new(move |_| {
        if let Some(providers) = providers_resource.get() {
            if !providers.is_empty() && selected_provider.get_untracked().is_none() {
                selected_provider.set(Some(providers.into_iter().next().unwrap()));
            }
        }
    });

    // Login modal state
    let show_login_modal = RwSignal::new(false);

    let loc = use_location();

    let (_, set_auth_journey_page) =
        use_cookie_with_options::<BottomNavigationCategory, JsonSerdeCodec>(
            AUTH_JOURNEY_PAGE,
            UseCookieOptions::default()
                .path("/")
                .max_age(REFRESH_MAX_AGE.as_millis() as i64),
        );
    // Lock in rate limit status to prevent race conditions
    let locked_rate_limit_status = RwSignal::new(None::<bool>);

    // Get auth state
    let auth = auth_state();
    let is_logged_in = auth.is_logged_in_with_oauth(); // Signal::stored(true); //

    // Reset locked status when token or provider changes to force a fresh check
    Effect::new(move |_| {
        selected_token.get();
        selected_provider.get();
        locked_rate_limit_status.set(None);
    });

    // Create rate limit resource to check if user can use free generation
    // Returns true if user can use free generation (not rate limited)
    let rate_limit_resource = auth.derive_resource(
        move || is_logged_in.get(),
        move |canisters, is_registered| {
            send_wrap(async move {
                let principal = canisters.user_principal();
                let rate_limits = canisters.rate_limits().await;
                let status = rate_limits
                    .get_rate_limit_status(principal, "VIDEOGEN".to_string(), is_registered)
                    .await
                    .ok()
                    .flatten();

                // Return true if user is NOT rate limited (can use free)
                Ok(match status {
                    Some(s) => !s.is_limited,
                    None => false, // Default to paid if status unknown
                })
            })
        },
    );

    // Create a resource that loads balance based on auth state and selected token
    let balance_resource = auth.derive_resource(
        move || selected_token.get(),
        move |canisters, token| {
            send_wrap(async move {
                let principal = canisters.user_principal();
                load_token_balance(principal, token).await
            })
        },
    );

    // Form validation - check based on provider requirements
    let form_valid = Signal::derive(move || {
        let provider = selected_provider.get();

        if let Some(p) = provider {
            if p.id == "talkinghead" {
                // TalkingHead requires both image and audio
                uploaded_image.get().is_some() && uploaded_audio.get().is_some()
            } else {
                // Other providers require prompt text
                !prompt_text.get().trim().is_empty()
            }
        } else {
            false // No provider selected
        }
    });
    let base_can_generate =
        Signal::derive(move || form_valid.get() && !generate_action.pending().get());

    // Create a signal for balance sufficiency that will be used inside Suspense
    let has_sufficient_balance = RwSignal::new(false);

    // Error handling from action
    let generation_error = Signal::derive(move || {
        generate_action
            .value()
            .get()
            .and_then(|result| result.err())
    });

    // File inputs for image and audio upload
    let image_input = NodeRef::<Input>::new();
    let audio_input = NodeRef::<Input>::new();

    let ev_ctx = auth.event_ctx();
    VideoUploadInitiated.send_event(ev_ctx);

    let login_click_action = Action::new(move |_: &()| {
        show_login_modal.set(true);
        let path = loc.pathname.get_untracked();
        let category: BottomNavigationCategory =
            BottomNavigationCategory::try_from(path.clone()).unwrap_or_default();
        set_auth_journey_page.set(Some(category));
        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
            let page_name = global.page_name();
            MixPanelEvent::track_signup_clicked(global, page_name);
        }
        async {}
    });

    view! {
        <div class="flex flex-col bg-black min-w-dvw min-h-dvh">
            // Header with back button and title
            <div class="flex items-center justify-between p-4 pt-12">
                <div class="text-white">
                    <BackButton fallback="/upload-options".to_string() />
                </div>
                <h1 class="text-lg font-bold text-white">"Create AI Video"</h1>
                <div class="w-6"></div> // Spacer for centering
            </div>

            // Main form content
            <div class="flex-1 px-4 py-6 pb-24 max-w-md mx-auto w-full">
                <div class="flex flex-col gap-6">

                    // Provider Selection Dropdown
                    <Suspense fallback=move || view! {
                        <div class="w-full">
                            <label class="block text-sm font-medium text-white mb-2">Model</label>
                            <div class="flex items-center justify-center p-4 bg-neutral-900 border border-neutral-800 rounded-lg">
                                <span class="text-neutral-400">Loading models...</span>
                            </div>
                        </div>
                    }>
                        {move || Suspend::new(async move {
                            let providers = providers_resource.await;
                            view! {
                                <ModelDropdown
                                    selected_provider=selected_provider
                                    show_dropdown=show_dropdown
                                    providers=providers
                                />
                            }
                        })}
                    </Suspense>


                    // Image Upload Section (Optional)
                    <ImageUploadSection
                        selected_provider=selected_provider.into()
                        uploaded_image=uploaded_image
                        image_input_ref=image_input
                    />

                    // Audio Upload Section (For TalkingHead)
                    <AudioUploadSection
                        selected_provider=selected_provider.into()
                        uploaded_audio=uploaded_audio
                        audio_input_ref=audio_input
                    />

                    // Prompt Section (Hide for TalkingHead)
                    <Show when=move || {
                        selected_provider.get()
                            .map(|p| p.id != "talkinghead")
                            .unwrap_or(true)
                    }>
                        <PromptSection
                            prompt_text=prompt_text
                            character_count=character_count
                        />
                    </Show>

                    // Credits Required Section
                    <CreditsSection
                        selected_provider=selected_provider.into()
                        selected_token=selected_token
                        show_token_dropdown=show_token_dropdown
                        is_logged_in=is_logged_in
                        rate_limit_resource=rate_limit_resource
                        balance_resource=balance_resource
                        has_sufficient_balance=has_sufficient_balance
                        locked_rate_limit_status=locked_rate_limit_status
                    />



                    // Error display
                    <ErrorDisplay generation_error=generation_error />

                    // Generate & Upload Video Button
                    <div class="mt-4">
                        <Suspense
                            fallback=move || view! {
                                <div class="w-full h-12 rounded-lg font-bold bg-gradient-to-r from-pink-500 to-purple-500 flex items-center justify-center text-white opacity-50">
                                    "Loading..."
                                </div>
                            }
                        >
                            {move || Suspend::new(async move {
                                let principal_result = auth.user_principal.await;
                                view! {
                                    <GradientButton
                                        on_click=move || {
                                            // Check if user is logged in
                                            if !is_logged_in.get_untracked() {
                                                // Show login modal if not logged in
                                                login_click_action.dispatch(());
                                            } else {
                                                match &principal_result {
                                                    Ok(user_principal) => {
                                                        // Get current form values
                                                        let prompt = prompt_text.get_untracked();
                                                        let provider_opt = selected_provider.get_untracked();

                                                        // Ensure provider is selected
                                                        let provider = match provider_opt {
                                                            Some(p) => p,
                                                            None => {
                                                                leptos::logging::error!("No provider selected");
                                                                return;
                                                            }
                                                        };

                                                        let image_data = uploaded_image.get_untracked();
                                                        let audio_data = uploaded_audio.get_untracked();

                                                        // Validation for TalkingHead provider
                                                        if provider.id == "talkinghead" {
                                                            if image_data.is_none() || audio_data.is_none() {
                                                                leptos::logging::error!("TalkingHead requires both image and audio");
                                                                return;
                                                            }
                                                        } else {
                                                            // For other providers, ensure prompt is not empty
                                                            if prompt.is_empty() {
                                                                leptos::logging::error!("Prompt cannot be empty");
                                                                return;
                                                            }
                                                        }

                                                        // Use locked rate limit status to determine if user can use free generation
                                                        let can_use_free = locked_rate_limit_status.get_untracked()
                                                            .unwrap_or(false);

                                                        // Determine which token type to send to API
                                                        let api_token_type = if can_use_free {
                                                            TokenType::Free  // Send Free to API for free generation
                                                        } else {
                                                            selected_token.get_untracked()  // Send actual selected token
                                                        };

                                                        // Track Create AI Video clicked
                                                        if let Some(global) = MixpanelGlobalProps::from_ev_ctx(ev_ctx) {
                                                            MixPanelEvent::track_create_ai_video_clicked(
                                                                global,
                                                                provider.name.clone(),
                                                                format!("{api_token_type:?}").to_lowercase()
                                                            );
                                                        }

                                                        // For TalkingHead, use placeholder prompt for backend validation
                                                        let final_prompt = if provider.id == "talkinghead" {
                                                            "[TalkingHead: Audio-based generation]".to_string()
                                                        } else {
                                                            prompt
                                                        };

                                                        // Create params struct and dispatch the action
                                                        let params = VideoGenerationParams {
                                                            user_principal: *user_principal,
                                                            prompt: final_prompt,
                                                            provider,
                                                            image_data,
                                                            audio_data,
                                                            token_type: api_token_type,  // Use the determined token type
                                                        };
                                                        // Store parameters before dispatching
                                                        set_stored_params.set(params.clone());
                                                        generate_action.dispatch(params);
                                                    }
                                                    Err(e) => {
                                                        leptos::logging::error!("Failed to get user principal: {:?}", e);
                                                        // You might want to show an error message to the user here
                                                    }
                                                }
                                            }
                                        }
                                        classes="w-full h-[45px] rounded-lg font-bold text-base".to_string()
                                        disabled=Signal::derive(move || {
                                            if !is_logged_in.get() {
                                                // Non-logged-in users can always click to trigger login (only need valid form)
                                                !base_can_generate.get()
                                            } else {
                                                // Logged-in users need sufficient balance
                                                !base_can_generate.get() || !has_sufficient_balance.get()
                                            }
                                        })
                                    >
                                        {move || {
                                            if generate_action.pending().get() {
                                                "Generating & Uploading..."
                                            } else {
                                                "Generate & Upload Video"
                                            }
                                        }}
                                    </GradientButton>
                                }
                            })}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>

        // Login Modal
        <LoginModal show=show_login_modal redirect_to=None />
    }
}
