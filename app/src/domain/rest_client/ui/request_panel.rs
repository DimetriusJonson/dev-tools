use crate::{
    components::layout::drag_splitter::DragSplitter,
    domain::rest_client::{
        model::request_params::{CustomHeader, RequestBodyKind, RequestInfo, RequestParams},
        ui::request_params_url::RequestParamsUrl,
        util::request_store::{RequestFieldKind, delete_stored_value, set_stored_value},
    },
    i18n::*,
    model::restclient::rest_client_response::RestClientResponse,
};
use leptos::{html::Div, prelude::*};

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

    let (response, set_response) = signal(None);

    create_req_watchers(params, project, request_info, set_request_info);
    create_watcher_bool(save_response, RequestFieldKind::SaveResponse, project, request_info);

    view! {
        <Show when=move || { request_info.read().id > 0 }
            fallback=move || view! { <div class="flex-1 flex items-center justify-center">{t!(i18n, rest_client_request_not_selected_msg)}</div> }
        >
            <div class="flex-2 flex flex-col gap-2 px-2 py-4 text-xs md:text-base">
                <RequestParamsUrl
                    project
                    params
                    request_info
                    set_request_info
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

                    <RequestResultPanel project request_info set_request_info save_response set_save_response response set_response params/>

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
        move || params.read_untracked().body_type.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    project,
                    request_info.read_untracked().id,
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
                project,
                request_info.read_untracked().id,
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
