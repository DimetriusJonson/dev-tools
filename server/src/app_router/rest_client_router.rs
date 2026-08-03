use std::str::FromStr;

use crate::{
    app_router::dump_receiver::DUMP_REQUEST,
    common::{app_error::AppError, app_state::AppState},
};
use app::model::restclient::{
    rest_client_request::RestClientRequest, rest_client_response::RestClientResponse,
};
use axum::{Json, extract::State};
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use reqwest::{Client, RequestBuilder, Url};

pub async fn rest_client_send_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RestClientRequest>,
) -> Result<Json<RestClientResponse>, AppError> {
    build_request(&request, Some(app_state.dump_port))?
        .send()
        .await
        .map_err(AppError::system_error)?;

    match build_request(&request, None)?.send().await {
        Ok(response) => {
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

            Ok(Json(RestClientResponse {
                status_code,
                headers,
                body,
                request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
                error: None,
            }))
        }
        Err(err) => Ok(Json(RestClientResponse {
            status_code: 0,
            headers: Vec::new(),
            body: "".to_owned(),
            request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
            error: Some(err.to_string()),
        })),
    }
}

fn build_request(
    request: &RestClientRequest,
    dump_port: Option<u16>,
) -> Result<RequestBuilder, AppError> {
    let method = Method::from_str(&request.method).map_err(AppError::system_error)?;
    let mut headers = HeaderMap::new();
    for (name, value) in &request.headers {
        headers.insert(
            HeaderName::from_str(name).map_err(AppError::system_error)?,
            HeaderValue::from_str(value).map_err(AppError::system_error)?,
        );
    }

    let url;
    if let Some(dump_port) = dump_port {
        let (new_url, old_host) = build_to_dump_receiver_url(request.url.to_owned(), dump_port)
            .map_err(AppError::system_error)?;
        url = new_url;
        if !headers.contains_key(http::header::HOST) {
            headers.insert(
                http::header::HOST,
                HeaderValue::from_str(&old_host).map_err(AppError::system_error)?,
            );
        }
    } else {
        url = request.url.to_owned();
    };

    Ok(Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(AppError::system_error)?
        .request(method, url)
        .headers(headers)
        .body(reqwest::Body::from(request.body.to_owned())))
}

fn build_to_dump_receiver_url(
    url_str: String,
    dump_port: u16,
) -> Result<(String, String), AppError> {
    let mut url = Url::parse(&url_str).map_err(AppError::system_error)?;
    let old_port = match url.port() {
        Some(port) => format!(":{}", port),
        None => "".to_owned(),
    };
    let old_host = format!("{}{}", url.host_str().unwrap().to_owned(), old_port);

    url.set_scheme("http").unwrap();
    url.set_host(Some("127.0.0.1")).map_err(AppError::system_error)?;
    url.set_port(Some(dump_port)).unwrap();

    Ok((url.to_string(), old_host))
}
