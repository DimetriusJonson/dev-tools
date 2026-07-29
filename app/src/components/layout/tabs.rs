use leptos::{html::Div, prelude::*};

#[component]
pub fn Tabs(
    items: impl Fn() -> Vec<(&'static str, NodeRef<Div>)> + Send + Sync + 'static,
    tab_selected: ReadSignal<usize>,
    set_tab_selected: WriteSignal<usize>,
) -> impl IntoView {
    move || {
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
             <div class="flex border-b border-gray-200 text-sm font-medium text-center focus:outline-none" role="tablist">

                 <ForEnumerate
                     each=move || tabs.clone()
                     key=|tab| tab.0.to_owned()
                     let(idx, tab)
                 >
                     <button role="tab"
                         aria-selected=move || tab_selected.get() == idx.get()
                         class="flex-1 py-2.5 border-b-2 cursor-pointer"
                         class=(["border-blue-600", "text-black", "dark:text-white"], move || tab_selected.get() == idx.get())
                         class=(["text-gray-500"], move || tab_selected.get() != idx.get())
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

             </div>
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
