use leptos::prelude::*;

#[component]
pub fn RequestPopupMenu(
    #[prop(optional)] class_name: String,
    items: impl Fn() -> Vec<(&'static str, &'static str, bool)> + Send + Sync + 'static,
    #[prop(into)] on_selected: Callback<(&'static str, &'static str)>,
) -> impl IntoView {
    view! {
        <div class={format!("flex flex-col bg-gray-800 rounded-xl shadow-2xl text-gray-300 w-fit h-fit whitespace-nowrap p-2 items-center {}", class_name)}>
            {move || 
                items().into_iter()
                  .map(|item| {
                    view! {
                        <div class="hover:bg-sky-500/50 cursor-pointer p-2 w-full"
                            class=(["border-b", "border-gray-600"], move || item.2)
                            on:click=move |_| {
                                on_selected.run((item.0, item.1));
                            }>
                            {item.1}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }
        </div>
    }
}
