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

#[cfg(feature = "ssr")]
pub async fn get_host_name() -> String {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    match extract::<HeaderMap>().await {
        Ok(headers) => headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        Err(err) => {
            eprintln!("Error:{}", err);
            err.to_string()
        },
    }
}
