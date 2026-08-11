use std::time::Duration;

use leptos::prelude::*;
use web_sys::wasm_bindgen::prelude::Closure;

use crate::code_mirror::{code_editor_change_lang, init_code_editor, set_code_editor_value};

#[component]
pub fn CodeMirrorEditor<M>(
    element_id: String,
    #[prop(optional)] class_name: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    value_monitor: ReadSignal<M>,
    lang: ReadSignal<String>,
) -> impl IntoView
where
    M: Clone + Send + Sync + 'static,
{
    let (value_updating, set_value_updating) = signal(false);

    let on_change = Closure::wrap(Box::new(move |new_val: String| {
        if !value_updating.get_untracked() {
            set_value.set(new_val);
        }
    }) as Box<dyn FnMut(String)>);

    Effect::new({
        let element_id = element_id.to_owned();
        move |_| {
            init_code_editor(&element_id.to_owned(), &value.get_untracked(), on_change.as_ref());
        }
    });

    Effect::watch(
        move || value_monitor.get(),
        move |_request_info, _prev, _| {
            set_value_updating.set(true);
            set_timeout(
                {
                    move || {
                        set_code_editor_value(&value.read_untracked());
                        set_timeout(
                            move || set_value_updating.set(false),
                            Duration::from_millis(1),
                        );
                    }
                },
                std::time::Duration::from_millis(1),
            );
        },
        false,
    );

    let on_change = Closure::wrap(Box::new(move |new_val: String| {
        if !value_updating.get_untracked() {
            set_value.set(new_val);
        }
    }) as Box<dyn FnMut(String)>);

    Effect::watch(
        move || lang.get(),
        move |lang, _prev, _| {
            code_editor_change_lang(lang, on_change.as_ref());
        },
        false,
    );

    view! {
        <div id={element_id.to_owned()} 
            class={format!("w-full h-0 px-1 md:px-4 py-2 rounded-md shadow-inner border
                text-gray-700 dark:text-gray-50
                bg-white dark:bg-dark-bg
                border-gray-300 dark:border-gray-700 {}", class_name)}
        ></div>
    }
}
