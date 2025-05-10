use super::spinner::Spinner;
use crate::show_any::ShowAny;
use leptos::portal::Portal;
use leptos::prelude::*;
#[derive(Clone, Copy)]
pub enum ShowOverlay {
    /// Show overlay and allow closing by user
    Closable(RwSignal<bool>),
    MaybeClosable {
        show: RwSignal<bool>,
        /// Allow closing based on this signal
        closable: RwSignal<bool>,
    },
    /// Show overlay but prevent closing by user
    AlwaysLocked(Signal<bool>),
}

impl From<bool> for ShowOverlay {
    fn from(b: bool) -> Self {
        ShowOverlay::Closable(RwSignal::new(b))
    }
}

impl From<RwSignal<bool>> for ShowOverlay {
    fn from(s: RwSignal<bool>) -> Self {
        ShowOverlay::Closable(s)
    }
}

impl From<Signal<bool>> for ShowOverlay {
    fn from(s: Signal<bool>) -> Self {
        ShowOverlay::AlwaysLocked(s)
    }
}

impl Get for ShowOverlay {
    type Value = bool;

    fn get(&self) -> bool {
        match self {
            ShowOverlay::Closable(s) => s.get(),
            ShowOverlay::AlwaysLocked(s) => s.get(),
            ShowOverlay::MaybeClosable { show, .. } => show.get(),
        }
    }

    fn try_get(&self) -> Option<bool> {
        match self {
            ShowOverlay::Closable(s) => s.try_get(),
            ShowOverlay::AlwaysLocked(s) => s.try_get(),
            ShowOverlay::MaybeClosable { show, .. } => show.try_get(),
        }
    }
}

impl Set for ShowOverlay {
    type Value = bool;

    fn set(&self, value: bool) {
        match self {
            ShowOverlay::Closable(s) => s.set(value),
            ShowOverlay::AlwaysLocked(_) => {}
            ShowOverlay::MaybeClosable { show, closable } => {
                if closable.get_untracked() {
                    show.set(value);
                }
            }
        }
    }

    fn try_set(&self, value: bool) -> Option<bool> {
        match self {
            ShowOverlay::Closable(s) => s.try_set(value),
            ShowOverlay::AlwaysLocked(_) => None,
            ShowOverlay::MaybeClosable { show, closable } => {
                if closable.try_get_untracked()? {
                    show.try_set(value)
                } else {
                    None
                }
            }
        }
    }
}

impl DefinedAt for ShowOverlay {
    fn defined_at(&self) -> Option<&'static std::panic::Location<'static>> {
        match self {
            ShowOverlay::Closable(s) => s.defined_at(),
            ShowOverlay::AlwaysLocked(s) => s.defined_at(),
            ShowOverlay::MaybeClosable { show, .. } => show.defined_at(),
        }
    }
}

#[component]
pub fn ShadowOverlay(#[prop(into)] show: ShowOverlay, children: ChildrenFn) -> impl IntoView {
    let children_s = StoredValue::new(children);
    view! {
        <ShowAny when=move || show.get()>
            // Portal is necessary
            // see more: https://stackoverflow.com/questions/28157125/why-does-transform-break-position-fixed/28157774#28157774
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

                    class="flex cursor-pointer modal-bg w-dvw h-dvh fixed left-0 top-0 bg-black/60 z-[99] justify-center items-center overflow-hidden"
                >
                    {(children_s.get_value())()}
                </div>
            </Portal>
        </ShowAny>
    }
}

#[component]
fn ActionRunningOverlay(message: String) -> impl IntoView {
    view! {
        <div class="w-full h-full flex flex-col gap-6 items-center justify-center text-white text-center text-xl font-semibold">
            <Spinner />
            <span>{message}</span>
            <span>Please wait...</span>
        </div>
    }
}

#[component]
pub fn PopupOverlay(#[prop(into)] show: ShowOverlay, children: ChildrenFn) -> impl IntoView {
    view! {
        <ShadowOverlay show>
            <div class="px-4 pt-4 pb-12 mx-6 w-full lg:w-1/2 max-h-[65%] rounded-xl bg-white">
                {children()}
            </div>
        </ShadowOverlay>
    }
}

/// Tracks an action's progress and shows a modal with the result
/// action -> The action to track
/// loading_message -> The message to show while the action is pending
/// modal -> The modal to show when the action is done
/// close -> Set this signal to true to close the modal (automatically reset upon closing)
#[component]
pub fn ActionTrackerPopup<
    AStorage: Storage<ArcAction<S, R>>,
    S: 'static + Send + Sync,
    R: 'static + Clone + Send + Sync,
    V: IntoView + 'static,
    IV: Fn(R) -> V + Clone + 'static + Send + Sync,
>(
    action: Action<S, R, AStorage>,
    #[prop(into)] loading_message: String,
    modal: IV,
    #[prop(optional, into)] close: RwSignal<bool>,
) -> impl IntoView {
    let pending = action.pending();
    let action_value = action.value();
    let res = Signal::derive(move || {
        if pending() {
            return None;
        }
        action_value()
    });
    let show_popup = Signal::derive(move || {
        let show = (pending() || res.with(|r| r.is_some())) && !close();
        close.set(false);
        show
    });
    let modal_s = StoredValue::new(modal);
    let loading_msg_s = StoredValue::new(loading_message);

    view! {
        <ShadowOverlay show=show_popup>
            <Show
                when=move || res.with(|r| r.is_some())
                fallback=move || {
                    view! { <ActionRunningOverlay message=loading_msg_s.get_value() /> }
                }
            >
                <div class="px-4 pt-4 pb-12 mx-6 w-full lg:w-1/2 max-h-[65%] rounded-xl bg-white">
                    {move || (modal_s.get_value())(res().unwrap())}
                </div>
            </Show>
        </ShadowOverlay>
    }
    .into_any()
}
