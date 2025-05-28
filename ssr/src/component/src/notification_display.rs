use leptos::web_sys::AnimationEvent;
use leptos::{html::Div, prelude::*};
use wasm_bindgen::prelude::{wasm_bindgen, Closure};

use crate::base_route::Notification;

#[derive(serde::Deserialize, Debug, Clone)]
struct NotificationPayload {
    title: String,
    body: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: i32) -> i32;
}

#[component]
pub fn NotificationDisplay() -> impl IntoView {
    let notification_context = expect_context::<Notification>();
    let current_notification = RwSignal::new(None::<NotificationPayload>);
    let is_visible = RwSignal::new(false);
    let is_sliding_out = RwSignal::new(false);

    let notification_node_ref = NodeRef::<Div>::new();

    Effect::new(move |_| {
        if let Some(value) = notification_context.0.get() {
            let payload = serde_json::from_value::<NotificationPayload>(value).ok();
            current_notification.set(payload);
            is_visible.set(true);
            is_sliding_out.set(false);

            let closure = Closure::once(move || {
                if is_visible.get_untracked() && !is_sliding_out.get_untracked() {
                    is_sliding_out.set(true);
                }
            });
            setTimeout(&closure, 5000);
            closure.forget();
        } else if is_visible.get_untracked() && !is_sliding_out.get_untracked() {
            is_sliding_out.set(true);
        }
    });

    let handle_animation_end = move |ev: AnimationEvent| {
        if ev.animation_name() == "slideOut" {
            is_visible.set(false);
            is_sliding_out.set(false);
            current_notification.set(None);
            notification_context.0.set(None);
        }
    };

    let close_notification = move |_| {
        if is_visible.get() && !is_sliding_out.get() {
            is_sliding_out.set(true);
        }
    };

    view! {
        <Show when=is_visible>
            <div
                node_ref=notification_node_ref
                class="notification-container"
                class:slide-in=move || is_visible.get() && !is_sliding_out.get()
                class:slide-out=is_sliding_out
                on:animationend=handle_animation_end
            >
                {move || current_notification.get().map(|notif| view! {
                    <div class="notification-content">
                    <div class="notification-header">
                        <strong>{notif.title.clone()}</strong>
                        <button class="close-button" on:click=close_notification>"X"</button>
                    </div>
                    <p>{notif.body.clone()}</p>
                </div>
                })}
            </div>
        </Show>
    }
}
