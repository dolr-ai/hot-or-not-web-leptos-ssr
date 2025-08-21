use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct AudioState {
    pub muted: RwSignal<bool>,
    pub volume: RwSignal<f64>,
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
            volume: RwSignal::new(0.0),
        }
    }

    pub fn get() -> Self {
        let this: Self = expect_context();
        this
    }

    pub fn toggle_mute() {
        let this: Self = expect_context();
        let is_muted = this.muted.get_untracked();
        this.muted.update(|m| *m = !*m);
        this.volume.set(if is_muted { 1.0 } else { 0.0 });
    }

    pub fn reset_to_muted() {
        let this: Self = expect_context();
        this.muted.set(true);
        this.volume.set(0.0);
    }
}
