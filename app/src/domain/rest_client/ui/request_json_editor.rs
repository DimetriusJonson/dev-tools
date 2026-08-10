use std::time::Duration;

use leptos::prelude::*;
use web_sys::wasm_bindgen::prelude::Closure;

use crate::{
    code_mirror::{init_json_editor, set_json_editor_value},
    domain::rest_client::model::request_params::RequestInfo,
};

#[component]
pub fn RequestJsonEditor(
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    request_info: ReadSignal<RequestInfo>,
) -> impl IntoView {
    let (value_updating, set_value_updating) = signal(false);

    let on_change = Closure::wrap(Box::new(move |new_val: String| {
        if !value_updating.get_untracked() {
            set_value.set(new_val);
        }
    }) as Box<dyn FnMut(String)>);

    Effect::new(move |_| {
        init_json_editor("request-json-editor", &value.get_untracked(), on_change.as_ref());
    });

    Effect::watch(
        move || request_info.get(),
        move |_request_info, _prev, _| {
            set_value_updating.set(true);
            set_timeout(
                move || {
                    set_json_editor_value(&value.read_untracked());
                    set_timeout(move || set_value_updating.set(false), Duration::from_millis(1));
                },
                std::time::Duration::from_millis(1),
            );
        },
        false,
    );

    view! {
        <div id="request-json-editor" class="w-full text-white border"></div>
    }
}
