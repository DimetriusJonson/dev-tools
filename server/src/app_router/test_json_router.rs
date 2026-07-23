use serde::Serialize;

use app::common::app_error::AppError;
use axum::Json;

#[derive(Serialize)]
pub struct TestJsonResult {
    pub text: String,
}

pub async fn test_json_handler() -> Result<Json<TestJsonResult>, AppError> {
    Ok(Json(TestJsonResult { text: "Test message.".to_owned() }))
}
