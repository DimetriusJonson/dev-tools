use leptos::prelude::*;

pub type SelectOption = (Option<String>, String);

#[component]
pub fn SelectInput(
    name: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] class_name: String,
    label: impl Fn() -> String + Send + Sync + 'static,
    not_selected_text: impl Fn() -> String + Send + Sync + 'static,
    options: impl Fn() -> Vec<SelectOption> + Send + Sync + 'static,
    #[prop(into)] on_change: Callback<String>
) -> impl IntoView {
    let label_memo = Memo::new(move |_| label());
    
    let options_memo = Memo::new(move |_| {
        let mut final_options = Vec::new();
        if !not_selected_text().is_empty() {
            final_options.push((Some("".to_owned()), not_selected_text()));
        }
        final_options.extend(options());
        final_options
    });

    view! {
        <span class={class_name}>
            <select aria-label=label_memo
                id = {name.to_owned()}
                class={"border rounded-lg block w-full p-2
            focus:outline-4

            transition-[background-color,border-color,box-shadow,color]
            duration-294

            focus:outline-blue-400/20
            dark:focus:outline-blue-200/20
            bg-white
            border-gray-300
            dark:bg-dark-bg
            dark:text-gray-50 
            dark:border-gray-700 
            hover:ring-gray-400
            hover:border-gray-400
            dark:hover:ring-gray-500
            dark:hover:border-gray-500
            focus:ring-indigo-400 
            focus:border-indigo-400
            active:ring-indigo-400 
            active:border-indigo-400 
            dark:active:ring-indigo-400 
            dark:active:border-indigo-400 
            dark:focus:ring-indigo-400 
            dark:focus:border-indigo-400

            disabled:text-weak
            disabled:bg-disabled-bg
            disabled:dark:border-bg-dark-bg
            disabled:border-bg-white
            disabled:placeholder:text-gray-500/30

            "}
                name = {name}
                bind:value=(value, set_value)
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    on_change.run(val)
                }
            >
                {move || {
                    options_memo.get().into_iter()
                    .map(|option| view! { 
                        <option class="dark:bg-dark-bg" value={option.0.to_owned()} selected={move || option.0 == Some(value.get())}>{option.1}</option>
                    }).collect::<Vec<_>>()
                }}

            </select>
        </span>
    }
}
