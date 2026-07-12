use std::fmt::Display;

#[derive(Debug)]
pub enum AppError {
    SystemError(String),
}

impl AppError {
    pub fn system_error(msg: impl ToString) -> Self {
        AppError::SystemError(msg.to_string())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

#[cfg(feature = "ssr")]
pub mod srr {
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
    };

    use crate::common::app_error::AppError;
    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            let (status, error_message) = match self {
                AppError::SystemError(msg) => {
                    log::error!("System error: {}", msg.to_string());
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
                }
            };

            (status, error_message).into_response()
        }
    }
}
