use std::time::Duration;

use leptos::prelude::*;
use web_sys::wasm_bindgen::{JsValue, prelude::Closure};

use crate::code_mirror::{code_editor_change_lang, init_code_editor, set_code_editor_value};

#[component]
pub fn CodeMirrorEditor(
    element_id: String,
    class_name: impl Fn() -> String + Send + Sync + 'static,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    lang: ReadSignal<String>,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let (value_updating, set_value_updating) = signal(false);
    let (editor_view, set_editor_view) = signal(JsValue::null());

    let on_change = Closure::wrap(Box::new(move |new_val: String| {
        if !value_updating.get_untracked() {
            set_value.set(new_val);
        }
    }) as Box<dyn FnMut(String)>);

    Effect::new({
        let element_id = element_id.to_owned();
        move |_| {
            let view = init_code_editor(&element_id.to_owned(), &value.get_untracked(), read_only, on_change.as_ref());
            set_editor_view.set(view);
        }
    });

    Effect::watch(
        move || value.get(),
        move |_value, _prev, _| {
            if !value_updating.get_untracked() {
                set_value_updating.set(true);
                set_timeout(
                    {
                        move || {
                            set_code_editor_value(&editor_view.read_untracked(), &value.read_untracked());
                            set_timeout(
                                move || set_value_updating.set(false),
                                Duration::from_millis(1),
                            );
                        }
                    },
                    std::time::Duration::from_millis(1),
                );
            }
        },
        false,
    );

    let on_change = Closure::wrap(Box::new(move |new_val: String| {
        if !value_updating.get_untracked() {
            set_value_updating.set(true);
            set_timeout(
                {
                    move || {
                        set_value.set(new_val);
                        set_timeout(
                            move || set_value_updating.set(false),
                            Duration::from_millis(1),
                        );
                    }
                },
                std::time::Duration::from_millis(1),
            );
        }
    }) as Box<dyn FnMut(String)>);

    Effect::watch(
        move || lang.get(),
        move |lang, _prev, _| {
            code_editor_change_lang(&editor_view.read_untracked(), lang, read_only, on_change.as_ref());
        },
        false,
    );

    view! {
        <div id={element_id.to_owned()} class=move || {format!("w-full h-0 px-1 md:px-4 py-2 bg-white dark:bg-dark-bg {}", class_name())}>
        </div>
    }
}
