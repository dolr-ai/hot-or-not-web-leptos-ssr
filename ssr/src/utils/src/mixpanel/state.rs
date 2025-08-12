use leptos::prelude::*;

#[derive(Clone, Default)]
pub struct MixpanelState {
    pub device_id: RwSignal<Option<String>>,
    pub custom_device_id: RwSignal<Option<String>>,
    pub metadata: RwSignal<Option<MixpanelUserMetadata>>,
}

#[derive(Clone, Default)]
pub struct MixpanelUserMetadata {
    pub email: Option<String>,
    pub signup_at: Option<i64>,
    pub user_principal: String,
}

impl MixpanelState {
    pub fn init() -> Self {
        let this = Self {
            ..Default::default()
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
        this.metadata.set(None);
    }

    pub fn get_metadata() -> RwSignal<Option<MixpanelUserMetadata>> {
        let this = use_context::<Self>().unwrap_or_else(Self::init);
        this.metadata
    }
}
