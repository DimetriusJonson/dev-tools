use std::collections::HashMap;

use leptos::{prelude::*, task::spawn_local};

use crate::domain::rest_client::util::request_store::{
    RequestFieldKind, generate_request_id, set_stored_requests_ids, set_stored_value,
};
use crate::i18n::*;
use crate::{
    common::{
        curl_parser::parser::parse_curl_cmd, ui_utils::paste_from_clipboard,
    },
    components::layout::message_banner::{Messages, show_error},
    domain::rest_client::model::request_params::RequestInfo,
};

#[component]
pub fn RestClientCUrlButton(
    project: ReadSignal<String>,
    set_current_request: WriteSignal<RequestInfo>,
    requests: ReadSignal<Vec<RwSignal<RequestInfo>>>,
    set_requests: WriteSignal<Vec<RwSignal<RequestInfo>>>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();

    let on_import_c_url = move |_| {
        spawn_local(async move {
            if let Some(curl_cmd) = paste_from_clipboard().await {
                match parse_curl_cmd(&curl_cmd) {
                    Ok(parsed_request) => {
                        let request = RequestInfo::new(
                            generate_request_id(project),
                            project.read_untracked().parse().unwrap_or(0),
                            parsed_request.url.to_owned(),
                            "".to_owned(),
                            parsed_request.method.to_string(),
                        );

                        set_requests.write().push(RwSignal::new(request.clone()));

                        set_current_request.set(request.clone());
                        set_stored_requests_ids(project, &requests.read_untracked());
                        set_stored_value(project, request.id, RequestFieldKind::Url, request.url);
                        set_stored_value(project, request.id, RequestFieldKind::Method, request.method);

                        let content_type = parsed_request
                            .headers
                            .iter()
                            .find(|h| h.0.as_str().to_lowercase() == "content-type")
                            .map(|h| h.1.to_str().ok())
                            .unwrap_or(None);

                        if let Some(content_type) = content_type && content_type.to_lowercase() == "application/x-www-form-urlencoded" {
                            if let Ok(map) = serde_urlencoded::from_str::<HashMap<String, String>>(
                                &parsed_request.body.join("\n"),
                            ) {
                                if let Ok(json) = serde_json::to_string(
                                    &map.into_iter().collect::<Vec<(String, String)>>(),
                                ) {
                                    set_stored_value(project, request.id, RequestFieldKind::BodyFormencoded, json);
                                    set_stored_value(
                                        project,
                                        request.id,
                                        RequestFieldKind::BodyType,
                                        "formencoded".to_owned(),
                                    );
                                }
                            }
                        } else if let Some(content_type) = content_type && content_type.to_lowercase().contains("json") { 
                            set_stored_value(
                                project,
                                request.id,
                                RequestFieldKind::BodyJson,
                                parsed_request.body.join("\n"),
                            );
                            set_stored_value(project, request.id, RequestFieldKind::BodyType, "text".to_owned());
                        } else {
                            set_stored_value(
                                project,
                                request.id,
                                RequestFieldKind::Body,
                                parsed_request.body.join("\n"),
                            );
                            set_stored_value(project, request.id, RequestFieldKind::BodyType, "text".to_owned());
                        }

                        set_stored_value(
                            project,
                            request.id,
                            RequestFieldKind::Headers,
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

    view! {

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


    }
}
