use std::collections::HashMap;

use leptos::prelude::GetUntracked;

use crate::domain::rest_client::model::request_params::RequestBodyFormValue;

pub fn formencoded_to_str(
    form_values: Vec<RequestBodyFormValue>,
) -> Result<String, serde_urlencoded::ser::Error> {
    let map: HashMap<String, String> = form_values
        .into_iter()
        .map(|fv| (fv.name.get_untracked(), fv.value.get_untracked()))
        .collect();

    serde_urlencoded::to_string(&map)
}

pub fn body_form_to_string(form_values: &[RequestBodyFormValue]) -> String {
    let list: KeyValueVector =
        form_values.iter().map(|h| (h.name.get_untracked(), h.value.get_untracked())).collect();

    serde_json::to_string(&list).unwrap()
}

pub type KeyValueVector = Vec<(String, String)>;
