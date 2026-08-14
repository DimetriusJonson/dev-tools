use std::str::FromStr;

use crate::{
    components::layout::drag_splitter::DragSplitter,
    domain::rest_client::{
        model::{
            request_params::{CustomHeader, RequestBodyFormValue, RequestBodyKind, RequestParams},
            rest_client_context::RestClientContext,
        },
        ui::request_params_url::RequestParamsUrl,
        util::{
            request_store::{
                RequestFieldKind, delete_stored_value, get_stored_value, set_stored_value,
            },
            rest_client_utils::KeyValueVector,
        },
    },
    i18n::*,
    model::restclient::rest_client_response::RestClientResponse,
};
use leptos::{html::Div, leptos_dom::logging::console_log, prelude::*};
use uuid::Uuid;

use crate::domain::rest_client::ui::{
    request_params_panel::RequestParamsPanel, request_result_panel::RequestResultPanel,
};

#[component]
pub fn RequestPanel() -> impl IntoView {
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().unwrap();

    let (url, set_url) = signal("".to_owned());
    let (method, set_method) = signal("".to_owned());
    let (body, set_body) = signal("".to_owned());
    let (params_tab_selected, set_params_tab_selected) = signal(0);
    let (body_type, set_body_type) = signal(RequestBodyKind::Text);
    let (body_formencoded, set_body_formencoded) = signal(Vec::new());
    let (headers, set_headers) = signal(Vec::<CustomHeader>::new());
    let (params, _set_params) = signal(RequestParams {
        url,
        set_url,
        method,
        set_method,
        params_tab_selected,
        set_params_tab_selected,
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
    let (response, set_response) = signal(None);

    let params_ref = NodeRef::<Div>::new();

    Effect::watch(
        move || rc_context.request.get(),
        move |value, prev, _| {
            if prev.is_none()
                || value.id != prev.unwrap().id
                || rc_context.project.read_untracked().parse::<i32>().unwrap_or(0)
                    != prev.unwrap().project_id
            {
                params.read_untracked().set_url.set(value.url.to_owned());
                params.read_untracked().set_method.set(value.method.to_owned());
                params
                    .read_untracked()
                    .set_headers
                    .set(load_headers(&rc_context.project.read_untracked(), value.id));

                params.read_untracked().set_params_tab_selected.set(
                    get_stored_value(
                        RequestFieldKind::ParamsTab,
                        "0".to_owned(),
                        &rc_context.project.read_untracked(),
                        rc_context.request.read_untracked().id,
                    )
                    .parse()
                    .unwrap_or(0),
                );

                params
                    .read_untracked()
                    .set_body_formencoded
                    .set(load_body_formencoded(&rc_context.project.read_untracked(), value.id));
                params.read_untracked().set_body.set(get_stored_value(
                    RequestFieldKind::Body,
                    "".to_owned(),
                    rc_context.project.read_untracked().as_str(),
                    value.id,
                ));
                params.read_untracked().set_body_type.set(
                    RequestBodyKind::from_str(&get_stored_value(
                        RequestFieldKind::BodyType,
                        "".to_owned(),
                        rc_context.project.read_untracked().as_str(),
                        value.id,
                    ))
                    .unwrap_or_default(),
                );
            }
        },
        false,
    );

    create_req_watchers(params, rc_context.clone());
    create_watcher_bool(save_response, RequestFieldKind::SaveResponse, rc_context.clone());

    view! {
        <Show when=move || { rc_context.request.read().id > 0 }
            fallback=move || view! { <div class="flex-1 flex items-center justify-center">{t!(i18n, rest_client_request_not_selected_msg)}</div> }
        >
            <div class="flex-2 flex flex-col gap-2 px-2 py-4 text-xs md:text-base">
                <RequestParamsUrl
                    params
                    on_result=move|res: RestClientResponse| {
                        if *save_response.read_untracked() {
                            let json_string = serde_json::to_string(&res).unwrap();
                            set_stored_value(rc_context.project, rc_context.request.read_untracked().id, RequestFieldKind::SaveResponseData, json_string)
                        } else {
                            delete_stored_value(rc_context.project, rc_context.request, RequestFieldKind::SaveResponseData)
                        }
                        set_response.set(Some(res));
                    }
                />

                <div class="flex-1 flex flex-col md:flex-row gap-2 text-xs md:text-base">
                    <RequestParamsPanel
                        node_ref=params_ref
                        params
                    />

                    <DragSplitter
                        class_name="hidden md:block".to_owned()
                        target_ref=params_ref
                        local_store_prop_name=move || "params_width".to_owned()
                        min_scr_ration={1.0 / 6.0}
                        max_scr_ration={1.0 / 2.0}
                        default_scr_ration={1.0 / 6.0} />

                    <RequestResultPanel save_response set_save_response response set_response params/>

                </div>
            </div>
        </Show>
    }
}

fn create_req_watchers(params: ReadSignal<RequestParams>, rc_context: RestClientContext) {
    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project,
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::Url,
                    value.to_string(),
                );
                rc_context.set_request.write().url = value.to_owned();
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().method.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project,
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::Method,
                    value.to_string(),
                );
                rc_context.set_request.write().method = value.to_owned();
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().params_tab_selected.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project,
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::ParamsTab,
                    value.to_string(),
                )
            }
        },
        false,
    );

    create_watcher(params.read_untracked().body, RequestFieldKind::Body, rc_context.clone());

    Effect::watch(
        move || params.read_untracked().body_type.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project,
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::BodyType,
                    value.to_string(),
                )
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().headers.get(),
        move |value, _prev, _| {
            set_stored_value(
                rc_context.project,
                rc_context.request.read_untracked().id,
                RequestFieldKind::Headers,
                headers_to_string(value),
            )
        },
        false,
    );
}

fn create_watcher(
    value: ReadSignal<String>,
    field: RequestFieldKind,
    rc_context: RestClientContext,
) {
    Effect::watch(
        move || value.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project,
                    rc_context.request.read_untracked().id,
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
    rc_context: RestClientContext,
) {
    Effect::watch(
        move || value.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project,
                    rc_context.request.read_untracked().id,
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
    for line in stored_value.lines() {
        if let Some(index) = line.find(":") {
            let (name, set_name) = signal(line[..index].to_owned());
            let (value, set_value) = signal(line[index + 1..].to_owned());

            let header =
                CustomHeader { id: Uuid::new_v4().to_string(), name, set_name, value, set_value };
            result.push(header);
        }
    }

    result
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
            for value in values.iter() {
                let (name, set_name) = signal(value.0.to_owned());
                let (value, set_value) = signal(value.1.to_owned());

                let header = RequestBodyFormValue {
                    id: Uuid::new_v4().to_string(),
                    name,
                    set_name,
                    value,
                    set_value,
                };
                result.push(header);
            }
        }
        Err(err) => console_log(&format!("Error: {}", err)),
    }
    result
}
