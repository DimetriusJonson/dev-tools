use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Getters, CopyGetters)]
pub struct ShareFileInfoDto {
    #[get = "pub"]
    file_name: String,
    #[get = "pub"]
    mime_type: String,
    #[get_copy = "pub"]
    is_image: bool,
    #[get_copy = "pub"]
    file_size: i32,
}

impl ShareFileInfoDto {
    pub fn new(file_name: String, mime_type: String, is_image: bool, file_size: i32) -> Self {
        Self { file_name, mime_type, is_image, file_size }
    }
}
