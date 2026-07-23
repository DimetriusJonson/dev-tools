use crate::common::constants::HEADERS_AUTOCOMPLETE;
use crate::common::ui_utils::is_base_header_name;
use crate::components::ui::button::{Button, ButtonColor, ButtonWidth};
use crate::domain::rest_client::ui::req_params::{CustomHeader, RequestParams};
use crate::i18n::*;
use leptos::prelude::*;

use crate::{
    common::constants::MEDIA_TYPES_AUTOCOMPLETE,
    components::ui::{autocomplete_input::AutocompleteInput, text_input::TextInput},
};

#[component]
pub fn ReqParamsHeadersPanel(
    params: ReadSignal<RequestParams>,
) -> impl IntoView {
    let i18n = use_i18n();

    let (custom_header_name, set_custom_header_name) = signal("".to_owned());
    let (custom_header_value, set_custom_header_value) = signal("".to_owned());

    view! {

        <div class="flex gap-4">
            <AutocompleteInput
                class_name="min-w-36".to_owned()
                placeholder=move || "Content-Type".to_owned()
                options={MEDIA_TYPES_AUTOCOMPLETE}
                value=params.read_untracked().content_type
                set_value=params.read_untracked().set_content_type
                on_change=move |_| {}
            />

            <AutocompleteInput
                class_name="min-w-36".to_owned()
                placeholder=move || "Accept".to_owned()
                options={MEDIA_TYPES_AUTOCOMPLETE}
                on_change=move |_| {}
                value=params.read_untracked().accept
                set_value=params.read_untracked().set_accept
            />

            <TextInput
                name="accept-lang".to_owned()
                input_type="text".to_owned()
                class_name="w-max-36".to_owned()
                placeholder=|| "Accept-Language".to_owned()
                value=params.read_untracked().accept_lang
                set_value=params.read_untracked().set_accept_lang
                on_change=move |_| {}
            />
            <TextInput
                name="user-agent".to_owned()
                input_type="text".to_owned()
                class_name="w-full".to_owned()
                placeholder=|| "User-Agent".to_owned()
                value=params.read_untracked().user_agent
                set_value=params.read_untracked().set_user_agent
                on_change=move |_| {}
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
                            if params.read_untracked().custom_headers.read_untracked().iter().find(|h|h.name.read_untracked().trim().to_lowercase() == name_converted).is_none() {
                                let id = params.read_untracked().custom_headers.read_untracked().iter().map(|h|h.id).max().unwrap_or_default() + 1;
                                let (name, set_name) = signal(custom_header_name.get_untracked());
                                let (value, set_value) = signal(custom_header_value.get_untracked());

                                params.read_untracked().set_custom_headers.write().push(CustomHeader{ id, name, set_name, value, set_value });
                                set_custom_header_name.set("".to_owned());
                                set_custom_header_value.set("".to_owned());
                            }
                        }
                    }
                />
            </div>
        </div>

        <For
            each=move || params.read_untracked().custom_headers.get()
            key=|custom_header| custom_header.id
            children=move |custom_header| {
                view! {
                    <div class="flex gap-4 w-full">
                        <AutocompleteInput
                            class_name="min-w-36".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                            options={HEADERS_AUTOCOMPLETE}
                            on_change=move |_| {
                                params.read_untracked().set_custom_headers.write().iter_mut()
                                    .filter(|h|h.id != custom_header.id)
                                    .for_each(|h| {h.set_name.set(custom_header.name.get_untracked())});
                            }
                            value=custom_header.name
                            set_value=custom_header.set_name
                        />
                        <AutocompleteInput
                            class_name="w-full".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                            options={MEDIA_TYPES_AUTOCOMPLETE}
                            on_change=move |_| {
                                params.read_untracked().set_custom_headers.write().iter_mut()
                                    .filter(|h|h.id != custom_header.id)
                                    .for_each(|h| {h.set_value.set(custom_header.value.get_untracked())});
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
                                params.read_untracked().set_custom_headers.write().retain(|h| h.id != custom_header.id);
                            }
                        />
                    </div>
                }
            }
        />

    }
}
