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
    pub fn get_device_id() -> Option<String> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.device_id.get()
    }
    pub fn get_device_id_untracked() -> Option<String> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.device_id.get_untracked()
    }
    pub fn set_device_id(device_id: String) {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.device_id.set(Some(device_id));
    }
    pub fn get_custom_device_id() -> Option<String> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.custom_device_id.get()
    }
    pub fn get_custom_device_id_untracked() -> Option<String> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.custom_device_id.get_untracked()
    }
    pub fn set_custom_device_id(custom_device_id: String) {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.custom_device_id.set(Some(custom_device_id));
    }
}
