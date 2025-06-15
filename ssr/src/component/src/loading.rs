use leptos::prelude::*;

#[component]
pub fn Loading(text: String, children: Children) -> impl IntoView {
    view! {
        {children()}
        <div class="flex flex-col gap-10 justify-center items-center bg-black h-dvh w-dvw">
            <img class="object-contain w-56 h-56 animate-pulse" src="/img/yral/logo.webp" />
            <span class="text-2xl text-white/60">{text}</span>
        </div>
    }
}
