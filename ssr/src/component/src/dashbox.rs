use gloo::timers::callback::Timeout;
use leptos::prelude::*;
use leptos_icons::*;
use utils::web::copy_to_clipboard;

#[component]
pub fn DashboxLoading() -> impl IntoView {
    view! {
        <div class="flex border-dashed w-full md:w-2/12 p-1 h-10 md:h-12 border-2 border-primary-500 rounded-full">
            <span class="bg-white/30 w-full h-full animate-pulse rounded-full "></span>
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
        <div class="flex items-center w-fit rounded-full border-dashed border-2 p-3 gap-2 border-primary-500">
            <span class="text-md lg:text-lg text-ellipsis line-clamp-1">{text}</span>
            <button on:click=move |_| {
                click_copy.dispatch(());
            }>
                <Icon attr:class="text-xl" icon=icondata::FaCopyRegular />
            </button>
        </div>
        <Show when=show_copied_popup>
            <div class="absolute flex flex-col justify-center items-center z-4">
                <span class="absolute top-28 flex flex-row justify-center items-center bg-white/90 rounded-md h-10 w-28 text-center shadow-lg">
                    <p class="text-black">Copied!</p>
                </span>
            </div>
        </Show>
    }
}
