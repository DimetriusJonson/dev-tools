use leptos::{ev::MouseEvent, html, prelude::*};

#[derive(Default)]
pub enum ButtonColor {
    #[default]
    Primary,
    Success,
    Light,
    Danger,
}

#[derive(Default)]
pub enum ButtonTextSize {
    Sm,
    #[default]
    Md,
}

#[derive(Default)]
pub enum ButtonWidth {
    #[default]
    Auto,
    OneSymbol,
    Md,
    Lg
}

#[component]
pub fn Button(
    #[prop(optional)] id: i32,
    label: impl Fn() -> String + Send + Sync + 'static,
    #[prop(optional)] class_name: String,
    #[prop(optional)] color: ButtonColor,
    #[prop(optional)] button_width: ButtonWidth,
    loading: impl Fn() -> bool + Send + Sync + 'static,
    disabled: impl Fn() -> bool + Send + Sync + 'static,
    on_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    let label_memo = Memo::new(move |_| label());
    let loading_memo = Memo::new(move |_| loading());
    let disabled_memo = Memo::new(move |_| disabled());

    let button_element: NodeRef<html::Button> = NodeRef::new();

    let base_classes = "rounded-3xl font-medium px-4 py-1 md:py-2 h-8dvh md:h-10 justify-center items-center text-sm md:text-base transition-[background-color,border-color,box-shadow,color] duration-294".to_owned();

    let variant_classes = match color {
        ButtonColor::Primary => "bg-primary hover:bg-primary/80 text-black".to_owned(),
        ButtonColor::Success => "bg-emerald-800 hover:bg-emerald-800/80 text-white".to_owned(),
        ButtonColor::Light => {
            "bg-gray-200 dark:hover:bg-gray-50 hover:bg-gray-300 text-black".to_owned()
        }
        ButtonColor::Danger => "bg-red-800 hover:bg-red-800/80 text-white".to_owned(),
    };

    let button_width_classes = match button_width {
        ButtonWidth::Auto => "w-auto".to_owned(),
        ButtonWidth::OneSymbol => "w-14".to_owned(),
        ButtonWidth::Md => "w-32".to_owned(),
        ButtonWidth::Lg => "w-38".to_owned(),
    };

    view! {
        <button
            node_ref=button_element
            id={id}
            aria-label=move || label_memo.get()
            class=move || format!("{} {} {} {} {} {}", base_classes, variant_classes, button_width_classes, 
                match loading_memo.get() {
                    true => "inline-flex leading-6 transition ease-in-out duration-150".to_owned(),
                    _ => "".to_owned()
                }, 
                match loading_memo.get() || disabled_memo.get() {
                    true => "cursor-not-allowed".to_owned(),
                    false => "cursor-pointer".to_owned(),
                }, class_name)
            on:click=on_click
            on:mouseup=move |_| if let Some(button) = button_element.get() { button.blur().unwrap(); }
            disabled=disabled_memo
           >

           <Show
                when=move || loading_memo.get()
                fallback=move || view! { {label_memo} }
            >
                <svg class="animate-spin [animation-duration:500ms] h-5 w-5 text-black" xmlns="http://w3.org" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                "\u{00A0}"
            </Show>

        </button>
    }
}
