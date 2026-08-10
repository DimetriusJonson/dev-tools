#[cfg(not(feature = "ssr"))]
mod csr {
    use leptos::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = initJsonEditor)]
        pub fn init_json_editor(id: &str, initial_val: &str, callback: &JsValue);

        #[wasm_bindgen(js_name = setJsonEditorValue)]
        pub fn set_json_editor_value(new_value: &str);
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use web_sys::wasm_bindgen::JsValue;

    pub fn init_json_editor(id: &str, initial_val: &str, callback: &JsValue) {}
    pub fn set_json_editor_value(new_value: &str) {}
}

#[cfg(not(feature = "ssr"))]
pub use csr::*;
#[cfg(feature = "ssr")]
pub use ssr::*;
