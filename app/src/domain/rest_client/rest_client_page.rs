use leptos::prelude::*;

use crate::{
    common::{
        local_store::{get_local_store_value, set_local_store_value},
        ui_utils::get_accept_language,
    },
    domain::rest_client::ui::{
        req_panel::ReqPanel,
        req_params::{CustomHeader, RequestParams},
    },
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let (url, set_url) = signal(get_local_store_value("rc_url", "".to_owned()));
    let (method, set_method) = signal(get_local_store_value("rc_method", "GET".to_owned()));
    let (body, set_body) = signal(get_local_store_value("rc_body", "".to_owned()));
    let (content_type, set_content_type) =
        signal(get_local_store_value("rc_content_type", "".to_owned()));
    let (accept, set_accept) = signal(get_local_store_value("rc_accept", "".to_owned()));
    let (user_agent, set_user_agent) =
        signal(get_local_store_value("rc_user_agent", "WebDevUsefulTools Client".to_owned()));
    let (accept_lang, set_accept_lang) =
        signal(get_local_store_value("rc_accept_lang", get_accept_language()));
    let (custom_headers, set_custom_headers) = signal(Vec::<CustomHeader>::new());
    let (req_params, _set_req_params) = signal(RequestParams {
        url,
        set_url,
        method,
        set_method,
        body,
        set_body,
        content_type,
        set_content_type,
        accept,
        set_accept,
        user_agent,
        set_user_agent,
        accept_lang,
        set_accept_lang,
        custom_headers,
        set_custom_headers,
    });

    let _ = Effect::new(move || {
        req_params.read_untracked().set_custom_headers.set(restore_custom_headers());
    });

    create_req_watchers(req_params);

    view! {
        <ReqPanel params=req_params />
    }
}

fn create_req_watchers(params: ReadSignal<RequestParams>) {
    create_watcher(params.read_untracked().url, "rc_url");
    create_watcher(params.read_untracked().method, "rc_method");
    create_watcher(params.read_untracked().body, "rc_body");
    create_watcher(params.read_untracked().content_type, "rc_content_type");
    create_watcher(params.read_untracked().accept, "rc_accept");
    create_watcher(params.read_untracked().accept_lang, "rc_accept_lang");
    create_watcher(params.read_untracked().user_agent, "rc_user_agent");

    Effect::watch(
        move || params.read_untracked().custom_headers.get(),
        move |value, _prev, _| {
            set_local_store_value("rc_custom_headers", custom_headers_to_string(value))
        },
        false,
    );
}

fn create_watcher(value: ReadSignal<String>, save_path: &str) {
    let save_path = save_path.to_owned();
    Effect::watch(
        move || value.get(),
        move |value, _prev, _| set_local_store_value(&save_path, value.to_string()),
        false,
    );
}

fn custom_headers_to_string(headers: &Vec<CustomHeader>) -> String {
    headers
        .iter()
        .map(|h| format!("{}:{}", h.name.get_untracked(), h.value.get_untracked()))
        .collect::<Vec<String>>()
        .join("\n")
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
