use leptos::prelude::*;

use crate::{
    common::{
        local_store::{get_local_store_value, set_local_store_value},
        ui_utils::get_accept_language,
    },
    domain::rest_client::ui::{
        req_panel::ReqPanel,
        req_params::{CustomHeader, RequestParamKind, RequestParams},
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

    view! {
        <ReqPanel params=req_params
            on_change=move |kind| save_changed_request(kind, req_params)
        />
    }
}

fn save_changed_request(kind: RequestParamKind, params: ReadSignal<RequestParams>) {
    match kind {
        RequestParamKind::Url => {
            set_local_store_value("rc_url", params.read_untracked().url.get_untracked())
        }
        RequestParamKind::Method => {
            set_local_store_value("rc_method", params.read_untracked().method.get_untracked())
        }
        RequestParamKind::Body => {
            set_local_store_value("rc_body", params.read_untracked().body.get_untracked())
        }
        RequestParamKind::ContentType => set_local_store_value(
            "rc_content_type",
            params.read_untracked().content_type.get_untracked(),
        ),
        RequestParamKind::Accept => {
            set_local_store_value("rc_accept", params.read_untracked().accept.get_untracked())
        }
        RequestParamKind::AcceptLanguage => set_local_store_value(
            "rc_accept_lang",
            params.read_untracked().accept_lang.get_untracked(),
        ),
        RequestParamKind::UserAgent => set_local_store_value(
            "rc_user_agent",
            params.read_untracked().user_agent.get_untracked(),
        ),
        RequestParamKind::CustomHeaders => set_local_store_value(
            "rc_custom_headers",
            custom_headers_to_string(&params.read_untracked().custom_headers.get_untracked()),
        ),
    }
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
