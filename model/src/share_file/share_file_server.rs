use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ShareFileServerDto{
    pub url: String,
    pub description: String,
}