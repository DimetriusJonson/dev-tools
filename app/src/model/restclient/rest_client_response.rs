use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum RestClientResponseBody {
    None, Text(String), Attachment(String)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RestClientResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: RestClientResponseBody,
    pub request_raw: String,
    pub error: Option<String>,
}
