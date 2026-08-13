use leptos::{html::Div, prelude::*};

use crate::{
    common::ui_utils::get_accept_language,
    domain::rest_client::ui::{
        rest_client_curl_button::RestClientCUrlButton,
        rest_client_explorer_row::RestClientExplorerRow,
        rest_client_project_selector::ProjectSelector,
    },
    i18n::*,
};
use crate::{
    components::ui::button::{Button, ButtonWidth},
    domain::rest_client::{
        model::request_params::RequestInfo,
        util::request_store::{
            RequestFieldKind, delete_stored_request, generate_request_id, get_stored_requests_ids,
            get_stored_value, set_stored_requests_ids, set_stored_value,
        },
    },
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

    let (requests, set_requests) = signal(Vec::<RwSignal<RequestInfo>>::new());

    let on_create_request = move |_| {
        let request = RequestInfo::new(
            generate_request_id(project),
            project.read_untracked().parse().unwrap_or(0),
            format!("http://{}/test_json", window().location().host().unwrap()),
            "".to_owned(),
            "GET".to_owned(),
        );

        set_requests.write().push(RwSignal::new(request.clone()));

        set_current_request.set(request.clone());
        set_stored_requests_ids(project, &requests.read_untracked());
        set_stored_value(project, request.id, RequestFieldKind::Url, request.url);
        set_stored_value(project, request.id, RequestFieldKind::Method, request.method);

        let headers = [
            ("Accept".to_owned(), "application/json".to_owned()),
            ("Accept-Language".to_owned(), get_accept_language()),
            ("User-Agent".to_owned(), "WebDevUsefulTools Client".to_owned()),
        ];
        set_stored_value(
            project,
            request.id,
            RequestFieldKind::Headers,
            headers.iter().map(|h| format!("{}:{}", h.0, h.1)).collect::<Vec<String>>().join("\n"),
        );
    };

    Effect::watch(
        move || project.get(),
        move |value, _prev, _| {
            set_requests.set(load_requests(value));
            if let Some(first) = requests.read_untracked().first() {
                set_current_request.set(first.get_untracked());
            } else {
                set_current_request.set(RequestInfo::new_empty());
            }
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
                        set_stored_requests_ids(project, &requests.read_untracked());
                        delete_stored_request(project.read_untracked().as_str(),  r.read_untracked().id);
                    });
                }/>

                <div class="flex gap-2">
                    <Button
                        title=move || "".to_owned()
                        label=move || t_display!(i18n, rest_client_explorer_create_request).to_string()
                        class_name="w-full".to_owned()
                        button_width=ButtonWidth::Auto
                        loading=move || false
                        on_click=on_create_request
                        disabled=move || false
                    />

                    <RestClientCUrlButton project set_current_request requests set_requests/>
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

fn load_requests(project_id: &str) -> Vec<RwSignal<RequestInfo>> {
    get_stored_requests_ids(project_id)
        .iter()
        .map(|id| {
            let url = get_stored_value(RequestFieldKind::Url, "".to_owned(), project_id, *id);
            let name = get_stored_value(RequestFieldKind::Name, "".to_owned(), project_id, *id);
            let method = get_stored_value(RequestFieldKind::Method, "".to_owned(), project_id, *id);
            RwSignal::new(RequestInfo::new(*id, project_id.parse().unwrap_or(0), url, name, method))
        })
        .collect()
}
