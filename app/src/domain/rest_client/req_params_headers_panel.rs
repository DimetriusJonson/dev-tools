use crate::common::constants::HEADERS_AUTOCOMPLETE;
use crate::common::ui_utils::is_base_header_name;
use crate::components::ui::button::{Button, ButtonColor, ButtonWidth};
use crate::i18n::*;
use leptos::prelude::*;

use crate::{
    common::{
        constants::MEDIA_TYPES_AUTOCOMPLETE,
        local_store::{get_local_store_value, set_local_store_value},
    },
    components::ui::{autocomplete_input::AutocompleteInput, text_input::TextInput},
};

#[derive(Clone, Debug)]
pub struct CustomHeader {
    id: usize,
    pub name: ReadSignal<String>,
    set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    set_value: WriteSignal<String>,
}

#[component]
pub fn ReqParamsHeadersPanel(
    content_type: ReadSignal<String>,
    set_content_type: WriteSignal<String>,
    accept: ReadSignal<String>,
    set_accept: WriteSignal<String>,
    user_agent: ReadSignal<String>,
    set_user_agent: WriteSignal<String>,
    accept_lang: ReadSignal<String>,
    set_accept_lang: WriteSignal<String>,
    custom_headers: ReadSignal<Vec<CustomHeader>>,
    set_custom_headers: WriteSignal<Vec<CustomHeader>>,
) -> impl IntoView {
    let i18n = use_i18n();

    let (custom_header_name, set_custom_header_name) = signal("".to_owned());
    let (custom_header_value, set_custom_header_value) = signal("".to_owned());

    let _ = Effect::new(move || {
        set_custom_headers.set(restore_custom_headers());
    });

    view! {

        <div class="flex gap-4">
            <AutocompleteInput
                class_name="min-w-36".to_owned()
                placeholder=move || "Content-Type".to_owned()
                options={MEDIA_TYPES_AUTOCOMPLETE}
                value=content_type
                set_value=set_content_type
                on_change=move |value| {
                    set_local_store_value("rc_content_type", value);
                }
            />

            <AutocompleteInput
                class_name="min-w-36".to_owned()
                placeholder=move || "Accept".to_owned()
                options={MEDIA_TYPES_AUTOCOMPLETE}
                on_change=move |value| {
                    set_local_store_value("rc_accept", value);
                }
                value=accept
                set_value=set_accept
            />

            <TextInput
                name="accept-lang".to_owned()
                input_type="text".to_owned()
                class_name="w-max-36".to_owned()
                placeholder=|| "Accept-Language".to_owned()
                value=accept_lang
                set_value=set_accept_lang
                on_change=move |_| {
                    set_local_store_value("rc_accept_lang", accept_lang.get_untracked());
                }
            />
            <TextInput
                name="user-agent".to_owned()
                input_type="text".to_owned()
                class_name="w-full".to_owned()
                placeholder=|| "User-Agent".to_owned()
                value=user_agent
                set_value=set_user_agent
                on_change=move |_| {
                    set_local_store_value("rc_user_agent", user_agent.get_untracked());
                }
            />
        </div>

        <div class="flex gap-4">
            <div class="flex gap-4 w-full">
                <AutocompleteInput
                    class_name="min-w-36".to_owned()
                    placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                    options={HEADERS_AUTOCOMPLETE}
                    on_change=move |_| {}
                    value=custom_header_name
                    set_value=set_custom_header_name
                />
                <AutocompleteInput
                    class_name="w-full".to_owned()
                    placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                    options={MEDIA_TYPES_AUTOCOMPLETE}
                    on_change=move |_| {}
                    value=custom_header_value
                    set_value=set_custom_header_value
                />
                <Button
                    label=move || "+".to_owned()
                    class_name="text-bold".to_owned()
                    button_width=ButtonWidth::OneSymbol
                    color=ButtonColor::Success
                    loading=move || false
                    disabled=move || false
                    on_click=move |_| {
                        let name_converted = custom_header_name.get_untracked().trim().to_lowercase();
                        if !name_converted.is_empty() &&
                            !is_base_header_name(&name_converted) &&
                            !custom_header_value.read_untracked().is_empty() {
                            if custom_headers.read_untracked().iter().find(|h|h.name.read_untracked().trim().to_lowercase() == name_converted).is_none() {
                                let id = custom_headers.read_untracked().iter().map(|h|h.id).max().unwrap_or_default() + 1;
                                let (name, set_name) = signal(custom_header_name.get_untracked());
                                let (value, set_value) = signal(custom_header_value.get_untracked());

                                set_custom_headers.write().push(CustomHeader{ id, name, set_name, value, set_value });
                                set_custom_header_name.set("".to_owned());
                                set_custom_header_value.set("".to_owned());
                                save_custom_headers(&custom_headers.read_untracked());
                            }
                        }
                    }
                />
            </div>
        </div>

        <For
            each=move || custom_headers.get()
            key=|custom_header| custom_header.id
            children=move |custom_header| {
                view! {
                    <div class="flex gap-4 w-full">
                        <AutocompleteInput
                            class_name="min-w-36".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                            options={HEADERS_AUTOCOMPLETE}
                            on_change=move |_| {
                                save_custom_headers(&custom_headers.read_untracked());
                            }
                            value=custom_header.name
                            set_value=custom_header.set_name
                        />
                        <AutocompleteInput
                            class_name="w-full".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                            options={MEDIA_TYPES_AUTOCOMPLETE}
                            on_change=move |_| {
                                save_custom_headers(&custom_headers.read_untracked());
                            }
                            value=custom_header.value
                            set_value=custom_header.set_value
                        />
                        <Button
                            label=move || "-".to_owned()
                            class_name="text-bold".to_owned()
                            button_width=ButtonWidth::OneSymbol
                            color=ButtonColor::Danger
                            loading=move || false
                            disabled=move || false
                            on_click=move |_| {
                                set_custom_headers.write().retain(|h| h.id != custom_header.id);
                                save_custom_headers(&custom_headers.read_untracked());
                            }
                        />
                    </div>
                }
            }
        />

    }
}

fn restore_custom_headers() -> Vec<CustomHeader> {
    let stored_value = get_local_store_value("rc_custom_headers", "".to_owned());
    if stored_value.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for (i, line) in stored_value.lines().enumerate() {
        if let Some(index) = line.find(":") {
            let (name, set_name) = signal(line[..index].to_owned());
            let (value, set_value) = signal(line[index + 1..].to_owned());

            let header = CustomHeader { id: i + 1, name, set_name, value, set_value };
            result.push(header);
        }
    }

    result
}

fn save_custom_headers(headers: &Vec<CustomHeader>) {
    let value = headers
        .iter()
        .map(|h| format!("{}:{}", h.name.get_untracked(), h.value.get_untracked()))
        .collect::<Vec<String>>()
        .join("\n");

    set_local_store_value("rc_custom_headers", value);
}
