use codee::string::FromToStringCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;

pub struct LocalStorage;

impl LocalStorage {
    pub fn uuid_get_or_init(key: &str) -> String {
        let (uuid, set_uuid, _) = use_local_storage::<String, FromToStringCodec>(key);
        let uuid_value = uuid.get_untracked();
        if uuid_value.is_empty() {
            let new_device_id = uuid::Uuid::new_v4().to_string();
            set_uuid.set(new_device_id.clone());
            new_device_id
        } else {
            uuid_value
        }
    }
}
