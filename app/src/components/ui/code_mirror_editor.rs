use crate::{
    code_mirror::{code_editor_change_lang, init_code_editor, set_code_editor_value},
    common::ui_utils::safe_updating_ui_value,
};
use leptos::prelude::*;
use web_sys::wasm_bindgen::{JsValue, prelude::Closure};

#[component]
pub fn CodeMirrorEditor(
    element_id: String,
    #[prop(optional)] class_name: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    lang: ReadSignal<String>,
    #[prop(optional)] hidden: Option<Box<dyn Fn() -> bool + Send + Sync + 'static>>,
    #[prop(optional)] read_only: bool,
) -> impl IntoView {
    let update_lock = RwSignal::new(false);
    let (editor_view, set_editor_view) = signal(JsValue::null());

    Effect::watch(
        move || value.get(),
        {
            let element_id = element_id.to_owned();
            move |_value, _prev, _| {
                if editor_view.read_untracked().is_null() {
                    let on_change = Closure::wrap(Box::new(move |new_val: String| {
                        safe_updating_ui_value(update_lock, move || {
                            set_value.set(new_val.to_owned())
                        });
                    }) as Box<dyn FnMut(String)>);

                    set_editor_view.set(init_code_editor(
                        &element_id.to_owned(),
                        &value.get_untracked(),
                        read_only,
                        on_change.as_ref(),
                    ));

                    on_change.forget();
                }

                safe_updating_ui_value(update_lock, move || {
                    set_code_editor_value(&editor_view.read_untracked(), &value.read_untracked());
                });
            }
        },
        false,
    );

    Effect::watch(
        move || lang.get(),
        move |lang, _prev, _| {
            if !editor_view.read_untracked().is_null() {
                let on_change = Closure::wrap(Box::new(move |new_val: String| {
                    safe_updating_ui_value(update_lock, move || set_value.set(new_val.to_owned()));
                }) as Box<dyn FnMut(String)>);

                code_editor_change_lang(
                    &editor_view.read_untracked(),
                    lang,
                    read_only,
                    on_change.as_ref(),
                );

                on_change.forget();
            }
        },
        false,
    );

    view! {
        <div id={element_id.to_owned()} class={format!("w-full h-0 bg-white dark:bg-dark-bg {}", class_name)}
            class:hidden={ move || hidden.as_ref().map(|v| v()).unwrap_or_default()}
        >
        </div>
    }
}
