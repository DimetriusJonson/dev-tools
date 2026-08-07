use leptos::{html::Div, prelude::*};

#[component]
pub fn Tabs(
    #[prop(optional)] class_name: String,
    items: impl Fn() -> Vec<(&'static str, NodeRef<Div>)> + Send + Sync + 'static,
    tab_selected: ReadSignal<usize>,
    set_tab_selected: WriteSignal<usize>,
) -> impl IntoView {
    move || {
        Effect::new({
            let tabs = items();
            move |_| {
                update_selected(tabs.clone(), tab_selected.get());
            }
        });

        Effect::watch(
            move || tab_selected.get(),
            {
                let tabs = items();
                move |value, prev, _| {
                    if prev.is_none() || value != prev.unwrap() {
                        update_selected(tabs.clone(), *value);
                    }
                }
            },
            false,
        );

        let tabs = items();
        let tabs2 = items();
        view! {
             <nav class={format!("flex gap-x-1 {}", class_name)} aria-label="Tabs" role="tablist" aria-orientation="horizontal">

                 <ForEnumerate
                     each=move || tabs.clone()
                     key=|tab| tab.0.to_owned()
                     let(idx, tab)
                 >
                     <button role="tab"
                         class="p-2 inline-flex flex-auto justify-center items-center gap-x-2 text-sm font-medium text-center rounded-lg disabled:opacity-50 disabled:pointer-events-none focus:outline-hidden cursor-pointer"
                         aria-selected=move || tab_selected.get() == idx.get()
                         class=(["bg-transparent", "text-gray-500", "dark:text-neutral-400", "hover:text-white", "focus:text-sky-500/50", "hover:bg-gray-600/50"], move || tab_selected.get() != idx.get())
                         class=(["bg-sky-500/50", "text-white", "hover:text-white", "focus:text-white", "dark:focus:text-white"], move || tab_selected.get() == idx.get())
                         on:click={
                             let tabs = tabs2.clone();
                             move |_event| {
                                 set_tab_selected.set(idx.get());
                                 update_selected(tabs.clone(), tab_selected.get());
                             }
                         }
                     >
                     {tab.0}
                     </button>
                 </ ForEnumerate>

             </nav>
        }
    }
}

fn update_selected(tabs: Vec<(&'static str, NodeRef<Div>)>, tab_selected: usize) {
    for (idx, tab) in tabs.iter().enumerate() {
        if let Some(tab_elem) = tab.1.get_untracked() {
            if idx == tab_selected {
                tab_elem.class_list().add_1("block").unwrap();
                tab_elem.class_list().remove_1("hidden").unwrap();
            } else {
                tab_elem.class_list().add_1("hidden").unwrap();
                tab_elem.class_list().remove_1("block").unwrap();
            }
        }
    }
}
