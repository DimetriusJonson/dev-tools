#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::app::App;
    console_error_panic_hook::set_once();

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(body) = document.body() {
                body.set_inner_html("");
            }
        }
    }
    
    leptos::mount::mount_to_body(App);
}
