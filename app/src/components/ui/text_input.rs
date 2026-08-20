use leptos::{html::Input, prelude::*};
use web_sys::wasm_bindgen::JsCast;

#[component]
pub fn TextInput(
    name: String,
    #[prop(optional)] class_name: String,
    #[prop(optional)] node_ref: NodeRef<Input>,
    placeholder: impl Fn() -> String + Send + Sync + 'static,
    input_type: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] on_cancel_change: Option<Callback<()>>,
    #[prop(into, optional)] on_press_enter: Option<Callback<()>>,
) -> impl IntoView {
    let placeholder_memo = Memo::new(move |_| placeholder());

    view! {
            <input node_ref=node_ref
                class={format!("px-1 md:px-4 py-2 rounded-md shadow-inner
            text-gray-700 
            placeholder:text-gray-500 
            dark:text-gray-50 

            autofill:bg-blue-300/20  
            dark:autofill:bg-gray-50

            focus:outline-4
            border

            transition-[background-color,border-color,box-shadow,color]
            duration-294
            
            hover:ring-gray-400
            hover:border-gray-400
            dark:hover:ring-gray-500
            dark:hover:border-gray-500
            bg-white
            dark:bg-dark-bg
            border-gray-300
            dark:border-gray-700
            focus:ring-indigo-400 
            focus:border-indigo-400
            active:ring-indigo-400 
            active:border-indigo-400 
            dark:active:ring-indigo-400 
            dark:active:border-indigo-400 
            dark:focus:ring-indigo-400 
            dark:focus:border-indigo-400
            focus:outline-blue-200/20
            aria-invalid:border-danger
            aria-invalid:dark:border-danger
            aria-invalid:dark:bg-danger-bg
            aria-invalid:focus:ring-danger 
            aria-invalid:focus:border-danger
            aria-invalid:active:ring-danger 
            aria-invalid:active:border-danger
            aria-invalid:focus:outline-danger/20
            aria-invalid:dark:focus:ring-danger 
            aria-invalid:dark:focus:border-danger 
            aria-invalid:dark:active:ring-danger 
            aria-invalid:dark:active:border-danger

            disabled:text-weak
            disabled:bg-disabled-bg
            disabled:dark:border-bg-dark-bg
            disabled:border-bg-white
            disabled:placeholder:text-gray-500/30

            {}
            ", class_name)}
                type=input_type
                name=name.to_owned()
                placeholder=placeholder_memo
                title=placeholder_memo
                bind:value=(value, set_value)
                on:change=move |ev| {
                    if let Some(on_change) = on_change {
                        let val = event_target_value(&ev);
                        on_change.try_run(val);
                    }
                }
                on:keydown=move |ev| {
                    if let Ok(key_event) = ev.dyn_into::<web_sys::KeyboardEvent>() {
                        let key = key_event.key();
                        if key == "Escape" && let Some(on_cancel_change) = on_cancel_change {
                            on_cancel_change.try_run(());
                        } else if key == "Enter" && let Some(on_press_enter) = on_press_enter {
                            on_press_enter.try_run(());
                        }
                    }
                }
            />
    }
}
