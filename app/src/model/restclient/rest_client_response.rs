use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum RestClientResponseBody {
    #[default]
    None,
    Text(String),
    Attachment(String),
    Image,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct RestClientResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: RestClientResponseBody,
    pub request_raw: String,
    pub error: Option<String>,
}
