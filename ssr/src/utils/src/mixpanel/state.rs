use leptos::prelude::*;

#[derive(Clone)]
pub struct MixpanelState {
    pub device_id: RwSignal<Option<String>>,
    pub custom_device_id: RwSignal<Option<String>>,
}

impl MixpanelState {
    pub fn init() -> Self {
        let this = Self {
            device_id: RwSignal::new(None),
            custom_device_id: RwSignal::new(None),
        };
        provide_context(this.clone());
        this
    }
    pub fn get_device_id() -> RwSignal<Option<String>> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.device_id
    }
    pub fn get_custom_device_id() -> RwSignal<Option<String>> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.custom_device_id
    }

    pub fn reset_device_id(device_id: String) {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.device_id.set(Some(device_id));
    }
}
