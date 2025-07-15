mod items;

use leptos::prelude::*;
use leptos_icons::*;
use leptos_router::hooks::query_signal;

use component::title::TitleText;

#[component]
fn FaqItem(#[prop(into)] header: String, #[prop(into)] content: String) -> impl IntoView {
    let show = RwSignal::new(false);

    view! {
        <div class="flex flex-col gap-1 p-3 w-full rounded-md bg-white/10">
            <div
                class="grid grid-cols-2 items-center w-full cursor-pointer"
                on:click=move |_| show.update(|s| *s = !*s)
            >
                <span class="text-lg">{header}</span>
                <div class="justify-self-end text-lg text-primary-600">
                    <Show when=show fallback=|| view! { <Icon icon=icondata::AiPlusOutlined /> }>
                        <Icon icon=icondata::AiMinusOutlined />
                    </Show>
                </div>
            </div>
            <Show when=show>
                <div class="text-sm text-white/70">{content.clone()}</div>
            </Show>
        </div>
    }
}

#[component]
fn FaqType<F: FnMut() + 'static>(
    #[prop(into)] name: String,
    #[prop(optional)] init_checked: bool,
    mut onclick: F,
) -> impl IntoView {
    view! {
        <label class="flex flex-col items-center cursor-pointer" on:click=move |_| onclick()>
            <input
                type="radio"
                value=""
                name="faq-selection"
                class="sr-only peer"
                checked=init_checked
            />
            <span class="text-md text-white/50 peer-checked:text-white">{name}</span>
            <div class="hidden p-1 rounded-full bg-primary-600 peer-checked:block"></div>
        </label>
    }
}

#[component]
fn FaqView(
    tab_idx: usize,
    cur_tab: Signal<usize>,
    tab_content: &'static [(&'static str, &'static str)],
) -> impl IntoView {
    view! {
        <Show when=move || {
            tab_idx == cur_tab()
        }>
            {tab_content
                .iter()
                .map(|(header, content)| view! { <FaqItem header=*header content=*content /> })
                .collect_view()}
        </Show>
    }
}

#[component]
fn FaqSwitcher() -> impl IntoView {
    let (cur_tab, set_cur_tab) = query_signal::<String>("tab");
    let current_tab = Signal::derive(move || match cur_tab.get().as_deref() {
        Some("general") => 0,
        Some("tokens") => 1,
        Some("nfts") => 2,
        _ => 0,
    });

    view! {
        <div class="flex flex-row gap-6 mb-4">
            <FaqType
                name="General"
                onclick=move || set_cur_tab(Some("general".into()))
                init_checked=true
            />
            <FaqType name="Tokens" onclick=move || set_cur_tab(Some("tokens".into())) />
            <FaqType name="NFTs" onclick=move || set_cur_tab(Some("nfts".into())) />
        </div>
        <div class="flex flex-col gap-4 w-full">
            <FaqView tab_idx=0 cur_tab=current_tab tab_content=&items::GENERAL_ITEMS />
            <FaqView tab_idx=1 cur_tab=current_tab tab_content=&items::TOKENS_ITEMS />
            <FaqView tab_idx=2 cur_tab=current_tab tab_content=&items::NFTS_ITEMS />
        </div>
    }
}

#[component]
pub fn Faq() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center px-8 pt-4 pb-14 w-screen min-h-screen text-white bg-black">
            <TitleText>
                <span class="font-bold">FAQs</span>
            </TitleText>
            <div class="my-8 w-full text-lg">Find all your answers here</div>
            <FaqSwitcher />
        </div>
    }
}
