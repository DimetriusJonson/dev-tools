use leptos::prelude::{GetUntracked, ReadUntracked};

use crate::domain::rest_client::{model::request_params::{RequestBodyKind, RequestParams}, util::rest_client_utils::formencoded_to_str};

pub fn build_curl_bash_cmd(request_params: &RequestParams) -> String {
    let mut result = String::new();

    result.push_str("curl ");

    // url
    result.push_str(&format!("\'{}\'", request_params.url.read_untracked()));

    //method
    result.push_str(&format!(" \\\r\n-X {}", &request_params.method.read_untracked()));

    //headers
    for header in request_params.headers.read_untracked().iter() {
        result.push_str(" \\\r\n");
        result.push_str(&format!(
            "-H \'{}: {}\'",
            &header.name.read_untracked(),
            &header.value.read_untracked()
        ));
    }

    // body
    let body = match request_params.body_type.get_untracked() {
        RequestBodyKind::Text => request_params.body.get_untracked(),
        RequestBodyKind::Formencoded => {
            match formencoded_to_str(request_params.body_formencoded.get_untracked()) {
                Ok(url) => url,
                Err(_err) => "".to_owned()
            }
        }
    };

    if !body.is_empty() {
        result.push_str(" \\\r\n");
        result.push_str(&format!("--data-raw \'{}\'", body));
    }

    result
}
