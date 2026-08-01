use leptos::prelude::{ReadSignal, ReadUntracked, RwSignal};

use crate::{common::local_store::{delete_local_store_value, get_local_store_value, set_local_store_value}, domain::rest_client::ui::request_params::RequestInfo};

pub mod request_body_form_panel;
pub mod request_headers_panel;
pub mod request_panel;
pub mod request_params;
pub mod request_params_panel;
pub mod request_popup_menu;
pub mod request_result_panel;
pub mod rest_client_explorer;
pub mod rest_client_project_selector;
pub mod rest_client_explorer_row;
pub mod rest_client_curl_button;

pub fn build_rc_req_store_key(project_id: &str, request_id: i32, name: &str) -> String {
    if project_id.is_empty() {
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
    let value = requests
        .iter()
        .map(|r| r.read_untracked().id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    set_local_store_value(
        &build_rc_store_key(project.read_untracked().as_str(), "requests_ids"),
        value,
    );
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

