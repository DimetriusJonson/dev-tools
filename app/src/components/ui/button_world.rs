use leptos::{ev::MouseEvent, html, prelude::*};

#[component]
pub fn ButtonWorld(
    #[prop(optional)] id: i32,
    title: impl Fn() -> String + Send + Sync + 'static,
    #[prop(optional)] class_name: String,
    loading: impl Fn() -> bool + Send + Sync + 'static,
    disabled: impl Fn() -> bool + Send + Sync + 'static,
    on_click: impl FnMut(MouseEvent) + 'static,
    #[prop(optional)] node_ref: Option<NodeRef::<leptos::html::Button>>,
) -> impl IntoView {
    let loading_memo = Memo::new(move |_| loading());
    let disabled_memo = Memo::new(move |_| disabled());
    let title_memo = Memo::new(move |_| title());

    let button_element: NodeRef<html::Button> = match node_ref {
        Some(node_ref) => node_ref,
        None => NodeRef::new(),
    };

    let base_classes = "w-10 flex items-center pl-1 py-1 font-medium transition-colors duration-300 bg-transparent text-sky-500/50 hover:text-sky-500".to_owned();

    view! {
        <button
            node_ref=button_element
            id={id}
            title=move || {title_memo.get()}
            aria-label=move || {title_memo.get()}
            class=move || format!("{} {} {}", base_classes,
                match disabled_memo.get() {
                    true => "cursor-not-allowed".to_owned(),
                    false => "cursor-pointer".to_owned(),
                }, class_name)
            on:click=on_click
            on:mouseup=move |_| if let Some(button) = button_element.get() { button.blur(); }
            disabled=disabled_memo
           >

            <svg class="w-8 h-8 transition-transform duration-700" 
                class:animate-spin=move || loading_memo.get() 
                fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="10"></circle>
                <path d="M2 12h20M12 2a15.3 15.3 0 014 10 15.3 15.3 0 01-4 10 15.3 15.3 0 01-4-10 15.3 15.3 0 014-10z"></path>
            </svg>
            "\u{00A0}"

        </button>
    }
}
