use std::{fmt::Display, num::ParseIntError, str::ParseBoolError};

#[derive(Debug)]
pub enum AppError {
    SystemError(String),
    BadRequest(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::SystemError(msg) => write!(f, "System error: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<ParseIntError> for AppError {
    fn from(value: ParseIntError) -> Self {
        Self::BadRequest(value.to_string())
    }
}

impl From<ParseBoolError> for AppError {
    fn from(value: ParseBoolError) -> Self {
        Self::BadRequest(value.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<http::Error> for AppError {
    fn from(value: http::Error) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<InvalidHeaderValue> for AppError {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<InvalidMethod> for AppError {
    fn from(value: InvalidMethod) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<InvalidHeaderName> for AppError {
    fn from(value: InvalidHeaderName) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(value: url::ParseError) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<axum::Error> for AppError {
    fn from(value: axum::Error) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<ToStrError> for AppError {
    fn from(value: ToStrError) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<ImageError> for AppError {
    fn from(value: ImageError) -> Self {
        Self::SystemError(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::SystemError(value.to_string())
    }
}

#[cfg(feature = "db")]
impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::SystemError(value.to_string())
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use http::{
    header::{InvalidHeaderName, InvalidHeaderValue, ToStrError}, method::InvalidMethod,
};
use image::ImageError;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::SystemError(msg) => {
                tracing::error!("System error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, error_message).into_response()
    }
}
