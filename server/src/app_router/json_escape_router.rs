use crate::common::app_error::AppError;

pub async fn unescape_json_handler(encoded: String) -> Result<String, AppError> {
    unescaper::unescape(&encoded).map_err(AppError::system_error)
}
