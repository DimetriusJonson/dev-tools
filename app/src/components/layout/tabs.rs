use leptos::{html::Div, prelude::*};

#[derive(Clone, Debug)]
pub struct TabItem {
    pub name: String,
    pub title: String,
    pub node_ref: NodeRef<Div>,
}

#[component]
pub fn Tabs(
    #[prop(optional)] class_name: String,
    items: impl Fn() -> Vec<TabItem> + Send + Sync + 'static,
    tab_selected: ReadSignal<usize>,
    set_tab_selected: WriteSignal<usize>,
) -> impl IntoView {
    move || {
        Effect::new({
            let tabs = items();
            move |_| {
                update_selected(&tabs, tab_selected.get());
            }
        });

        Effect::watch(
            move || tab_selected.get(),
            {
                let tabs = items();
                move |value, prev, _| {
                    if prev.is_none() || value != prev.unwrap() {
                        update_selected(&tabs.clone(), *value);
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
                     key=|tab| tab.name.to_owned()
                     let(idx, tab)
                 >
                     <button role="tab"
                         title=tab.title
                         class="p-2 inline-flex flex-auto justify-center items-center gap-x-2 text-sm font-medium text-center rounded-lg disabled:opacity-50 disabled:pointer-events-none focus:outline-hidden cursor-pointer"
                         aria-selected=move || tab_selected.get() == idx.get()
                         class=(["bg-transparent", "text-neutral-800", "dark:text-neutral-400", "hover:bg-gray-600/20"], move || tab_selected.get() != idx.get())
                         class=(["bg-gray-600/50", "text-white"], move || tab_selected.get() == idx.get())
                         on:click={let tabs = tabs2.clone();
                            move |_event| {
                                set_tab_selected.set(idx.get());
                                update_selected(&tabs, idx.get());
                            }
                        }
                     >
                     {tab.name}
                     </button>
                 </ ForEnumerate>

             </nav>
        }
    }
}

fn update_selected(tabs: &[TabItem], tab_selected: usize) {
    for tab in tabs.iter() {
        if let Some(tab_elem) = tab.node_ref.get_untracked() {
            tab_elem.class_list().add_1("hidden").expect("Failed add hidden to tab element");
            tab_elem.class_list().remove_1("block").expect("Failed remove block from tab element");
        }
    }

    if let Some(tab_node) = tabs
        .iter()
        .enumerate()
        .filter(|(idx, _tab)| *idx == tab_selected)
        .map(|(_idx, tab)| tab.node_ref)
        .next()
        && let Some(tab_elem) = tab_node.get_untracked()
    {
        tab_elem.class_list().add_1("block").expect("Failed add block to tab element");
        tab_elem.class_list().remove_1("hidden").expect("Failed remove hidden from tab element");
    }
}

impl TabItem {
    pub fn new(name: &str, title: &str, node_ref: NodeRef<Div>) -> Self {
        Self { name: name.to_owned(), title: title.to_owned(), node_ref }
    }

    pub fn new_simple(name: &str, node_ref: NodeRef<Div>) -> Self {
        Self { name: name.to_owned(), title: "".to_owned(), node_ref }
    }
}
