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

    Ok(serde_urlencoded::to_string(&map)?)
}
