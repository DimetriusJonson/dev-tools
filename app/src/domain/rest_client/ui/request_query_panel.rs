use std::str::FromStr;

use crate::common::ui_utils::safe_updating_ui_value;
use crate::components::layout::property_editor::{KeyValueTableItem, PropertyEditor};
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::i18n::*;
use leptos::prelude::*;
use url::Url;
use uuid::Uuid;

#[derive(Clone)]
struct QueryItem {
    pub id: String,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

#[component]
pub fn RequestQueryPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().expect("Failed get rc_context");

    let (items, set_items) = signal(Vec::<QueryItem>::new());
    let update_lock = RwSignal::new(false);

    Effect::watch(
        move || rc_context.request.get(),
        move |value, _prev, _| {
            if let Ok(url) = Url::parse(&value.url) {
                safe_updating_ui_value(update_lock, move || {
                    let mut query_items = Vec::new();
                    for pair in url.query_pairs() {
                        let (name, set_name) = signal(pair.0.to_string());
                        let (value, set_value) = signal(pair.1.to_string());
                        query_items.push(QueryItem {
                            id: Uuid::new_v4().to_string(),
                            name,
                            set_name,
                            value,
                            set_value,
                        });
                    }
                    set_items.set(query_items)
                });
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if (prev.is_none() || value != prev.unwrap())
                && let Ok(url) = Url::parse(value)
            {
                safe_updating_ui_value(update_lock, move || {
                    let mut query_items = Vec::new();
                    for pair in url.query_pairs() {
                        let (name, set_name) = signal(pair.0.to_string());
                        let (value, set_value) = signal(pair.1.to_string());
                        query_items.push(QueryItem {
                            id: Uuid::new_v4().to_string(),
                            name,
                            set_name,
                            value,
                            set_value,
                        });
                    }
                    set_items.set(query_items)
                });
            }
        },
        false,
    );

    Effect::watch(
        move || items.get(),
        move |value, _prev, _| {
            if !update_lock.get_untracked()
                && let Ok(mut url) = Url::from_str(&params.get_untracked().url.read_untracked())
            {
                url.query_pairs_mut().clear();
                for item in value {
                    url.query_pairs_mut()
                        .append_pair(&item.name.read_untracked(), &item.value.read_untracked());
                }

                let url_str = url.to_string();
                if params.read_untracked().url.read_untracked() != url_str {
                    safe_updating_ui_value(update_lock, move || {
                        params.read_untracked().url.set(url_str.to_owned())
                    });
                }
            }
        },
        false,
    );

    view! {
        <PropertyEditor
            key_label=move || t_display!(i18n, rest_client_param_name_placeholder).to_string()
            value_label=move || t_display!(i18n, rest_client_param_value_placeholder).to_string()
            items=move || items.get()
            on_add=move |v:(String, String)| {
                let id = Uuid::new_v4().to_string();
                if !v.0.trim().is_empty() && !v.1.trim().is_empty() {
                    let (name, set_name) = signal(v.0);
                    let (value, set_value) = signal(v.1);

                    set_items.write().push(QueryItem{ id, name, set_name, value, set_value });
                }
            }
            on_delete=move |id| {
                set_items.write().retain(|h| h.id != id);
            }
            on_change_key=move |v: (String, String)| {
                set_items.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_name.set(v.1.to_owned())});
            }
            on_change_value=move |v: (String, String)| {
                set_items.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_value.set(v.1.to_owned())});
            }
        />
    }
}

impl KeyValueTableItem for QueryItem {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn name(&self) -> ReadSignal<String> {
        self.name
    }

    fn set_name(&self) -> WriteSignal<String> {
        self.set_name
    }

    fn value(&self) -> ReadSignal<String> {
        self.value
    }

    fn set_value(&self) -> WriteSignal<String> {
        self.set_value
    }
}
