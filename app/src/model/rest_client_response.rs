use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RestClientResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}
