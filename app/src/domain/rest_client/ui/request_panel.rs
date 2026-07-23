use leptos::prelude::*;

use crate::{
    common::{
        local_store::{get_local_store_value, set_local_store_value},
        ui_utils::get_accept_language,
    }, domain::rest_client::ui::{
        request_params::{CustomHeader, RequestInfo, RequestParams}, request_params_panel::RequestParamsPanel, request_result_panel::RequestResultPanel,
    },
};

#[component]
pub fn RequestPanel(request_info: RequestInfo) -> impl IntoView {
    let (url, set_url) = signal(get_stored_value("rc_url", request_info.url, request_info.id));
    let (method, set_method) = signal(get_stored_value("rc_method", request_info.method, request_info.id));
    let (body, set_body) = signal(get_stored_value("rc_body", "".to_owned(), request_info.id));
    let (content_type, set_content_type) =
        signal(get_stored_value("rc_content_type", "".to_owned(), request_info.id));
    let (accept, set_accept) = signal(get_stored_value("rc_accept", "".to_owned(), request_info.id));
    let (user_agent, set_user_agent) =
        signal(get_stored_value("rc_user_agent", "WebDevUsefulTools Client".to_owned(), request_info.id));
    let (accept_lang, set_accept_lang) =
        signal(get_stored_value("rc_accept_lang", get_accept_language(), request_info.id));
    let (custom_headers, set_custom_headers) = signal(Vec::<CustomHeader>::new());
    let (params, _set_params) = signal(RequestParams {
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
        params.read_untracked().set_custom_headers.set(restore_custom_headers());
    });

    create_req_watchers(params, request_info.id);

    let (response, set_response) = signal(None);

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <RequestParamsPanel
                params on_result=move|res| {
                    set_response.set(Some(res));
                }
            />

            <RequestResultPanel data=response/>

        </div>
    }
}

fn get_stored_value(name: &str, default: String, request_id: i32) -> String {
    get_local_store_value(&format!("{}-{}", request_id, name), default)
}

fn create_req_watchers(params: ReadSignal<RequestParams>, request_id: i32) {
    create_watcher(params.read_untracked().url, "rc_url", request_id);
    create_watcher(params.read_untracked().method, "rc_method", request_id);
    create_watcher(params.read_untracked().body, "rc_body", request_id);
    create_watcher(params.read_untracked().content_type, "rc_content_type", request_id);
    create_watcher(params.read_untracked().accept, "rc_accept", request_id);
    create_watcher(params.read_untracked().accept_lang, "rc_accept_lang", request_id);
    create_watcher(params.read_untracked().user_agent, "rc_user_agent", request_id);

    Effect::watch(
        move || params.read_untracked().custom_headers.get(),
        move |value, _prev, _| {
            set_local_store_value(&format!("{}-rc_custom_headers", request_id), custom_headers_to_string(value))
        },
        false,
    );
}

fn create_watcher(value: ReadSignal<String>, name: &str, request_id: i32) {
    let name = name.to_owned();
    Effect::watch(
        move || value.get(),
        move |value, _prev, _| set_local_store_value(&format!("{}-{}", request_id, name), value.to_string()),
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
