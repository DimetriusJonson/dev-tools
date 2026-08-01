pub mod request_body_form_panel;
pub mod request_headers_panel;
pub mod request_panel;
pub mod request_params;
pub mod request_params_panel;
pub mod request_popup_menu;
pub mod request_result_panel;
pub mod rest_client_explorer;
pub mod rest_client_project_selector;

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
