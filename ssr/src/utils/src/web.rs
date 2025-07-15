use gloo::file::ObjectUrl;
use leptos_use::use_window;

#[derive(Clone)]
pub struct FileWithUrl {
    pub file: gloo::file::File,
    pub url: ObjectUrl,
}

impl FileWithUrl {
    #[cfg(feature = "hydrate")]
    pub fn new(file: gloo::file::File) -> Self {
        let url = ObjectUrl::from(file.clone());
        Self { file, url }
    }
}

/// Share a URL with the Web Share API
/// returns None if the API is not available
pub fn share_url(url: &str) -> Option<()> {
    #[cfg(not(feature = "hydrate"))]
    {
        _ = url;
        None
    }
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;
        use web_sys::{js_sys::Reflect, ShareData};
        let window = use_window();
        let nav = window.navigator()?;
        if !Reflect::has(&nav.clone().into(), &JsValue::from_str("share")).unwrap_or_default() {
            return None;
        }
        let share_data = ShareData::new();
        share_data.set_url(url);
        _ = nav.share_with_data(&share_data);
        Some(())
    }
}

/// Copy text to clipboard
/// returns None if the API is not available
pub fn copy_to_clipboard(text: &str) -> Option<()> {
    let navigator = use_window().navigator()?;
    _ = navigator.clipboard().write_text(text);
    Some(())
}

/// Paste text from clipboard
/// passes None if the API is not available
/// or if the clipboard is unavailable
pub async fn paste_from_clipboard() -> Option<String> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen_futures::JsFuture;

        let navigator = use_window().navigator()?;
        let text_prom = navigator.clipboard().read_text();
        let text_val = JsFuture::from(text_prom).await.ok()?;
        text_val.as_string()
    }
    #[cfg(not(feature = "hydrate"))]
    {
        None
    }
}
