use codee::string::FromToStringCodec;
use consts::AUTH_UTIL_COOKIES_MAX_AGE_MS;
use leptos::prelude::*;
use leptos_use::{use_cookie_with_options, UseCookieOptions};

pub struct Storage;

impl Storage {
    pub fn uuid_get_or_init(key: &str) -> String {
        let (uuid, set_uuid) = use_cookie_with_options::<String, FromToStringCodec>(
            key,
            UseCookieOptions::default()
                .path("/")
                .max_age(AUTH_UTIL_COOKIES_MAX_AGE_MS),
        );
        let uuid_value = uuid.get_untracked();
        if let Some(uuid) = uuid_value {
            uuid
        } else {
            let new_device_id = uuid::Uuid::new_v4().to_string();
            set_uuid.set(Some(new_device_id.clone()));
            new_device_id
        }
    }
}
