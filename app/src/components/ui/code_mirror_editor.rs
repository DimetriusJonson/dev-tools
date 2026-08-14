use leptos::prelude::*;

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
    #[cfg(not(feature = "ssr"))]
    {
        use web_sys::wasm_bindgen::{JsValue, prelude::Closure};
        use crate::{
            code_mirror::{code_editor_change_lang, init_code_editor, set_code_editor_value},
            common::ui_utils::safe_updating_ui_value,
        };

        let update_lock = RwSignal::new(false);
        let (editor_view, set_editor_view) = signal(JsValue::null());
        let on_change = Closure::wrap(Box::new(move |new_val: String| {
            safe_updating_ui_value(update_lock, move || set_value.set(new_val.to_owned()));
        }) as Box<dyn FnMut(String)>);

        Effect::new({
            let element_id = element_id.to_owned();
            move |_| {
                let view = init_code_editor(
                    &element_id.to_owned(),
                    &value.get_untracked(),
                    read_only,
                    on_change.as_ref(),
                );
                set_editor_view.set(view);
            }
        });

        Effect::watch(
            move || value.get(),
            move |_value, _prev, _| {
                safe_updating_ui_value(update_lock, move || {
                    set_code_editor_value(&editor_view.read_untracked(), &value.read_untracked())
                });
            },
            false,
        );

        let on_change = Closure::wrap(Box::new(move |new_val: String| {
            safe_updating_ui_value(update_lock, move || set_value.set(new_val.to_owned()));
        }) as Box<dyn FnMut(String)>);

        Effect::watch(
            move || lang.get(),
            move |lang, _prev, _| {
                code_editor_change_lang(
                    &editor_view.read_untracked(),
                    lang,
                    read_only,
                    on_change.as_ref(),
                );
            },
            false,
        );
    }

    #[cfg(feature = "ssr")]
    {
        // prevent compile warning
        use leptos::leptos_dom::logging::console_log;
        if read_only {
            console_log(&lang.read_untracked());
            *set_value.write_untracked() = "".to_owned();
            console_log(&value.read_untracked());
        }
    }

    view! {
        <div id={element_id.to_owned()} class={format!("w-full h-0 px-1 md:px-4 py-2 bg-white dark:bg-dark-bg {}", class_name)}
            class:hidden={ move || hidden.as_ref().map(|v| v()).unwrap_or_default()}
        >
        </div>
    }
}
