use crate::components::ui::button::{Button, ButtonColor, ButtonWidth};
use crate::components::ui::text_input::TextInput;
use crate::domain::rest_client::ui::request_params::{RequestBodyFormValue, RequestParams};
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn RequestBodyFormPanel(
    params: ReadSignal<RequestParams>,
) -> impl IntoView {
    let i18n = use_i18n();

    let (param_name, set_param_name) = signal("".to_owned());
    let (param_value, set_param_value) = signal("".to_owned());

    view! {
        <div class="flex flex-col gap-1 md:gap-4 {}">
            <div class="flex flex-row gap-1 md:gap-4">

                <TextInput
                    name="param_name".to_owned()
                    input_type="text".to_owned()
                    class_name="sm:min-w-36".to_owned()
                    placeholder=move || {t!(i18n, rest_client_param_name_placeholder).to_html()}
                    value=param_name
                    set_value=set_param_name
                    on_change=move |_| {}
                />

                <TextInput
                    name="param_value".to_owned()
                    input_type="text".to_owned()
                    class_name="w-full".to_owned()
                    placeholder=move || {t!(i18n, rest_client_param_value_placeholder).to_html()}
                    value=param_value
                    set_value=set_param_value
                    on_change=move |_| {}
                />

                <Button
                    label=move || "+".to_owned()
                    class_name="text-bold".to_owned()
                    button_width=ButtonWidth::OneSymbol
                    color=ButtonColor::Success
                    loading=move || false
                    disabled=move || false
                    on_click=move |_| {
                        let name_converted = param_name.get_untracked().to_lowercase();
                        if !name_converted.is_empty() && params.read_untracked().body_formencoded.read_untracked().iter().find(|fv|fv.name.read_untracked().to_lowercase() == name_converted).is_none() {

                            let id = params.read_untracked().body_formencoded.read_untracked().iter().map(|fv|fv.id).max().unwrap_or_default() + 1;
                            let (name, set_name) = signal(param_name.get_untracked());
                            let (value, set_value) = signal(param_value.get_untracked());

                            params.read_untracked().set_body_formencoded.write().push(RequestBodyFormValue{ id, name, set_name, value, set_value });
                            set_param_name.set("".to_owned());
                            set_param_value.set("".to_owned());
                        }
                    }
                />
            </div>
        </div>

        <For
            each=move || params.read_untracked().body_formencoded.get()
            key=|form_value| form_value.id
            children=move |form_value| {
                view! {
                    <div class="flex gap-1 md:gap-4">

                        <TextInput
                            name={format!("param_name_{}", form_value.id)}
                            input_type="text".to_owned()
                            class_name="sm:min-w-36".to_owned()
                            placeholder=move || {t!(i18n, rest_client_param_name_placeholder).to_html()}
                            on_change=move |value: String| {
                                params.read_untracked().set_body_formencoded.write().iter_mut()
                                    .filter(|fv|fv.id == form_value.id)
                                    .for_each(|fv| {fv.set_name.set(value.to_owned())});
                            }
                            value=form_value.name
                            set_value=form_value.set_name
                        />

                        <TextInput
                            name={format!("param_value_{}", form_value.id)}
                            input_type="text".to_owned()
                            class_name="w-full".to_owned()
                            placeholder=move || {t!(i18n, rest_client_param_value_placeholder).to_html()}
                            on_change=move |value: String| {
                                params.read_untracked().set_body_formencoded.write().iter_mut()
                                    .filter(|h|h.id == form_value.id)
                                    .for_each(|h| {h.set_value.set(value.to_owned())});
                            }
                            value=form_value.value
                            set_value=form_value.set_value
                        />

                        <Button
                            label=move || "-".to_owned()
                            class_name="text-bold".to_owned()
                            button_width=ButtonWidth::OneSymbol
                            color=ButtonColor::Danger
                            loading=move || false
                            disabled=move || false
                            on_click=move |_| {
                                params.read_untracked().set_body_formencoded.write().retain(|fv| fv.id != form_value.id);
                            }
                        />
                    </div>
                }
            }
        />

    }
}
