use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = initCodeEditor)]
    pub fn init_code_editor(
        id: &str,
        initial_val: &str,
        _lang: &str,
        read_only: bool,
        callback: &JsValue,
    ) -> JsValue;

    #[wasm_bindgen(js_name = setCodeEditorValue)]
    pub fn set_code_editor_value(editor_view: &JsValue, new_value: &str);

    #[wasm_bindgen(js_name = codeEditorChangeLang)]
    pub fn code_editor_change_lang(
        editor_view: &JsValue,
        lang: &str,
        read_only: bool,
        callback: &JsValue,
    );
}
