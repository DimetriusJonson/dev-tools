use crate::{components::ui::{
    autocomplete_input::AutocompleteInputStyle, button::{Button, ButtonColor, ButtonHeight, ButtonWidth},
}, i18n::use_i18n};
use leptos::prelude::*;
use leptos_i18n::t_string;
use web_sys::MouseEvent;

use crate::components::ui::autocomplete_input::AutocompleteInput;

pub trait KeyValueTableItem {
    fn id(&self) -> String;
    fn name(&self) -> ReadSignal<String>;
    fn set_name(&self) -> WriteSignal<String>;
    fn value(&self) -> ReadSignal<String>;
    fn set_value(&self) -> WriteSignal<String>;
}

#[component]
pub fn PropertyEditor<E>(
    key_label: impl Fn() -> String + Send + Sync + 'static,
    value_label: impl Fn() -> String + Send + Sync + 'static,
    items: impl Fn() -> Vec<E> + Send + Sync + 'static,
    #[prop(into, optional)] key_options: Vec<&'static str>,
    #[prop(into, optional)] value_options: Vec<&'static str>,
    #[prop(into)] on_add: Callback<(String, String)>,
    #[prop(into)] on_delete: Callback<String>,
    #[prop(into)] on_change_key: Callback<(String, String)>,
    #[prop(into)] on_change_value: Callback<(String, String)>,
) -> impl IntoView
where
    E: KeyValueTableItem + Send + Sync + Clone + 'static,
{
    let i18n = use_i18n();

    let key_label_memo = Memo::new(move |_| key_label());
    let value_label_memo = Memo::new(move |_| value_label());

    let (key, set_key) = signal("".to_owned());
    let (value, set_value) = signal("".to_owned());

    let on_add_click = move |_: MouseEvent| {
        on_add.run((key.get_untracked(), value.get_untracked()));
        set_key.set("".to_owned());
        set_value.set("".to_owned());
    };

    view! {
        <div class="flex flex-col gap-x-1 md:gap-x-2 gap-y-2">
            <div class="flex flex-row gap-x-1 md:gap-x-2">
                <AutocompleteInput
                    class_name="sm:min-w-36".to_owned()
                    placeholder=move || key_label_memo.get()
                    input_style=AutocompleteInputStyle::Compact
                    options={key_options.clone()}
                    on_change=move |_| {}
                    value=key
                    set_value=set_key
                />
                <AutocompleteInput
                    class_name="w-full".to_owned()
                    placeholder=move || value_label_memo.get()
                    input_style=AutocompleteInputStyle::Compact
                    options={value_options.clone()}
                    on_press_enter=move |_| {
                        on_add_click(MouseEvent::new("click").unwrap())
                    }
                    value=value
                    set_value=set_value
                />
                <Button
                    label=move || "+".to_owned()
                    class_name="text-bold".to_owned()
                    button_width=ButtonWidth::OneSymbol
                    color=ButtonColor::Success
                    loading=move || false
                    disabled=move || false
                    on_click=on_add_click
                />
            </div>
        </div>

        <For
            each=move || items()
            key=|key| key.id()
            children=move |item| {
                view! {
                    <div class="group flex gap-x-1 md:gap-x-2">
                        <AutocompleteInput
                            class_name="sm:min-w-36".to_owned()
                            placeholder=move || key_label_memo.get()
                            input_style=AutocompleteInputStyle::Compact
                            options={key_options.clone()}
                            on_change={
                                let id = item.id();
                                move |value: String| {
                                  on_change_key.run((id.to_owned(), value.to_owned()));
                                }
                            }
                            value=item.name().clone()
                            set_value=item.set_name()
                        />
                        <AutocompleteInput
                            class_name="w-full".to_owned()
                            placeholder=move || value_label_memo.get()
                            input_style=AutocompleteInputStyle::Compact
                            options={value_options.clone()}
                            on_change={
                                let id = item.id();
                                move |value: String| {
                                    on_change_value.run((id.to_owned(), value));
                                }
                            }
                            value=item.value()
                            set_value=item.set_value()
                        />
                        <Button
                            label=move || "x".to_owned()
                            title=t_string!(i18n, delete_btn).to_owned()
                            class_name="hidden group-hover:block text-bold text-gray-500 hover:text-danger".to_owned()
                            button_width=ButtonWidth::OneSymbol
                            button_height=ButtonHeight::Custom
                            color=ButtonColor::Custom
                            loading=move || false
                            disabled=move || false
                            on_click={
                                let id = item.id();
                                move |_| {
                                    on_delete.run(id.to_owned());
                                }
                            }
                        />
                    </div>
                }
            }
        />

    }
}
