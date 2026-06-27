use urlencoding::{decode, encode};

use crate::common::app_error::AppError;

pub async fn encode_url_handler(url: String) -> String {
    encode(&url).to_string()
}
pub async fn decode_url_handler(encoded: String) -> Result<String, AppError> {
    Ok(decode(&encoded).map_err(AppError::system_error)?.to_string())
}
