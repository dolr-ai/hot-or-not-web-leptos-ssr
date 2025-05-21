use leptos::prelude::*;
use leptos_icons::*;
use leptos::portal::Portal;

#[component]
pub fn Popup(#[prop(into)] show: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    view! {
        <Portal>
            <div
                on:click={
                    #[cfg(feature = "hydrate")]
                    {
                        move |ev| {
                            use web_sys::HtmlElement;
                            let target = event_target::<HtmlElement>(&ev);
                            if target.class_list().contains("modal-bg") {
                                show.set(false);
                            }
                        }
                    }
                    #[cfg(not(feature = "hydrate"))] { |_| () }
                }
                class="flex cursor-pointer modal-bg inset-0 fixed bg-black/60 z-[999] justify-center items-center backdrop-blur-sm"
            >
                <div style="background: url('/img/common/gradient-backdrop.png'); background-size: cover; background-position: center;" class="mx-4 py-4 px-[20px] max-w-full max-h-full items-center cursor-auto flex-col flex justify-around bg-neutral-900 rounded-md">
                    <div class="flex w-full justify-end items-center">
                        <button
                            on:click=move |_| show.set(false)
                            class="text-white text-center p-1 text-lg md:text-xl bg-neutral-600 rounded-full"
                        >
                            <Icon icon=icondata::ChCross />
                        </button>
                    </div>
                    <div class="pb-4 w-full">{children()}</div>
                </div>
            </div>
        </Portal>
    }.into_any()
}
