use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Getters)]
#[get = "pub"]
pub struct ShareFileServerDto {
    url: String,
    description: String,
}

impl ShareFileServerDto {
    pub fn new(url: String, description: String) -> Self {
        Self { url, description }
    }
}
