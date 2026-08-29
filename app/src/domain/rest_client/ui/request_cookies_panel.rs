use crate::common::ui_utils::safe_updating_ui_value;
use crate::components::layout::property_editor::{KeyValueTableItem, PropertyEditor};
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::i18n::*;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone)]
struct CookiesItem {
    pub id: String,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

#[component]
pub fn RequestCookiesPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();

    let (items, set_items) = signal(Vec::<CookiesItem>::new());
    let update_lock = RwSignal::new(false);

    Effect::watch(
        move || params.read_untracked().headers.get(),
        move |value, prev, _| {
            if let Some(prev) = prev {
                let prev_cookie =
                    prev.iter().find(|h| h.name.read_untracked().to_lowercase() == "cookie");
                let actual_cookie =
                    value.iter().find(|h| h.name.read_untracked().to_lowercase() == "cookie");

                if prev_cookie.is_none()
                    || (prev_cookie.is_some()
                        && actual_cookie.is_some()
                        && prev_cookie.unwrap().value.get_untracked()
                            != actual_cookie.unwrap().value.get_untracked())
                {
                    let actual_cookies =
                        parse_cookies(&actual_cookie.unwrap().value.read_untracked());

                    safe_updating_ui_value(update_lock, move || {
                        let mut cookies_items = Vec::new();
                        for (cookie_name, cookie_value) in &actual_cookies {
                            let (name, set_name) = signal(cookie_name.to_owned());
                            let (value, set_value) = signal(cookie_value.to_owned());
                            cookies_items.push(CookiesItem {
                                id: Uuid::new_v4().to_string(),
                                name,
                                set_name,
                                value,
                                set_value,
                            });
                        }
                        set_items.set(cookies_items)
                    });
                }
            }
        },
        false,
    );

    Effect::watch(
        move || items.get(),
        move |value, _prev, _| {
            if !update_lock.get_untracked()
                && let Some(actual_cookie_header) = params
                    .get_untracked()
                    .headers
                    .get_untracked()
                    .iter()
                    .find(|h| h.name.read_untracked().to_lowercase() == "cookie")
                    .map(|h| h.clone())
            {
                let cookie_value = value
                    .iter()
                    .map(|item| {
                        format!("{}={}", &item.name.read_untracked(), &item.value.read_untracked())
                    })
                    .collect::<Vec<String>>()
                    .join("; ");

                if actual_cookie_header.value.get_untracked() != cookie_value {
                    safe_updating_ui_value(update_lock, move || {
                        actual_cookie_header.value.set(cookie_value.to_owned())
                    });
                }
            }
        },
        false,
    );

    view! {
        <PropertyEditor
            key_label=move || t_display!(i18n, rest_client_cookie_name_placeholder).to_string()
            value_label=move || t_display!(i18n, rest_client_cookie_value_placeholder).to_string()
            items=move || items.get()
            on_add=move |v:(String, String)| {
                let id = Uuid::new_v4().to_string();
                if !v.0.trim().is_empty() && !v.1.trim().is_empty() {
                    let (name, set_name) = signal(v.0);
                    let (value, set_value) = signal(v.1);

                    set_items.write().push(CookiesItem{ id, name, set_name, value, set_value });
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

impl KeyValueTableItem for CookiesItem {
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

fn parse_cookies(cookie_header: &str) -> Vec<(String, String)> {
    cookie_header
        .split(';')
        .filter_map(|s| {
            let mut parts = s.trim().splitn(2, '=');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect()
}
