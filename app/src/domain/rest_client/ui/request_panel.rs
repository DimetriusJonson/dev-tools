use crate::{
    common::local_store::delete_local_store_value,
    components::layout::{
        drag_splitter::DragSplitter,
        message_banner::{Messages, show_error},
    },
    domain::rest_client::ui::{
        request_params::RequestBodyFormValue, request_result_panel::ReqResultData,
    },
    i18n::*,
};
use leptos::{
    html::{Button, Div},
    leptos_dom::logging::console_log,
    prelude::*,
};

use crate::{
    common::local_store::{get_local_store_value, set_local_store_value},
    domain::rest_client::ui::{
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
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (url, set_url) = signal("".to_owned());
    let (insecure, set_insecure) = signal(false);
    let (method, set_method) = signal("".to_owned());
    let (body, set_body) = signal("".to_owned());
    let (body_type, set_body_type) = signal("text".to_owned());
    let (body_formencoded, set_body_formencoded) = signal(Vec::new());
    let (headers, set_headers) = signal(Vec::<CustomHeader>::new());
    let (params, _set_params) = signal(RequestParams {
        url,
        set_url,
        insecure,
        set_insecure,
        method,
        set_method,
        body,
        set_body,
        body_type,
        set_body_type,
        body_formencoded,
        set_body_formencoded,
        headers,
        set_headers,
    });
    let (save_response, set_save_response) = signal(false);

    let (body_tab_selected, set_body_tab_selected) = signal(0);

    let params_ref = NodeRef::<Div>::new();
    let send_btn_node_ref = NodeRef::<Button>::new();

    let (response, set_response) = signal(None);

    let _ = Effect::new(move || {
        params.read_untracked().set_headers.set(load_headers(request_info.read_untracked().id));
        params
            .read_untracked()
            .set_body_formencoded
            .set(load_body_formencoded(request_info.read_untracked().id));
    });

    create_request_info_watcher(
        params,
        request_info,
        send_btn_node_ref,
        set_response,
        set_save_response,
        set_body_tab_selected,
        messages,
    );
    create_req_watchers(params, request_info, set_request_info);
    create_watcher_bool(save_response, "rc_save_response", request_info);

    view! {
        <Show when=move || { request_info.read().id > 0 }
            fallback=move || view! { <div class="flex-1 flex items-center justify-center">{t!(i18n, rest_client_request_not_selected_msg)}</div> }
        >
            <div class="flex-2 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
                <RequestParamsPanel
                    request_info
                    node_ref=params_ref
                    send_btn_node_ref
                    body_tab_selected
                    set_body_tab_selected
                    params
                    on_result=move|res: ReqResultData| {
                        if *save_response.read_untracked() {
                            let json_string = serde_json::to_string(&res).unwrap();
                            set_local_store_value(
                                &format!("{}-rc_save_response_data", request_info.read_untracked().id),
                                json_string,
                            )
                        } else {
                            delete_local_store_value(&format!("{}-rc_save_response_data", request_info.read_untracked().id))
                        }
                        set_response.set(Some(res));
                    }
                />

                <DragSplitter
                    class_name="hidden md:block".to_owned()
                    target_ref=params_ref
                    local_store_prop_name=move || "rc_params_width".to_owned()
                    min_scr_ration={1.0 / 6.0}
                    max_scr_ration={1.0 / 2.0}
                    default_scr_ration={1.0 / 6.0} />

                <RequestResultPanel save_response set_save_response data=response/>

            </div>
        </Show>
    }
}

fn get_stored_value(name: &str, default: String, request_id: i32) -> String {
    get_local_store_value(&format!("{}-{}", request_id, name), default)
}

fn get_stored_value_as_bool(name: &str, default: bool, request_id: i32) -> bool {
    let str = get_local_store_value(&format!("{}-{}", request_id, name), default.to_string());
    str::parse(&str).unwrap_or_default()
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

    create_watcher_bool(params.read_untracked().insecure, "rc_insecure", request_info);
    create_watcher(params.read_untracked().body, "rc_body", request_info);
    create_watcher(params.read_untracked().body_type, "rc_body_type", request_info);

    Effect::watch(
        move || params.read_untracked().headers.get(),
        move |value, _prev, _| {
            set_local_store_value(
                &format!("{}-rc_headers", request_info.read_untracked().id),
                headers_to_string(value),
            )
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().body_formencoded.get(),
        move |value, _prev, _| {
            set_local_store_value(
                &format!("{}-rc_body_formencoded", request_info.read_untracked().id),
                body_form_to_string(value),
            )
        },
        false,
    );
}

fn create_request_info_watcher(
    params: ReadSignal<RequestParams>,
    request_info: ReadSignal<RequestInfo>,
    send_btn_ref: NodeRef<Button>,
    set_response: WriteSignal<Option<ReqResultData>>,
    set_save_response: WriteSignal<bool>,
    set_body_tab_selected: WriteSignal<usize>,
    messages: Messages,
) {
    Effect::watch(
        move || request_info.get(),
        move |value, prev, _| {
            let id = value.id;

            if value.autorun
                && (prev.is_none() || !prev.unwrap().autorun)
                && let Some(send_btn) = send_btn_ref.get_untracked()
            {
                send_btn.click();
            }

            if prev.is_none() || id != prev.unwrap().id {
                set_response.set(None);
                params.read_untracked().set_url.set(value.url.to_owned());
                params.read_untracked().set_method.set(value.method.to_owned());
                params.read_untracked().set_insecure.set(get_stored_value_as_bool(
                    "rc_insecure",
                    false,
                    id,
                ));
                params.read_untracked().set_body.set(get_stored_value(
                    "rc_body",
                    "".to_owned(),
                    id,
                ));
                params.read_untracked().set_body_type.set(get_stored_value(
                    "rc_body_type",
                    "".to_owned(),
                    id,
                ));
                set_body_tab_selected.set(
                    match params.read_untracked().body_type.read_untracked().as_str() {
                        "formencoded" => 1,
                        _ => 0,
                    },
                );
                params.read_untracked().set_headers.set(load_headers(id));
                params.read_untracked().set_body_formencoded.set(load_body_formencoded(id));

                let save_response = get_local_store_value(
                    &format!("{}-rc_save_response", request_info.read_untracked().id),
                    "false".to_owned(),
                )
                .parse::<bool>()
                .unwrap();
                set_save_response.set(save_response);
                if save_response {
                    let data_str = get_local_store_value(
                        &format!("{}-rc_save_response_data", id),
                        "".to_owned(),
                    );
                    if !data_str.is_empty() {
                        match serde_json::from_str::<ReqResultData>(&data_str) {
                            Ok(data) => set_response.set(Some(data)),
                            Err(err) => show_error(err.to_string(), messages),
                        }
                    }
                }
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

fn create_watcher_bool(value: ReadSignal<bool>, name: &str, request_info: ReadSignal<RequestInfo>) {
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

fn headers_to_string(headers: &[CustomHeader]) -> String {
    headers
        .iter()
        .map(|h| format!("{}:{}", h.name.get_untracked(), h.value.get_untracked()))
        .collect::<Vec<String>>()
        .join("\n")
}

fn load_headers(request_id: i32) -> Vec<CustomHeader> {
    let stored_value = get_local_store_value(&format!("{}-rc_headers", request_id), "".to_owned());
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

fn body_form_to_string(form_values: &[RequestBodyFormValue]) -> String {
    let list: KeyValueVector =
        form_values.iter().map(|h| (h.name.get_untracked(), h.value.get_untracked())).collect();

    serde_json::to_string(&list).unwrap()
}

fn load_body_formencoded(request_id: i32) -> Vec<RequestBodyFormValue> {
    let stored_value =
        get_local_store_value(&format!("{}-rc_body_formencoded", request_id), "".to_owned());
    if stored_value.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    match serde_json::from_str::<KeyValueVector>(&stored_value) {
        Ok(values) => {
            for (i, value) in values.iter().enumerate() {
                let (name, set_name) = signal(value.0.to_owned());
                let (value, set_value) = signal(value.1.to_owned());

                let header = RequestBodyFormValue { id: i + 1, name, set_name, value, set_value };
                result.push(header);
            }
        }
        Err(err) => console_log(&format!("Error: {}", err)),
    }
    result
}

type KeyValueVector = Vec<(String, String)>;
