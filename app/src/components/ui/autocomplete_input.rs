use leptos::prelude::*;
use web_sys::KeyboardEvent;

#[component]
pub fn AutocompleteInput(
    #[prop(into)] options: Vec<&'static str>,
    #[prop(optional)] class_name: String,
    placeholder: impl Fn() -> String + Send + Sync + 'static,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(into)] on_change: Callback<String>
) -> impl IntoView {
    let (is_open, set_open) = signal(false);
    let (focused_idx, set_focused_idx) = signal::<Option<usize>>(None);
    let placeholder_memo = Memo::new(move |_| placeholder());

    let filtered_options = Memo::new(move |_| {
        let query = value.get().to_lowercase();
        if query.is_empty() {
            Vec::new()
        } else {
            options
                .iter()
                .filter(|opt| opt.to_lowercase().contains(&query))
                .cloned()
                .collect::<Vec<&'static str>>()
        }
    });

    let on_keydown = move |ev: KeyboardEvent| {
        let key = ev.key();
        let list_len = filtered_options.get().len();

        if key == "ArrowDown" {
            ev.prevent_default();
            set_focused_idx.update(|idx| {
                *idx = Some(match *idx {
                    None => 0,
                    Some(i) => (i + 1) % list_len,
                });
            });
        } else if key == "ArrowUp" {
            ev.prevent_default();
            set_focused_idx.update(|idx| {
                *idx = Some(match *idx {
                    None => list_len - 1,
                    Some(i) => {
                        if i == 0 {
                            list_len - 1
                        } else {
                            i - 1
                        }
                    }
                });
            });
        } else if key == "Enter" {
            if let Some(idx) = focused_idx.get() {
                let val = filtered_options.get()[idx].to_string();
                set_value.set(val.to_owned());
                set_open.set(false);
                on_change.run(val);
            }
        } else if key == "Escape" {
            set_open.set(false);
        }
    };

    view! {
        <div class={format!("relative w-64 dark:text-white {}", class_name)}>
            <input
                type="text"
                placeholder=placeholder_memo
                class="w-full border rounded-lg w-full p-2
        focus:outline-4
        h-8 md:h-10 text-xs md:text-base

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
        disabled:placeholder:text-gray-500/30"
                prop:value=value
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_value.set(val.to_owned());
                    set_open.set(true);
                    set_focused_idx.set(None);
                    on_change.run(val);
                }
                on:keydown=on_keydown
                on:focus=move |_| set_open.set(true)
                on:blur=move |_| {
                    set_timeout(move || set_open.set(false), std::time::Duration::from_millis(150));
                }
            />

            <Show when=move || is_open.get() && !filtered_options.read().is_empty()>
                {view! {
                        <ul class="absolute z-10 w-fit mt-1 max-h-60 overflow-auto bg-white dark:bg-dark-bg border border-gray-300 dark:border-gray-700 rounded-lg">
                            <ForEnumerate
                                each=move || filtered_options.get()
                                key=|option| option.to_owned()
                                children={move |index:ReadSignal<usize>, option: &'static str| {
                                    let opt_clone = option.to_owned();
                                    view! {
                                        <li
                                            class="px-4 py-2 text-sm text-gray-700 dark:text-gray-50 hover:bg-blue-50 hover:text-blue-900 cursor-pointer transition-colors duration-150"
                                            class=(["bg-blue-500", "text-white"], move || focused_idx.get() == Some(index.get()))
                                            on:click=move |_| {
                                                set_value.set(opt_clone.to_owned());
                                                set_open.set(false);
                                                on_change.run(opt_clone.to_owned());
                                            }
                                        >
                                            {option}
                                        </li>
                                    }
                                }}
                            />
                        </ul>
                }}
            </Show>
        </div>
    }
}
