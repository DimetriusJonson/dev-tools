use std::time::Duration;

use leptos::prelude::{GetUntracked, RwSignal, Set, set_timeout};
use web_sys::{
    Blob, BlobPropertyBag, HtmlAnchorElement, Url, js_sys,
    wasm_bindgen::{JsCast, JsValue},
};

pub fn copy_to_clipboard(_data: &str) {
    #[cfg(not(feature = "ssr"))]
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let _ = clipboard.write_text(_data);
    }
}

pub async fn paste_from_clipboard() -> Option<String> {
    #[cfg(not(feature = "ssr"))]
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        if let Ok(str) = clipboard.read_text().await {
            return str.as_string();
        }
    }

    None
}

pub fn save_file_to_disk(bytes: Vec<u8>, filename: &str, mime_type: &str) -> Result<(), JsValue> {
    let js_array = js_sys::Array::new();
    let uint8_array = unsafe { js_sys::Uint8Array::view(&bytes) };
    js_array.push(&uint8_array);

    let options = BlobPropertyBag::new();
    options.set_type(mime_type);
    let blob = Blob::new_with_u8_array_sequence_and_options(&js_array, &options)?;

    let url = Url::create_object_url_with_blob(&blob)?;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let anchor = document.create_element("a")?.dyn_into::<HtmlAnchorElement>()?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    Url::revoke_object_url(&url)?;

    Ok(())
}

#[cfg(feature = "ssr")]
pub fn get_browser_language() -> String {
    "en".to_owned()
}

#[cfg(not(feature = "ssr"))]
pub fn get_browser_language() -> String {
    let window = web_sys::window().expect("window should exist");
    let navigator = window.navigator();

    let languages = navigator.languages();
    let mut best_lang = "en".to_string();

    if languages.length() > 0 {
        if let Some(lang) = languages.get(0).as_string() {
            best_lang = lang;
        }
    }

    if best_lang.starts_with("ru") { "ru".to_string() } else { "en".to_string() }
}

#[cfg(not(feature = "ssr"))]
pub async fn get_host_name() -> String {
    leptos::prelude::window().location().hostname().unwrap_or_default()
}

pub fn get_browser_host_info() -> (String, String, Option<u16>) {
    #[cfg(not(feature = "ssr"))]
    {
        let loc = leptos::prelude::window().location();
        let port = loc.port().unwrap();
        let port = if !port.is_empty() { Some(port.parse::<u16>().unwrap()) } else { None };
        return (loc.protocol().unwrap(), loc.host().unwrap(), port);
    }

    #[cfg(feature = "ssr")]
    ("http".to_owned(), "localhost".to_owned(), None)
}

#[cfg(feature = "ssr")]
pub async fn get_host_name() -> String {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let host = match extract::<HeaderMap>().await {
        Ok(headers) => headers.get("host").and_then(|h| h.to_str().ok()).map(|s| s.to_string()),
        Err(_) => None,
    };

    match host {
        Some(host) => host,
        None => match extract::<axum::http::request::Parts>().await {
            Ok(parts) => parts.uri.authority().map(|a| a.host().to_owned()).unwrap_or_default(),
            Err(err) => err.to_string(),
        },
    }
}

pub fn get_accept_language() -> String {
    #[cfg(not(feature = "ssr"))]
    let val = leptos::prelude::window().navigator().language().unwrap_or("en-US".to_owned());

    #[cfg(feature = "ssr")]
    let val = "en-US".to_owned();

    val
}

pub fn single_select_option(value: &str) -> (Option<String>, String) {
    (Some(value.to_owned()), value.to_owned())
}

#[cfg(feature = "ssr")]
pub fn get_browser_width() -> Result<f64, JsValue> {
    Ok(1024.0)
}

#[cfg(not(feature = "ssr"))]
pub fn get_browser_width() -> Result<f64, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No global window found"))?;

    let width = window
        .inner_width()?
        .as_f64()
        .ok_or_else(|| JsValue::from_str("Could not convert inner_width to f64"))?;

    Ok(width)
}

#[cfg(feature = "ssr")]
pub fn get_browser_height() -> Result<f64, JsValue> {
    Ok(768.0)
}

#[cfg(not(feature = "ssr"))]
pub fn get_browser_height() -> Result<f64, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No global window found"))?;

    let width = window
        .inner_height()?
        .as_f64()
        .ok_or_else(|| JsValue::from_str("Could not convert inner_height to f64"))?;

    Ok(width)
}

pub fn safe_updating_ui_value(
    update_lock: RwSignal<bool>,
    update_fn: impl Fn() + Send + Sync + 'static,
) {
    if !update_lock.get_untracked() {
        update_lock.set(true);
        set_timeout(
            move || {
                update_fn();
                set_timeout(move || update_lock.set(false), Duration::from_millis(1));
            },
            Duration::from_millis(1),
        );
    }
}

pub fn create_cookie(_name: &str, _value: &str, _max_age_secs: Option<u64>) {
    #[cfg(not(feature = "ssr"))]
    {
        // Get the global window and document objects
        let window = web_sys::window().expect("No global window found");
        let document = window.document().expect("No document found on window");

        // Format the standard cookie string
        let cookie_string = if let Some(_max_age_secs) = _max_age_secs {
            format!("{}={}; Path=/; Max-Age={}; Secure; SameSite=Lax", _name, _value, _max_age_secs)
        } else {
            format!("{}={}; Path=/; Secure; SameSite=Lax", _name, _value)
        };

        let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();

        // Set the cookie via the DOM
        html_document.set_cookie(&cookie_string).expect("Failed to write cookie");
    }
}

pub fn remove_cookie(_name: &str, _path: &str) {
    #[cfg(not(feature = "ssr"))]
    {
        use web_sys::HtmlDocument;
        let document = gloo_utils::document().unchecked_into::<HtmlDocument>();

        let cookie_str = format!("{}=''; max-age=0; path={}", _name, _path);

        let _ = document.set_cookie(&cookie_str);
    }
}
