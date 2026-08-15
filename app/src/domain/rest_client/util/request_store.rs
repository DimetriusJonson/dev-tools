use leptos::prelude::{ReadSignal, ReadUntracked, RwSignal};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::{
    common::local_store::{delete_local_store_value, get_local_store_value, set_local_store_value},
    domain::rest_client::model::request_params::{RequestInfo, RestClientProject},
};

#[derive(Clone, Copy, Debug, PartialEq, EnumString, EnumIter, Display)]
pub enum RequestFieldKind {
    #[strum(serialize = "url")]
    Url,
    #[strum(serialize = "name")]
    Name,
    #[strum(serialize = "method")]
    Method,
    #[strum(serialize = "body")]
    Body,
    #[strum(serialize = "headers")]
    Headers,
    #[strum(serialize = "save_response")]
    SaveResponse,
    #[strum(serialize = "body_type")]
    BodyType,
    #[strum(serialize = "params_tab")]
    ParamsTab,
    #[strum(serialize = "body_formencoded")]
    BodyFormencoded,
    #[strum(serialize = "save_response_data")]
    SaveResponseData,
    #[strum(serialize = "headers_height")]
    HeadersHeight,
    #[strum(serialize = "formatting")]
    Formatting,
}

pub fn build_request_stored_key(project_id: &str, request_id: i32, name: &str) -> String {
    if project_id.is_empty() || project_id == "0" {
        format!("{}-rc_{}", request_id, name)
    } else {
        format!("{}-{}-rc_{}", project_id, request_id, name)
    }
}

pub fn build_project_stored_key(project_id: &str, name: &str) -> String {
    if project_id.is_empty() {
        format!("rc_{}", name)
    } else {
        format!("{}-rc_{}", project_id, name)
    }
}

pub fn set_stored_requests_ids(project: ReadSignal<String>, requests: &[RwSignal<RequestInfo>]) {
    if !requests.is_empty() {
        let value = requests
            .iter()
            .map(|r| r.read_untracked().id.to_string())
            .collect::<Vec<String>>()
            .join(",");
        set_local_store_value(
            &build_project_stored_key(project.read_untracked().as_str(), "requests_ids"),
            value,
        );
    } else {
        delete_local_store_value(&build_project_stored_key(
            project.read_untracked().as_str(),
            "requests_ids",
        ));
    }
}

pub fn delete_stored_request(project_id: &str, request_id: i32) {
    for field in RequestFieldKind::iter() {
        delete_local_store_value(&build_request_stored_key(
            project_id,
            request_id,
            &field.to_string(),
        ));
    }
}

pub fn copy_stored_request(src_project_id: &str, src_request_id: i32, dst_project_id: &str, dst_request_id: i32) {
    for field in RequestFieldKind::iter() {
        let value = get_stored_value(field, "".to_owned(), src_project_id, src_request_id);
        set_local_store_value(
            &build_request_stored_key(
                dst_project_id,
                dst_request_id,
                &field.to_string(),
            ),
            value,
        );
    }
}

pub fn generate_request_id(project: ReadSignal<String>) -> i32 {
    let requests_ids = get_stored_requests_ids(project.read_untracked().as_str());
    if !requests_ids.is_empty()
        && let Some(id) = requests_ids.iter().max()
    {
        return *id + 1;
    }

    1
}

pub fn get_stored_requests_ids(project_id: &str) -> Vec<i32> {
    let requests_ids =
        get_local_store_value(&build_project_stored_key(project_id, "requests_ids"), "".to_owned());

    if !requests_ids.is_empty() {
        requests_ids.split(",").map(|s| s.parse::<i32>().unwrap()).collect()
    } else {
        vec![]
    }
}

pub fn get_stored_projects() -> Vec<RestClientProject> {
    let projects_value = get_local_store_value("rc_projects", "".to_owned());
    serde_json::from_str(&projects_value).unwrap_or(Vec::new())
}

pub fn set_stored_projects(projects: &Vec<RestClientProject>) {
    if let Ok(json) = serde_json::to_string(projects) {
        set_local_store_value("rc_projects", json)
    }
}

pub fn get_stored_current_project() -> String {
    get_local_store_value("rc_current_project", "".to_owned())
}

pub fn set_stored_current_project(value: String) {
    set_local_store_value("rc_current_project", value.to_owned())
}

pub fn get_stored_value(
    field: RequestFieldKind,
    default: String,
    project_id: &str,
    request_id: i32,
) -> String {
    let key = build_request_stored_key(project_id, request_id, &field.to_string());
    get_local_store_value(&key, default)
}

/*
pub fn get_stored_value_as_bool(
    field: RequestFieldKind,
    default: bool,
    project_id: &str,
    request_id: i32,
) -> bool {
    let key = build_request_stored_key(project_id, request_id, &field.to_string());
    let str = get_local_store_value(&key, default.to_string());
    str::parse(&str).unwrap_or_default()
}
 */

pub fn set_stored_value(
    project: ReadSignal<String>,
    request_id: i32,
    field: RequestFieldKind,
    value: String,
) {
    if request_id == 0 {
        return;
    }

    set_local_store_value(
        &build_request_stored_key(
            project.read_untracked().as_str(),
            request_id,
            &field.to_string(),
        ),
        value,
    )
}

pub fn delete_stored_value(
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    field: RequestFieldKind,
) {
    delete_local_store_value(&build_request_stored_key(
        project.read_untracked().as_str(),
        request_info.read_untracked().id,
        &field.to_string(),
    ))
}
