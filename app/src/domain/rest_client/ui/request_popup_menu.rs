use leptos::prelude::*;

#[component]
pub fn RequestPopupMenu(
    #[prop(optional)] class_name: String,
    items: impl Fn() -> Vec<(&'static str, &'static str)> + Send + Sync + 'static,
    #[prop(into)] on_selected: Callback<(&'static str, &'static str)>,
) -> impl IntoView {
    view! {
        <div class={format!("flex flex-col bg-gray-800 rounded-xl shadow-2xl text-gray-300 w-fit h-fit whitespace-nowrap p-4 items-center {}", class_name)}>
            {move || 
                items().into_iter()
                  .map(|item| {
                    view! {
                        <div class="hover:bg-sky-500/50 cursor-pointer rounded-xl p-2 w-full"
                            on:click=move |_| {
                                on_selected.run(item);
                            }>
                            {item.1}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }
        </div>
    }
}
