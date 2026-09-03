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

pub fn save_file_to_disk(bytes: Vec<u8>, filename: &str, mime_type: &str) -> Result<(), String> {
    let js_array = js_sys::Array::new();
    let uint8_array = unsafe { js_sys::Uint8Array::view(&bytes) };
    js_array.push(&uint8_array);

    let options = BlobPropertyBag::new();
    options.set_type(mime_type);
    let blob = Blob::new_with_u8_array_sequence_and_options(&js_array, &options)
        .map_err(|err| err.as_string().unwrap_or("Cant create blob for save file".to_owned()))?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|err| err.as_string().unwrap_or("Cant create url with blob".to_owned()))?;

    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
    {
        let anchor = document
            .create_element("a")
            .map_err(|err| err.as_string().unwrap_or("Cant create anchor element".to_owned()))?
            .dyn_into::<HtmlAnchorElement>()
            .map_err(|err| {
                err.as_string().unwrap_or("Failed cast to HtmlAnchorElement element".to_owned())
            })?;

        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();

        Url::revoke_object_url(&url)
            .map_err(|err| err.as_string().unwrap_or("Cant revoke object url".to_owned()))?;
    }

    Ok(())
}

#[cfg(feature = "ssr")]
pub fn get_browser_language() -> String {
    "en".to_owned()
}

#[cfg(not(feature = "ssr"))]
pub fn get_browser_language() -> String {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();

        let languages = navigator.languages();
        let mut best_lang = "en".to_string();

        if languages.length() > 0 {
            if let Some(lang) = languages.get(0).as_string() {
                best_lang = lang;
            }
        }

        if best_lang.starts_with("ru") { return "ru".to_string() } else { return "en".to_string() }
    }

    "en".to_string()
}

#[cfg(not(feature = "ssr"))]
pub async fn get_host_name() -> String {
    leptos::prelude::window().location().hostname().unwrap_or_default()
}

pub fn get_browser_host_info() -> Result<(String, String, Option<u16>), String> {
    #[cfg(not(feature = "ssr"))]
    {
        let loc = leptos::prelude::window().location();
        let port = loc
            .port()
            .map_err(|err| err.as_string().unwrap_or("Failed get location port".to_owned()))?;
        let port = if !port.is_empty() {
            Some(port.parse::<u16>().map_err(|err| err.to_string())?)
        } else {
            None
        };
        return Ok((
            loc.protocol().map_err(|err| {
                err.as_string().unwrap_or("Failed get location protocol".to_owned())
            })?,
            loc.host()
                .map_err(|err| err.as_string().unwrap_or("Failed get location host".to_owned()))?,
            port,
        ));
    }

    #[cfg(feature = "ssr")]
    Ok(("http".to_owned(), "localhost".to_owned(), None))
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
pub fn get_browser_width() -> Result<f64, String> {
    Ok(1024.0)
}

#[cfg(not(feature = "ssr"))]
pub fn get_browser_width() -> Result<f64, String> {
    let window = web_sys::window().ok_or_else(|| "No global window found")?;

    let width = window
        .inner_width()
        .map_err(|err| err.as_string().unwrap_or("Failed get window width".to_owned()))?
        .as_f64()
        .ok_or_else(|| "Could not convert inner_width to f64")?;

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

pub fn create_cookie(_name: &str, _value: &str, _max_age_secs: Option<u64>) -> Result<(), String> {
    #[cfg(not(feature = "ssr"))]
    {
        // Get the global window and document objects
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
        {
            // Format the standard cookie string
            let cookie_string = if let Some(_max_age_secs) = _max_age_secs {
                format!(
                    "{}={}; Path=/; Max-Age={}; Secure; SameSite=Lax",
                    _name, _value, _max_age_secs
                )
            } else {
                format!("{}={}; Path=/; Secure; SameSite=Lax", _name, _value)
            };

            if let Ok(html_document) = document.dyn_into::<web_sys::HtmlDocument>() {
                // Set the cookie via the DOM
                html_document.set_cookie(&cookie_string).map_err(|err| {
                    err.as_string().unwrap_or(format!("Cant create cookie {}", cookie_string))
                })?;
            }
        }
    }

    Ok(())
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
