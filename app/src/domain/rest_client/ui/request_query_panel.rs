use std::str::FromStr;

use crate::components::layout::property_editor::{KeyValueTableItem, PropertyEditor};
use crate::domain::rest_client::ui::request_params::{RequestInfo, RequestParams};
use crate::i18n::*;
use leptos::prelude::*;
use url::Url;

#[derive(Clone)]
struct QueryItem {
    pub id: usize,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

#[component]
pub fn RequestQueryPanel(
    request_info: ReadSignal<RequestInfo>,
    params: ReadSignal<RequestParams>,
) -> impl IntoView {
    let i18n = use_i18n();
    let (items, set_items) = signal(Vec::<QueryItem>::new());
    let (items_updating, set_items_updating) = signal(false);

    Effect::watch(
        move || request_info.get(),
        move |value, _prev, _| {
            if !items_updating.get_untracked() {
                if let Ok(url) = Url::parse(&value.url) {
                    let mut query_items = Vec::new();
                    for (i, pair) in url.query_pairs().enumerate() {
                        let (name, set_name) = signal(pair.0.to_string());
                        let (value, set_value) = signal(pair.1.to_string());
                        query_items.push(QueryItem { id: i + 1, name, set_name, value, set_value });
                    }

                    set_items_updating.set(true);
                    set_timeout(
                        move || {
                            set_items.set(query_items);
                            set_items_updating.set(false);
                        },
                        std::time::Duration::from_millis(150),
                    );
                }
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if !items_updating.get_untracked() && (prev.is_none() || value != prev.unwrap()) {
                if let Ok(url) = Url::parse(value) {
                    let mut query_items = Vec::new();
                    for (i, pair) in url.query_pairs().enumerate() {
                        let (name, set_name) = signal(pair.0.to_string());
                        let (value, set_value) = signal(pair.1.to_string());
                        query_items.push(QueryItem { id: i + 1, name, set_name, value, set_value });
                    }

                    set_items_updating.set(true);
                    set_timeout(
                        move || {
                            set_items.set(query_items);
                            set_items_updating.set(false);
                        },
                        std::time::Duration::from_millis(150),
                    );
                }
            }
        },
        false,
    );

    Effect::watch(
        move || items.get(),
        move |value, _prev, _| {
            if !items_updating.get_untracked()
                && let Ok(mut url) = Url::from_str(&params.get_untracked().url.read_untracked())
            {
                url.query_pairs_mut().clear();
                for item in value {
                    url.query_pairs_mut()
                        .append_pair(&item.name.read_untracked(), &item.value.read_untracked());
                }
                let url_str = url.to_string();
                if params.read_untracked().url.read_untracked() != url_str {
                    set_items_updating.set(true);
                    set_timeout(
                        move || {
                            params.read_untracked().set_url.set(url_str);
                            set_items_updating.set(false);
                        },
                        std::time::Duration::from_millis(150),
                    );
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
                let id = items.read_untracked().iter().map(|h|h.id).max().unwrap_or_default() + 1;
                let (name, set_name) = signal(v.0);
                let (value, set_value) = signal(v.1);

                set_items.write().push(QueryItem{ id, name, set_name, value, set_value });
            }
            on_delete=move |id| {
                set_items.write().retain(|h| h.id != id);
            }
            on_change_key=move |v: (usize, String)| {
                set_items.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_name.set(v.1.to_owned())});
            }
            on_change_value=move |v: (usize, String)| {
                set_items.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_value.set(v.1.to_owned())});
            }
        />
    }
}

impl KeyValueTableItem for QueryItem {
    fn id(&self) -> usize {
        self.id
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
