#[cfg(not(feature = "ssr"))]
mod csr {
    use leptos::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = initCodeEditor)]
        pub fn init_code_editor(id: &str, initial_val: &str, callback: &JsValue);

        #[wasm_bindgen(js_name = setCodeEditorValue)]
        pub fn set_code_editor_value(new_value: &str);

        #[wasm_bindgen(js_name = codeEditorChangeLang)]
        pub fn code_editor_change_lang(lang: &str, callback: &JsValue);
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use web_sys::wasm_bindgen::JsValue;

    pub fn init_code_editor(_id: &str, _initial_val: &str, _callback: &JsValue) {}
    pub fn set_code_editor_value(_new_value: &str) {}

    pub fn code_editor_change_lang(_lang: &str, _callback: &JsValue) {}
}

#[cfg(not(feature = "ssr"))]
pub use csr::*;
#[cfg(feature = "ssr")]
pub use ssr::*;
