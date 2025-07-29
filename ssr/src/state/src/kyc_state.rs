use leptos::prelude::*;

#[derive(Clone, Default, PartialEq)]
pub enum KycStatus {
    #[default]
    Pending,
    InProgress,
    Verified,
}

#[derive(Default, Clone)]
pub struct KycState {
    pub kyc_status: RwSignal<KycStatus>,
}

impl KycState {
    pub fn init() -> Self {
        let this = Self {
            ..Default::default()
        };
        provide_context(this.clone());
        this
    }

    pub fn get() -> RwSignal<KycStatus> {
        use_context::<Self>()
            .unwrap_or_else(KycState::init)
            .kyc_status
    }

    pub fn is_verified() -> bool {
        let this = use_context::<Self>().unwrap_or_else(KycState::init);
        this.kyc_status.get() == KycStatus::Verified
    }

    pub fn set_status(status: KycStatus) {
        let this = use_context::<Self>().unwrap_or_else(KycState::init);
        this.kyc_status.set(status);
    }

    pub fn toggle() {
        let this = use_context::<Self>().unwrap_or_else(KycState::init);
        this.kyc_status.update(|v| {
            *v = match *v {
                KycStatus::Pending => KycStatus::InProgress,
                KycStatus::InProgress => KycStatus::Verified,
                KycStatus::Verified => KycStatus::Pending,
            };
        });
    }
}
