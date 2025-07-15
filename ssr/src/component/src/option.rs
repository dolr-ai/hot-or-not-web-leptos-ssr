use leptos::prelude::*;

#[component]
pub fn SelectOption(#[prop(into)] is: String, value: ReadSignal<String>) -> impl IntoView {
    let display = is.clone();
    let val = is.clone();
    view! {
        <option value=val selected=move || value() == is>
            {display}
        </option>
    }
}
