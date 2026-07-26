use crate::{
    common::ui_utils::get_browser_width, components::layout::drag_splitter::DragSplitter,
    domain::rest_client::ui::request_result_panel::ReqResultData, i18n::*,
};
use leptos::{
    html::{Button, Div},
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

    let (url, set_url) = signal("".to_owned());
    let (method, set_method) = signal("".to_owned());
    let (body, set_body) = signal("".to_owned());
    let (headers, set_headers) = signal(Vec::<CustomHeader>::new());
    let (params, _set_params) = signal(RequestParams {
        url,
        set_url,
        method,
        set_method,
        body,
        set_body,
        headers,
        set_headers,
    });

    let screen_width = get_browser_width().unwrap();
    let min_params_width = screen_width / 6;

    let params_ref = NodeRef::<Div>::new();
    let send_btn_node_ref = NodeRef::<Button>::new();

    let (response, set_response) = signal(None);

    let _ = Effect::new(move || {
        params.read_untracked().set_headers.set(load_headers(request_info.read_untracked().id));
    });

    create_request_info_watcher(params, request_info, send_btn_node_ref, set_response);
    create_req_watchers(params, request_info, set_request_info);

    view! {
        <Show when=move || { request_info.read().id > 0 }
            fallback=move || view! { <div class="flex-1 flex h-[94dvh] items-center justify-center">{t!(i18n, rest_client_request_not_selected_msg)}</div> }
        >
            <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
                <RequestParamsPanel node_ref=params_ref send_btn_node_ref
                    params on_result=move|res| {
                        set_response.set(Some(res));
                    }
                />

                <DragSplitter target_ref=params_ref local_store_prop_name="rc_params_width"
                    min_width={min_params_width} max_width={screen_width - (screen_width / 3)}
                    default_width={min_params_width} />

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
}

fn create_request_info_watcher(
    params: ReadSignal<RequestParams>,
    request_info: ReadSignal<RequestInfo>,
    send_btn_ref: NodeRef<Button>,
    set_response: WriteSignal<Option<ReqResultData>>,
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
                params.read_untracked().set_body.set(get_stored_value(
                    "rc_body",
                    "".to_owned(),
                    id,
                ));
                params.read_untracked().set_headers.set(load_headers(id));
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
