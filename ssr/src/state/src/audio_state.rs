use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct AudioState {
    pub muted: RwSignal<bool>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            muted: RwSignal::new(true),
        }
    }

    pub fn get() -> Self {
        let this: Self = expect_context();
        this
    }

    pub fn toggle_mute() {
        let this: Self = expect_context();
        this.muted.update(|m| *m = !*m);
    }

    pub fn reset_to_muted() {
        let this: Self = expect_context();
        this.muted.set(true);
    }
}
