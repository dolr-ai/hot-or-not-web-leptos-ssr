use leptos::prelude::*;
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};

struct DisplayMutedIconTimeout {
    // TODO: use TAIT once stable
    // instead of Dyn dispatch
    start: Box<dyn Fn(()) + Send + Sync>,
    stop: Box<dyn Fn() + Send + Sync>,
    _is_pending: Signal<bool>,
    show_mute_icon: RwSignal<bool>,
}

impl DisplayMutedIconTimeout {
    fn new(show_mute_icon: RwSignal<bool>) -> Self {
        let UseTimeoutFnReturn {
            start,
            stop,
            is_pending,
            ..
        } = use_timeout_fn(
            move |()| {
                show_mute_icon.set(false);
            },
            // 6 secs
            6000f64,
        );
        Self {
            start: Box::new(start),
            stop: Box::new(stop),
            _is_pending: is_pending,
            show_mute_icon,
        }
    }

    fn flash(&self) {
        self.show_mute_icon.set(true);
        (self.start)(());
    }

    fn stop(&self) {
        self.show_mute_icon.set(false);
        (self.stop)();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AudioState {
    pub show_mute_icon: RwSignal<bool>,
    pub muted: RwSignal<bool>,
    display_flash: StoredValue<DisplayMutedIconTimeout>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioState {
    pub fn new() -> Self {
        let show_mute_icon = RwSignal::new(true);
        let display_flash = StoredValue::new(DisplayMutedIconTimeout::new(show_mute_icon));
        Self {
            show_mute_icon,
            muted: RwSignal::new(true),
            display_flash,
        }
    }

    pub fn get() -> Self {
        let this: Self = expect_context();

        if this.show_mute_icon.get_untracked() {
            this.display_flash.with_value(|d| d.flash());
        }

        this
    }

    pub fn toggle_mute() {
        let this: Self = expect_context();
        if !this.muted.get_untracked() {
            this.display_flash.with_value(|d| d.flash());
        } else {
            this.display_flash.with_value(|d| d.stop());
        }
        this.muted.update(|m| *m = !*m);
    }

    pub fn reset_to_muted() {
        let this: Self = expect_context();
        this.muted.set(true);
        this.display_flash.with_value(|d| d.stop());
    }
}
