use std::collections::HashMap;

use leptos::{html::Div, prelude::*, task::spawn_local};

use crate::{
    common::{curl_parser::parser::parse_curl_cmd, ui_utils::get_accept_language},
    components::layout::message_banner::{Messages, show_error},
    domain::rest_client::ui::{
        build_rc_req_store_key, build_rc_store_key, delete_request,
        rest_client_explorer_row::RestClientExplorerRow,
        rest_client_project_selector::ProjectSelector, save_requests_ids,
    },
    i18n::*,
};
use crate::{
    common::{
        local_store::{get_local_store_value, set_local_store_value},
        ui_utils::paste_from_clipboard,
    },
    components::ui::button::{Button, ButtonWidth},
    domain::rest_client::ui::request_params::RequestInfo,
};

#[component]
pub fn RestClientExplorer(
    project: ReadSignal<String>,
    set_project: WriteSignal<String>,
    current_request: ReadSignal<RequestInfo>,
    set_current_request: WriteSignal<RequestInfo>,
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (requests, set_requests) = signal(Vec::<RwSignal<RequestInfo>>::new());

    let on_create_request = move |_| {
        let request = RequestInfo::new(
            generate_request_id(project),
            format!("http://{}/test_json", window().location().host().unwrap()),
            "".to_owned(),
            "GET".to_owned(),
        );

        set_requests.write().push(RwSignal::new(request.clone()));

        set_current_request.set(request.clone());
        save_requests_ids(project, &requests.read_untracked());
        set_local_store_value(
            &build_rc_req_store_key(project.read_untracked().as_str(), request.id, "url"),
            request.url,
        );
        set_local_store_value(
            &build_rc_req_store_key(project.read_untracked().as_str(), request.id, "method"),
            request.method,
        );

        let headers = [
            ("Accept".to_owned(), "application/json".to_owned()),
            ("Accept-Language".to_owned(), get_accept_language()),
            ("User-Agent".to_owned(), "WebDevUsefulTools Client".to_owned()),
        ];
        set_local_store_value(
            &build_rc_req_store_key(project.read_untracked().as_str(), request.id, "headers"),
            headers.iter().map(|h| format!("{}:{}", h.0, h.1)).collect::<Vec<String>>().join("\n"),
        );
    };

    let on_import_c_url = move |_| {
        spawn_local(async move {
            if let Some(curl_cmd) = paste_from_clipboard().await {
                match parse_curl_cmd(&curl_cmd) {
                    Ok(parsed_request) => {
                        let request = RequestInfo::new(
                            generate_request_id(project),
                            parsed_request.url.to_owned(),
                            "".to_owned(),
                            parsed_request.method.to_string(),
                        );

                        set_requests.write().push(RwSignal::new(request.clone()));

                        set_current_request.set(request.clone());
                        save_requests_ids(project, &requests.read_untracked());
                        set_local_store_value(
                            &build_rc_req_store_key(
                                project.read_untracked().as_str(),
                                request.id,
                                "url",
                            ),
                            request.url,
                        );
                        set_local_store_value(
                            &build_rc_req_store_key(
                                project.read_untracked().as_str(),
                                request.id,
                                "insecure",
                            ),
                            parsed_request.insecure.to_string(),
                        );
                        set_local_store_value(
                            &build_rc_req_store_key(
                                project.read_untracked().as_str(),
                                request.id,
                                "method",
                            ),
                            request.method,
                        );

                        if let Some(content_type) = parsed_request
                            .headers
                            .iter()
                            .find(|h| h.0.as_str().to_lowercase() == "content-type")
                            .map(|h| h.1.to_str().ok())
                            .unwrap_or(None)
                            && content_type.to_lowercase() == "application/x-www-form-urlencoded"
                        {
                            if let Ok(map) = serde_urlencoded::from_str::<HashMap<String, String>>(
                                &parsed_request.body.join("\n"),
                            ) {
                                if let Ok(json) = serde_json::to_string(
                                    &map.into_iter().collect::<Vec<(String, String)>>(),
                                ) {
                                    set_local_store_value(
                                        &build_rc_req_store_key(
                                            project.read_untracked().as_str(),
                                            request.id,
                                            "body_formencoded",
                                        ),
                                        json,
                                    );
                                    set_local_store_value(
                                        &build_rc_req_store_key(
                                            project.read_untracked().as_str(),
                                            request.id,
                                            "body_type",
                                        ),
                                        "formencoded".to_owned(),
                                    );
                                }
                            }
                        } else {
                            set_local_store_value(
                                &build_rc_req_store_key(
                                    project.read_untracked().as_str(),
                                    request.id,
                                    "body",
                                ),
                                parsed_request.body.join("\n"),
                            );
                            set_local_store_value(
                                &build_rc_req_store_key(
                                    project.read_untracked().as_str(),
                                    request.id,
                                    "body_type",
                                ),
                                "text".to_owned(),
                            );
                        }

                        set_local_store_value(
                            &build_rc_req_store_key(
                                project.read_untracked().as_str(),
                                request.id,
                                "headers",
                            ),
                            parsed_request
                                .headers
                                .iter()
                                .map(|h| format!("{}:{}", h.0, h.1.to_str().unwrap_or("")))
                                .collect::<Vec<String>>()
                                .join("\n"),
                        );
                    }
                    Err(err) => show_error(format!("Error: {}", err), messages),
                }
            }
        })
    };

    Effect::watch(
        move || project.get(),
        move |value, _prev, _| {
            set_requests.set(load_requests(value));
        },
        false,
    );

    Effect::watch(
        move || current_request.get(),
        move |value, _prev, _| {
            if let Some(req) =
                requests.read_untracked().iter().find(|r| r.read_untracked().id == value.id)
            {
                req.write().url = value.url.to_owned();
                req.write().method = value.method.to_owned();
            }
        },
        false,
    );

    view! {
        <div node_ref=node_ref class="flex-1 max-w-40 sm:max-w-none sm:flex-none flex flex-col gap-y-0 dark:text-white">
            <div class="flex flex-col p-2 md:p-4 gap-y-2">
                <ProjectSelector project set_project on_delete=move |_| {
                    requests.read_untracked().iter().for_each(|r| {
                        save_requests_ids(project, &requests.read_untracked());
                        delete_request(project.read_untracked().as_str(),  r.read_untracked().id);
                    });
                }/>

                <div class="flex gap-2">
                    <Button
                        label=move || t_display!(i18n, rest_client_explorer_create_request).to_string()
                        class_name="w-full".to_owned()
                        button_width=ButtonWidth::Auto
                        loading=move || false
                        on_click=on_create_request
                        disabled=move || false
                    />

                    <div class="w-10 h-8dvh md:h-10 inline-flex items-center justify-center hover:bg-sky-500/30 dark:hover:bg-sky-300/30 cursor-pointer rounded-xl p-2 border border-gray-500"
                        title=move || t_string!(i18n, rest_client_curl_import_title)
                        on:click=on_import_c_url
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1021 854" class="stroke-sky-500 dark:stroke-sky-300">
                            <g fill="none" stroke-width="34" transform="translate(17 17)">
                                <circle cx="58" cy="529" r="58"/>
                                <circle cx="58" cy="263" r="58"/>
                                <path stroke-width="100" d="M596 101 234 721"/>
                                <circle cx="210" cy="762" r="58"/>
                                <circle cx="621" cy="58" r="58"/>
                                <path stroke-width="100" d="M904 101 542 721"/>
                                <circle cx="929" cy="58" r="58"/>
                                <circle cx="518" cy="762" r="58"/>
                            </g>
                        </svg>
                    </div>
                </div>

            </div>

            { move || { requests.get().into_iter()
                .map(|request| {
                    view! {<RestClientExplorerRow project request current_request set_current_request requests set_requests/>}
                }).collect_view()
            }}
        </div>
    }
}

fn generate_request_id(project: ReadSignal<String>) -> i32 {
    let requests_ids = load_requests_ids(project.read_untracked().as_str());
    if !requests_ids.is_empty()
        && let Some(id) = requests_ids.iter().max()
    {
        return *id + 1;
    }

    1
}

fn load_requests_ids(project_id: &str) -> Vec<i32> {
    let requests_ids =
        get_local_store_value(&build_rc_store_key(project_id, "requests_ids"), "".to_owned());

    if !requests_ids.is_empty() {
        requests_ids.split(",").map(|s| s.parse::<i32>().unwrap()).collect()
    } else {
        vec![]
    }
}

fn load_requests(project_id: &str) -> Vec<RwSignal<RequestInfo>> {
    load_requests_ids(project_id)
        .iter()
        .map(|id| {
            let url = get_local_store_value(
                &build_rc_req_store_key(project_id, *id, "url"),
                "".to_owned(),
            );
            let name = get_local_store_value(
                &build_rc_req_store_key(project_id, *id, "name"),
                "".to_owned(),
            );
            let method = get_local_store_value(
                &build_rc_req_store_key(project_id, *id, "method"),
                "".to_owned(),
            );
            RwSignal::new(RequestInfo::new(*id, url, name, method))
        })
        .collect()
}
