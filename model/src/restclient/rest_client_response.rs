use getset::{CopyGetters, Getters, WithSetters};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum RestClientResponseBody {
    #[default]
    None,
    Text(String),
    Attachment(String),
    Image,
}

#[derive(Clone, Serialize, Deserialize, Default, Getters, CopyGetters, WithSetters)]
#[set_with = "pub"]
pub struct RestClientResponse {
    #[get_copy = "pub"]
    status_code: u16,
    #[get = "pub"]
    headers: Vec<(String, String)>,
    #[get = "pub"]
    body: RestClientResponseBody,
    #[get = "pub"]
    request_raw: String,
    #[get = "pub"]
    error: Option<String>,
    #[get_copy = "pub"]
    size: Option<u64>,
    #[get_copy = "pub"]
    request_time: u64,
}
