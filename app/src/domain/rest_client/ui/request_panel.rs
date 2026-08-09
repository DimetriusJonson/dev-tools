use std::str::FromStr;

use crate::{
    components::layout::{
        drag_splitter::DragSplitter,
        message_banner::{Messages, show_error},
    },
    domain::rest_client::{
        model::request_params::{
            CustomHeader, RequestBodyFormValue, RequestBodyKind, RequestInfo, RequestParams,
        },
        ui::request_params_url::RequestParamsUrl,
        util::request_store::{
            RequestFieldKind, delete_stored_value, get_stored_value, set_stored_value,
        },
    },
    i18n::*,
    model::restclient::rest_client_response::RestClientResponse,
};
use leptos::{
    html::{Button, Div},
    leptos_dom::logging::console_log,
    prelude::*,
};

use crate::domain::rest_client::ui::{
    request_params_panel::RequestParamsPanel, request_result_panel::RequestResultPanel,
};

#[component]
pub fn RequestPanel(
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    set_request_info: WriteSignal<RequestInfo>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (url, set_url) = signal("".to_owned());
    let (method, set_method) = signal("".to_owned());
    let (body, set_body) = signal("".to_owned());
    let (body_type, set_body_type) = signal(RequestBodyKind::Text);
    let (body_formencoded, set_body_formencoded) = signal(Vec::new());
    let (headers, set_headers) = signal(Vec::<CustomHeader>::new());
    let (params, _set_params) = signal(RequestParams {
        url,
        set_url,
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

    create_request_info_watcher(
        params,
        project,
        request_info,
        send_btn_node_ref,
        set_response,
        set_save_response,
        set_body_tab_selected,
        messages,
    );
    create_req_watchers(params, project, request_info, set_request_info);
    create_watcher_bool(save_response, RequestFieldKind::SaveResponse, project, request_info);

    view! {
        <Show when=move || { request_info.read().id > 0 }
            fallback=move || view! { <div class="flex-1 flex items-center justify-center">{t!(i18n, rest_client_request_not_selected_msg)}</div> }
        >
            <div class="flex-2 flex flex-col gap-2 px-2 py-4 text-xs md:text-base">
                <RequestParamsUrl
                    send_btn_node_ref
                    params
                    on_result=move|res: RestClientResponse| {
                        if *save_response.read_untracked() {
                            let json_string = serde_json::to_string(&res).unwrap();
                            set_stored_value(project, request_info.read_untracked().id, RequestFieldKind::SaveResponseData, json_string)
                        } else {
                            delete_stored_value(project, request_info, RequestFieldKind::SaveResponseData)
                        }
                        set_response.set(Some(res));
                    }
                />

                <div class="flex-1 flex flex-col md:flex-row gap-2 text-xs md:text-base">
                    <RequestParamsPanel
                        project
                        request_info
                        node_ref=params_ref
                        body_tab_selected
                        set_body_tab_selected
                        params
                    />

                    <DragSplitter
                        class_name="hidden md:block".to_owned()
                        target_ref=params_ref
                        local_store_prop_name=move || "params_width".to_owned()
                        min_scr_ration={1.0 / 6.0}
                        max_scr_ration={1.0 / 2.0}
                        default_scr_ration={1.0 / 6.0} />

                    <RequestResultPanel project request_info save_response set_save_response data=response params/>

                </div>
            </div>
        </Show>
    }
}

fn create_req_watchers(
    params: ReadSignal<RequestParams>,
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    set_request_info: WriteSignal<RequestInfo>,
) {
    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    project,
                    request_info.read_untracked().id,
                    RequestFieldKind::Url,
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
                set_stored_value(
                    project,
                    request_info.read_untracked().id,
                    RequestFieldKind::Method,
                    value.to_string(),
                );
                set_request_info.write().method = value.to_owned();
            }
        },
        false,
    );

    create_watcher(params.read_untracked().body, RequestFieldKind::Body, project, request_info);

    Effect::watch(
        move || params.read_untracked().body_type,
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    project,
                    request_info.read_untracked().id,
                    RequestFieldKind::BodyType,
                    value.read_untracked().to_string(),
                )
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().headers.get(),
        move |value, _prev, _| {
            set_stored_value(
                project,
                request_info.read_untracked().id,
                RequestFieldKind::Headers,
                headers_to_string(value),
            )
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().body_formencoded.get(),
        move |value, _prev, _| {
            set_stored_value(
                project,
                request_info.read_untracked().id,
                RequestFieldKind::BodyFormencoded,
                body_form_to_string(value),
            )
        },
        false,
    );
}

fn create_request_info_watcher(
    params: ReadSignal<RequestParams>,
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    send_btn_ref: NodeRef<Button>,
    set_response: WriteSignal<Option<RestClientResponse>>,
    set_save_response: WriteSignal<bool>,
    set_body_tab_selected: WriteSignal<usize>,
    messages: Messages,
) {
    Effect::watch(
        move || request_info.get(),
        move |value, prev, _| {
            let id = value.id;
            let project_id = project.get_untracked();

            if value.autorun
                && (prev.is_none() || !prev.unwrap().autorun)
                && let Some(send_btn) = send_btn_ref.get_untracked()
            {
                send_btn.click();
            }

            if prev.is_none()
                || id != prev.unwrap().id
                || project_id.parse::<i32>().unwrap_or(0) != prev.unwrap().project_id
            {
                set_response.set(None);
                params.read_untracked().set_url.set(value.url.to_owned());
                params.read_untracked().set_method.set(value.method.to_owned());
                params.read_untracked().set_body.set(get_stored_value(
                    RequestFieldKind::Body,
                    "".to_owned(),
                    project.read_untracked().as_str(),
                    id,
                ));

                params.read_untracked().set_body_type.set(
                    RequestBodyKind::from_str(&get_stored_value(
                        RequestFieldKind::BodyType,
                        "".to_owned(),
                        project.read_untracked().as_str(),
                        id,
                    ))
                    .unwrap_or_default(),
                );
                set_body_tab_selected.set(
                    match params.read_untracked().body_type.get_untracked() {
                        RequestBodyKind::Formencoded => 1,
                        RequestBodyKind::Text => 0,
                    },
                );

                params.read_untracked().set_headers.set(load_headers(&project_id, id));
                params
                    .read_untracked()
                    .set_body_formencoded
                    .set(load_body_formencoded(&project_id, id));

                let save_response = get_stored_value(
                    RequestFieldKind::SaveResponse,
                    "false".to_owned(),
                    project.read_untracked().as_str(),
                    request_info.read_untracked().id,
                )
                .parse::<bool>()
                .unwrap();
                set_save_response.set(save_response);
                if save_response {
                    let data_str = get_stored_value(
                        RequestFieldKind::SaveResponseData,
                        "".to_owned(),
                        project.read_untracked().as_str(),
                        request_info.read_untracked().id,
                    );
                    if !data_str.is_empty() {
                        match serde_json::from_str::<RestClientResponse>(&data_str) {
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

fn create_watcher(
    value: ReadSignal<String>,
    field: RequestFieldKind,
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
) {
    Effect::watch(
        move || value.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    project,
                    request_info.read_untracked().id,
                    field,
                    value.to_string(),
                )
            }
        },
        false,
    );
}

fn create_watcher_bool(
    value: ReadSignal<bool>,
    field: RequestFieldKind,
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
) {
    Effect::watch(
        move || value.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    project,
                    request_info.read_untracked().id,
                    field,
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

fn load_headers(project_id: &str, request_id: i32) -> Vec<CustomHeader> {
    let stored_value =
        get_stored_value(RequestFieldKind::Headers, "".to_owned(), project_id, request_id);
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

fn load_body_formencoded(project_id: &str, request_id: i32) -> Vec<RequestBodyFormValue> {
    let stored_value =
        get_stored_value(RequestFieldKind::BodyFormencoded, "".to_owned(), project_id, request_id);
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
