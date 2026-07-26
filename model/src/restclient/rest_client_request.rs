use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RestClientRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}
