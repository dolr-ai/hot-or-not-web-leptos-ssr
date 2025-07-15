use gloo::timers::callback::Timeout;
use leptos::prelude::*;
use leptos_icons::*;
use utils::web::copy_to_clipboard;

#[component]
pub fn DashboxLoading() -> impl IntoView {
    view! {
        <div class="flex p-1 w-full h-10 rounded-full border-2 border-dashed md:w-2/12 md:h-12 border-primary-500">
            <span class="w-full h-full rounded-full animate-pulse bg-white/30"></span>
        </div>
    }
}

#[component]
pub fn DashboxLoaded(text: String) -> impl IntoView {
    let show_copied_popup = RwSignal::new(false);

    let text_copy = text.clone();
    let click_copy = Action::new(move |()| {
        let text = text_copy.clone();
        async move {
            let _ = copy_to_clipboard(&text);

            show_copied_popup.set(true);
            Timeout::new(1200, move || show_copied_popup.set(false)).forget();
        }
    });

    view! {
        <div class="flex gap-2 items-center p-3 rounded-full border-2 border-dashed w-fit border-primary-500">
            <span class="lg:text-lg text-md text-ellipsis line-clamp-1">{text}</span>
            <button on:click=move |_| {
                click_copy.dispatch(());
            }>
                <Icon attr:class="text-xl" icon=icondata::FaCopyRegular />
            </button>
        </div>
        <Show when=show_copied_popup>
            <div class="flex absolute flex-col justify-center items-center z-4">
                <span class="flex absolute top-28 flex-row justify-center items-center w-28 h-10 text-center rounded-md shadow-lg bg-white/90">
                    <p class="text-black">Copied!</p>
                </span>
            </div>
        </Show>
    }
}
