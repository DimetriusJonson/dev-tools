use std::str::FromStr;

use crate::common::constants::HEADERS_AUTOCOMPLETE;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::layout::property_editor::PropertyEditor;
use crate::domain::rest_client::model::request_params::{CustomHeader, RequestInfo, RequestParams};
use crate::domain::rest_client::util::request_store::{RequestFieldKind, get_stored_value};
use crate::i18n::*;
use http::{HeaderName, HeaderValue};
use leptos::prelude::*;
use uuid::Uuid;

use crate::common::constants::MEDIA_TYPES_AUTOCOMPLETE;

#[component]
pub fn RequestHeadersPanel(
    params: ReadSignal<RequestParams>,
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    Effect::watch(
        move || request_info.get(),
        move |value, prev, _| {
            if prev.is_none()
                || value.id != prev.unwrap().id
                || project.read_untracked().parse::<i32>().unwrap_or(0) != prev.unwrap().project_id
            {
                params
                    .read_untracked()
                    .set_headers
                    .set(load_headers(&project.read_untracked(), value.id));
            }
        },
        false,
    );

    view! {
        <PropertyEditor
            key_label=move || t_display!(i18n, rest_client_header_name).to_string()
            value_label=move || t_display!(i18n, rest_client_header_value).to_string()
            items=move || params.read_untracked().headers.get()
            key_options=HEADERS_AUTOCOMPLETE
            value_options=MEDIA_TYPES_AUTOCOMPLETE
            on_add=move |v:(String, String)| {
                let name_converted = v.0.to_lowercase();
                if params.read_untracked().headers.read_untracked().iter().find(|h|h.name.read_untracked().to_lowercase() == name_converted).is_none() {
                    if let Err(err) = HeaderName::from_str(&v.0) {
                        show_error(err.to_string(), messages);
                        return;
                    }

                    if let Err(err) = HeaderValue::from_str(&v.0) {
                        show_error(err.to_string(), messages);
                        return;
                    }

                    let id = Uuid::new_v4().to_string();
                    let (name, set_name) = signal(v.0);
                    let (value, set_value) = signal(v.1);

                    params.read_untracked().set_headers.write().push(CustomHeader{ id, name, set_name, value, set_value });
                }
            }
            on_delete=move |id:String| {
                params.read_untracked().set_headers.write().retain(|h| h.id != id);
            }
            on_change_key=move |v: (String, String)| {
                if let Err(err) = HeaderName::from_str(&v.1) {
                    show_error(err.to_string(), messages);
                } else {
                    params.read_untracked().set_headers.write().iter_mut()
                        .filter(|h|h.id == v.0)
                        .for_each(|h| {h.set_name.set(v.1.to_owned())});
                }
            }
            on_change_value=move |v: (String, String)| {
                if let Err(err) = HeaderValue::from_str(&v.1) {
                    show_error(err.to_string(), messages);
                } else {
                    params.read_untracked().set_headers.write().iter_mut()
                        .filter(|h|h.id == v.0)
                        .for_each(|h| {h.set_value.set(v.1.to_owned())});
                }
            }
        />
    }
}

fn load_headers(project_id: &str, request_id: i32) -> Vec<CustomHeader> {
    let stored_value =
        get_stored_value(RequestFieldKind::Headers, "".to_owned(), project_id, request_id);
    if stored_value.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for line in stored_value.lines() {
        if let Some(index) = line.find(":") {
            let (name, set_name) = signal(line[..index].to_owned());
            let (value, set_value) = signal(line[index + 1..].to_owned());

            let header =
                CustomHeader { id: Uuid::new_v4().to_string(), name, set_name, value, set_value };
            result.push(header);
        }
    }

    result
}
