use std::str::FromStr;

use crate::common::constants::HEADERS_AUTOCOMPLETE;
use crate::common::ui_utils::is_base_header_name;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button::{Button, ButtonColor, ButtonWidth};
use crate::domain::rest_client::ui::request_params::{CustomHeader, RequestParams};
use crate::i18n::*;
use http::{HeaderName, HeaderValue};
use leptos::prelude::*;

use crate::{
    common::constants::MEDIA_TYPES_AUTOCOMPLETE,
    components::ui::autocomplete_input::AutocompleteInput,
};

#[component]
pub fn RequestHeadersPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (header_name, set_header_name) = signal("".to_owned());
    let (header_value, set_header_value) = signal("".to_owned());

    view! {
        <div class="flex flex-col gap-1 md:gap-4">
            <div class="flex flex-row gap-1 md:gap-4">
                <AutocompleteInput
                    class_name="sm:min-w-36".to_owned()
                    placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                    options={HEADERS_AUTOCOMPLETE}
                    on_change=move |_| {}
                    value=header_name
                    set_value=set_header_name
                />
                <AutocompleteInput
                    class_name="w-full".to_owned()
                    placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                    options={MEDIA_TYPES_AUTOCOMPLETE}
                    on_change=move |_| {}
                    value=header_value
                    set_value=set_header_value
                />
                <Button
                    label=move || "+".to_owned()
                    class_name="text-bold".to_owned()
                    button_width=ButtonWidth::OneSymbol
                    color=ButtonColor::Success
                    loading=move || false
                    disabled=move || false
                    on_click=move |_| {
                        let name_converted = header_name.get_untracked().to_lowercase();
                        if !is_base_header_name(&name_converted) 
                            && params.read_untracked().headers.read_untracked().iter().find(|h|h.name.read_untracked().to_lowercase() == name_converted).is_none() {
                                if let Err(err) = HeaderName::from_str(&header_name.get_untracked()) {
                                    show_error(err.to_string(), messages);
                                    return;
                                }

                                if let Err(err) = HeaderValue::from_str(&header_value.get_untracked()) {
                                    show_error(err.to_string(), messages);
                                    return;
                                }

                                let id = params.read_untracked().headers.read_untracked().iter().map(|h|h.id).max().unwrap_or_default() + 1;
                                let (name, set_name) = signal(header_name.get_untracked());
                                let (value, set_value) = signal(header_value.get_untracked());

                                params.read_untracked().set_headers.write().push(CustomHeader{ id, name, set_name, value, set_value });
                                set_header_name.set("".to_owned());
                                set_header_value.set("".to_owned());
                            }
                    }
                />
            </div>
        </div>

        <For
            each=move || params.read_untracked().headers.get()
            key=|header| header.id
            children=move |header| {
                view! {
                    <div class="flex gap-1 md:gap-4">
                        <AutocompleteInput
                            class_name="sm:min-w-36".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                            options={HEADERS_AUTOCOMPLETE}
                            on_change=move |value: String| {
                                if let Err(err) = HeaderName::from_str(&value) {
                                    show_error(err.to_string(), messages);
                                } else {
                                    params.read_untracked().set_headers.write().iter_mut()
                                        .filter(|h|h.id == header.id)
                                        .for_each(|h| {h.set_name.set(value.to_owned())});
                                }
                            }
                            value=header.name
                            set_value=header.set_name
                        />
                        <AutocompleteInput
                            class_name="w-full".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                            options={MEDIA_TYPES_AUTOCOMPLETE}
                            on_change=move |value: String| {
                                if let Err(err) = HeaderValue::from_str(&value) {
                                    show_error(err.to_string(), messages);
                                } else {
                                    params.read_untracked().set_headers.write().iter_mut()
                                        .filter(|h|h.id == header.id)
                                        .for_each(|h| {h.set_value.set(value.to_owned())});
                                }
                            }
                            value=header.value
                            set_value=header.set_value
                        />
                        <Button
                            label=move || "-".to_owned()
                            class_name="text-bold".to_owned()
                            button_width=ButtonWidth::OneSymbol
                            color=ButtonColor::Danger
                            loading=move || false
                            disabled=move || false
                            on_click=move |_| {
                                params.read_untracked().set_headers.write().retain(|h| h.id != header.id);
                            }
                        />
                    </div>
                }
            }
        /> 

    }
}
