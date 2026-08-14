use std::str::FromStr;

use crate::{
    components::layout::{
        drag_splitter::DragSplitter,
        message_banner::{Messages, show_error},
    },
    domain::rest_client::{
        model::{
            request_params::{CustomHeaders, RequestBodyFormValue, RequestBodyKind, RequestParams},
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
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().unwrap();

    let (params, _set_params) = signal(RequestParams::new());
    let (response, set_response) = signal(None);

    let params_ref = NodeRef::<Div>::new();

    create_request_watcher(params, rc_context.clone(), set_response, messages);
    create_params_watchers(params, rc_context.clone());
    create_response_watcher(params, rc_context.clone(), response);

    view! {
        <div class="flex-1 flex items-center justify-center"
            class:hidden=move || { rc_context.request.read().id > 0 }>
            {t!(i18n, rest_client_request_not_selected_msg)}
        </div>

        <div class="flex-2 flex flex-col gap-2 px-2 py-4 text-xs md:text-base"
            class:hidden=move || { rc_context.request.read().id == 0 }
            >
            <RequestParamsUrl params set_response />

            <div class="flex-1 flex flex-col md:flex-row gap-2 text-xs md:text-base">
                <RequestParamsPanel node_ref=params_ref params />

                <DragSplitter
                    class_name="hidden md:block".to_owned()
                    target_ref=params_ref
                    local_store_prop_name=move || "params_width".to_owned()
                    min_scr_ration={1.0 / 6.0}
                    max_scr_ration={1.0 / 2.0}
                    default_scr_ration={1.0 / 6.0} />

                <RequestResultPanel response params/>

            </div>
        </div>
    }
}

fn create_response_watcher(
    params: ReadSignal<RequestParams>,
    rc_context: RestClientContext,
    response: ReadSignal<Option<RestClientResponse>>,
) {
    Effect::watch(
        move || response.get(),
        move |value, _prev, _| {
            if *params.read_untracked().save_response.read_untracked()
                && let Some(response) = value
            {
                let json_string = serde_json::to_string(&response).unwrap();
                set_stored_value(
                    rc_context.project.read_only(),
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::SaveResponseData,
                    json_string,
                )
            } else {
                delete_stored_value(
                    rc_context.project.read_only(),
                    rc_context.request.read_only(),
                    RequestFieldKind::SaveResponseData,
                )
            }
        },
        false,
    );
}

fn create_request_watcher(
    params: ReadSignal<RequestParams>,
    rc_context: RestClientContext,
    set_response: WriteSignal<Option<RestClientResponse>>,
    messages: Messages,
) {
    Effect::watch(
        move || rc_context.request.get(),
        move |value, prev, _| {
            if prev.is_none()
                || value.id != prev.unwrap().id
                || rc_context.project.read_untracked().parse::<i32>().unwrap_or(0)
                    != prev.unwrap().project_id
            {
                params.read_untracked().url.set(value.url.to_owned());
                params.read_untracked().method.set(value.method.to_owned());
                params.read_untracked().headers.set(CustomHeaders::read_from_store(
                    &rc_context.project.read_untracked(),
                    value.id,
                ));

                params.read_untracked().params_tab_selected.set(
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
                    .body_formencoded
                    .set(load_body_formencoded(&rc_context.project.read_untracked(), value.id));
                params.read_untracked().body.set(get_stored_value(
                    RequestFieldKind::Body,
                    "".to_owned(),
                    rc_context.project.read_untracked().as_str(),
                    value.id,
                ));
                params.read_untracked().body_type.set(
                    RequestBodyKind::from_str(&get_stored_value(
                        RequestFieldKind::BodyType,
                        "".to_owned(),
                        rc_context.project.read_untracked().as_str(),
                        value.id,
                    ))
                    .unwrap_or_default(),
                );

                let save_response = get_stored_value(
                    RequestFieldKind::SaveResponse,
                    "false".to_owned(),
                    rc_context.project.read_untracked().as_str(),
                    rc_context.request.read_untracked().id,
                )
                .parse::<bool>()
                .unwrap_or_default();

                params.read_untracked().save_response.set(save_response);
                if save_response {
                    let data_str = get_stored_value(
                        RequestFieldKind::SaveResponseData,
                        "".to_owned(),
                        rc_context.project.read_untracked().as_str(),
                        rc_context.request.read_untracked().id,
                    );
                    if !data_str.is_empty() {
                        match serde_json::from_str::<RestClientResponse>(&data_str) {
                            Ok(data) => set_response.set(Some(data)),
                            Err(err) => {
                                set_response.set(None);
                                show_error(err.to_string(), messages);
                            }
                        }
                    }
                } else {
                    set_response.set(None);
                }

                params.read_untracked().formatting.set(
                    get_stored_value(
                        RequestFieldKind::Formatting,
                        "true".to_owned(),
                        rc_context.project.read_untracked().as_str(),
                        rc_context.request.read_untracked().id,
                    )
                    .parse::<bool>()
                    .unwrap_or(false),
                );
            }
        },
        false,
    );
}

fn create_params_watchers(params: ReadSignal<RequestParams>, rc_context: RestClientContext) {
    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project.read_only(),
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::Url,
                    value.to_string(),
                );
                rc_context.request.write().url = value.to_owned();
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().method.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project.read_only(),
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::Method,
                    value.to_string(),
                );
                rc_context.request.write().method = value.to_owned();
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().params_tab_selected.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project.read_only(),
                    rc_context.request.read_untracked().id,
                    RequestFieldKind::ParamsTab,
                    value.to_string(),
                )
            }
        },
        false,
    );

    create_watcher(
        params.read_untracked().body.read_only(),
        RequestFieldKind::Body,
        rc_context.clone(),
    );

    Effect::watch(
        move || params.read_untracked().body_type.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                set_stored_value(
                    rc_context.project.read_only(),
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
                rc_context.project.read_only(),
                rc_context.request.read_untracked().id,
                RequestFieldKind::Headers,
                value.to_string(),
            )
        },
        false,
    );

    create_watcher_bool(
        params.read_untracked().save_response.read_only(),
        RequestFieldKind::SaveResponse,
        rc_context.clone(),
    );
    create_watcher_bool(
        params.read_untracked().formatting.read_only(),
        RequestFieldKind::Formatting,
        rc_context.clone(),
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
                    rc_context.project.read_only(),
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
                    rc_context.project.read_only(),
                    rc_context.request.read_untracked().id,
                    field,
                    value.to_string(),
                )
            }
        },
        false,
    );
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
