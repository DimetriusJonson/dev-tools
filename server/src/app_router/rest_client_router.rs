use std::str::FromStr;

use app::{
    common::app_error::AppError,
    model::{rest_client_request::RestClientRequest, rest_client_response::RestClientResponse},
};
use axum::Json;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use reqwest::Client;

pub async fn rest_client_send_handler(
    Json(request): Json<RestClientRequest>,
) -> Result<Json<RestClientResponse>, AppError> {
    let method = Method::from_str(&request.method).map_err(AppError::system_error)?;
    let mut headers = HeaderMap::new();
    for (name, value) in &request.headers {
        headers.insert(
            HeaderName::from_str(name).map_err(AppError::system_error)?,
            HeaderValue::from_str(value).map_err(AppError::system_error)?,
        );
    }

    let rb = Client::new().request(method, request.url).body(reqwest::Body::from(request.body));

    let response = rb.send().await.map_err(AppError::system_error)?;

    let status_code = response.status().as_u16();

    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .filter_map(|(key, value)| {
            let key_str = key.as_str().to_string();
            let val_str = value.to_str().ok()?.to_string();
            Some((key_str, val_str))
        })
        .collect();
    let body = response.text().await.map_err(AppError::system_error)?;

    Ok(Json(RestClientResponse { status_code, headers, body }))
}
