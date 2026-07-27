use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ShareFileInfoDto{
    pub file_name: String,
    pub mime_type: String,
    pub is_image: bool,
}