use std::str::FromStr;

use crate::common::constants::HEADERS_AUTOCOMPLETE;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::layout::property_editor::PropertyEditor;
use crate::domain::rest_client::model::request_params::{CustomHeader, RequestParams};
use crate::i18n::*;
use http::{HeaderName, HeaderValue};
use leptos::prelude::*;
use uuid::Uuid;

use crate::common::constants::MEDIA_TYPES_AUTOCOMPLETE;

#[component]
pub fn RequestHeadersPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    view! {
        <PropertyEditor
            key_label=move || t_display!(i18n, rest_client_header_name).to_string()
            value_label=move || t_display!(i18n, rest_client_header_value).to_string()
            items=move || params.read_untracked().headers.read().vec_owned()
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

                    params.read_untracked().headers.write().push(CustomHeader{ id, name, set_name, value, set_value });
                }
            }
            on_delete=move |id:String| {
                params.read_untracked().headers.write().remove_by_id(id);
            }
            on_change_key=move |v: (String, String)| {
                if let Err(err) = HeaderName::from_str(&v.1) {
                    show_error(err.to_string(), messages);
                } else {
                    params.read_untracked().headers.write().iter_mut()
                        .filter(|h|h.id == v.0)
                        .for_each(|h| {h.set_name.set(v.1.to_owned())});
                }
            }
            on_change_value=move |v: (String, String)| {
                if let Err(err) = HeaderValue::from_str(&v.1) {
                    show_error(err.to_string(), messages);
                } else {
                    params.read_untracked().headers.write().iter_mut()
                        .filter(|h|h.id == v.0)
                        .for_each(|h| {h.set_value.set(v.1.to_owned())});
                }
            }
        />
    }
}
