use leptos::prelude::*;

use crate::{
    common::{local_store::{get_local_store_value, set_local_store_value}, ui_utils::get_accept_language}, domain::rest_client::ui::{
        request_params::{CustomHeader, RequestInfo, RequestParams},
        request_params_panel::RequestParamsPanel,
        request_result_panel::RequestResultPanel,
    },
};

#[component]
pub fn RequestPanel(
    request_info: ReadSignal<RequestInfo>,
    set_request_info: WriteSignal<RequestInfo>,
) -> impl IntoView {
    let (url, set_url) = signal("".to_owned());
    let (method, set_method) = signal("".to_owned());
    let (body, set_body) = signal("".to_owned());
    let (content_type, set_content_type) = signal("".to_owned());
    let (accept, set_accept) = signal("".to_owned());
    let (user_agent, set_user_agent) = signal("".to_owned());
    let (accept_lang, set_accept_lang) = signal("".to_owned());
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
        params
            .read_untracked()
            .set_custom_headers
            .set(load_custom_headers(request_info.read_untracked().id));
    });

    create_request_info_watcher(params, request_info);
    create_req_watchers(params, request_info, set_request_info);

    let (response, set_response) = signal(None);

    view! {
        <Show when=move || { request_info.read().id > 0 }
            fallback=|| view! { <div class="flex-1 flex h-[94dvh] items-center justify-center">{"Select project please."}</div> }
        >
            <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
                <RequestParamsPanel
                    params on_result=move|res| {
                        set_response.set(Some(res));
                    }
                />

                <RequestResultPanel data=response/>

            </div>
        </Show>
    }
}

fn get_stored_value(name: &str, default: String, request_id: i32) -> String {
    get_local_store_value(&format!("{}-{}", request_id, name), default)
}

fn create_req_watchers(
    params: ReadSignal<RequestParams>,
    request_info: ReadSignal<RequestInfo>,
    set_request_info: WriteSignal<RequestInfo>,
) {
    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_local_store_value(
                    &format!("{}-{}", request_info.read_untracked().id, "rc_url"),
                    value.to_string(),
                );
                set_request_info.write().url = value.to_owned();
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().method.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_local_store_value(
                    &format!("{}-{}", request_info.read_untracked().id, "rc_method"),
                    value.to_string(),
                );
                set_request_info.write().method = value.to_owned();
            }
        },
        false,
    );

    create_watcher(params.read_untracked().body, "rc_body", request_info);
    create_watcher(params.read_untracked().content_type, "rc_content_type", request_info);
    create_watcher(params.read_untracked().accept, "rc_accept", request_info);
    create_watcher(params.read_untracked().accept_lang, "rc_accept_lang", request_info);
    create_watcher(params.read_untracked().user_agent, "rc_user_agent", request_info);

    Effect::watch(
        move || params.read_untracked().custom_headers.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_local_store_value(
                    &format!("{}-rc_custom_headers", request_info.read_untracked().id),
                    custom_headers_to_string(value),
                )
            }
        },
        false,
    );
}

fn create_request_info_watcher(
    params: ReadSignal<RequestParams>,
    request_info: ReadSignal<RequestInfo>,
) {
    Effect::watch(
        move || request_info.get(),
        move |value, prev, _| {
            let id = value.id;
            if prev.is_none() || id != prev.unwrap().id {
                params.read_untracked().set_url.set(value.url.to_owned());
                params.read_untracked().set_method.set(value.method.to_owned());
                params.read_untracked().set_body.set(get_stored_value(
                    "rc_body",
                    "".to_owned(),
                    id,
                ));
                params.read_untracked().set_content_type.set(get_stored_value(
                    "rc_content_type",
                    "".to_owned(),
                    id,
                ));
                params.read_untracked().set_accept.set(get_stored_value(
                    "rc_accept",
                    "".to_owned(),
                    id,
                ));
                params.read_untracked().set_accept_lang.set(get_stored_value(
                    "rc_accept_lang",
                    get_accept_language(),
                    id,
                ));
                params.read_untracked().set_user_agent.set(get_stored_value(
                    "rc_user_agent",
                    "WebDevUsefulTools Client".to_owned(),
                    id,
                ));
                params.read_untracked().set_custom_headers.set(load_custom_headers(id));
            }
        },
        false,
    );
}

fn create_watcher(value: ReadSignal<String>, name: &str, request_info: ReadSignal<RequestInfo>) {
    let name = name.to_owned();
    Effect::watch(
        move || value.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_local_store_value(
                    &format!("{}-{}", request_info.read_untracked().id, name),
                    value.to_string(),
                )
            }
        },
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

fn load_custom_headers(request_id: i32) -> Vec<CustomHeader> {
    let stored_value =
        get_local_store_value(&format!("{}-rc_custom_headers", request_id), "".to_owned());
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
