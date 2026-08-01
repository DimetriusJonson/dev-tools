use leptos::prelude::{ReadSignal, ReadUntracked, RwSignal};

use crate::{
    common::local_store::{delete_local_store_value, get_local_store_value, set_local_store_value},
    domain::rest_client::ui::request_params::RequestInfo,
};

pub fn build_rc_req_store_key(project_id: &str, request_id: i32, name: &str) -> String {
    if project_id.is_empty() || project_id == "0" {
        format!("{}-rc_{}", request_id, name)
    } else {
        format!("{}-{}-rc_{}", project_id, request_id, name)
    }
}

pub fn build_rc_store_key(project_id: &str, name: &str) -> String {
    if project_id.is_empty() {
        format!("rc_{}", name)
    } else {
        format!("{}-rc_{}", project_id, name)
    }
}

pub fn save_requests_ids(project: ReadSignal<String>, requests: &[RwSignal<RequestInfo>]) {
    if !requests.is_empty() {
        let value = requests
            .iter()
            .map(|r| r.read_untracked().id.to_string())
            .collect::<Vec<String>>()
            .join(",");
        set_local_store_value(
            &build_rc_store_key(project.read_untracked().as_str(), "requests_ids"),
            value,
        );
    } else {
        delete_local_store_value(&build_rc_store_key(
            project.read_untracked().as_str(),
            "requests_ids",
        ));
    }
}

pub fn delete_request(project_id: &str, request_id: i32) {
    let keys = vec![
        "url",
        "name",
        "method",
        "body",
        "headers",
        "insecure",
        "save_response",
        "body_type",
        "body_formencoded",
        "formencoded",
        "save_response_data",
        "headers_height",
    ];
    for key in keys {
        delete_local_store_value(&build_rc_req_store_key(project_id, request_id, key));
    }
}

pub fn generate_request_id(project: ReadSignal<String>) -> i32 {
    let requests_ids = load_requests_ids(project.read_untracked().as_str());
    if !requests_ids.is_empty()
        && let Some(id) = requests_ids.iter().max()
    {
        return *id + 1;
    }

    1
}

pub fn load_requests_ids(project_id: &str) -> Vec<i32> {
    let requests_ids =
        get_local_store_value(&build_rc_store_key(project_id, "requests_ids"), "".to_owned());

    if !requests_ids.is_empty() {
        requests_ids.split(",").map(|s| s.parse::<i32>().unwrap()).collect()
    } else {
        vec![]
    }
}

pub fn get_stored_value(name: &str, default: String, project_id: &str, request_id: i32) -> String {
    let key = build_rc_req_store_key(project_id, request_id, name);
    get_local_store_value(&key, default)
}

pub fn get_stored_value_as_bool(
    name: &str,
    default: bool,
    project_id: &str,
    request_id: i32,
) -> bool {
    let key = build_rc_req_store_key(project_id, request_id, name);
    let str = get_local_store_value(&key, default.to_string());
    str::parse(&str).unwrap_or_default()
}

pub fn set_stored_value(project: ReadSignal<String>, request_id: i32, name: &str, value: String) {
    if request_id == 0 {
        return;
    }

    set_local_store_value(
        &build_rc_req_store_key(project.read_untracked().as_str(), request_id, name),
        value,
    )
}

pub fn delete_stored_value(
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    name: &str,
) {
    delete_local_store_value(&build_rc_req_store_key(
        project.read_untracked().as_str(),
        request_info.read_untracked().id,
        name,
    ))
}
