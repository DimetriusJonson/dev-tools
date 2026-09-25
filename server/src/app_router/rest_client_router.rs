use std::str::FromStr;

use crate::{
    app_router::dump_receiver::DUMP_REQUEST,
    common::{app_error::AppError, app_state::AppState},
};
use axum::{
    Json,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use model::restclient::{
    rest_client_request::RestClientRequest,
    rest_client_response::{RestClientResponse, RestClientResponseBody},
};
use reqwest::{Client, RequestBuilder, Url};
use url::ParseError;

#[axum::debug_handler]
pub async fn rest_client_send_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RestClientRequest>,
) -> Result<Json<RestClientResponse>, AppError> {
    build_request(&request, Some(app_state.dump_port))?.send().await?;

    match build_request(&request, None)?.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let content_length = response.content_length();

            let headers: Vec<(String, String)> = response
                .headers()
                .iter()
                .filter_map(|(key, value)| {
                    let key_str = key.as_str().to_string();
                    let val_str = value.to_str().ok()?.to_string();
                    Some((key_str, val_str))
                })
                .collect();

            if response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|val| val.to_str().ok())
                .map(|content_type| content_type.starts_with("image/"))
                .unwrap_or(false)
            {
                return Ok(Json(RestClientResponse {
                    status_code,
                    headers,
                    body: RestClientResponseBody::Image,
                    request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
                    error: None,
                    size: content_length,
                }));
            }

            if let Some(file_name) = resolve_content_disposition(&request, &response).await? {
                return Ok(Json(RestClientResponse {
                    status_code,
                    headers,
                    body: RestClientResponseBody::Attachment(file_name),
                    request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
                    error: None,
                    size: content_length,
                }));
            }

            if let Some(content_length) = content_length
                && content_length > app_state.max_content_length
            {
                return Err(AppError::BadRequest(format!(
                    "Rest client send: The response size is too large (max={}, actual={}).",
                    app_state.max_content_length, content_length
                )));
            }

            let body = response.text().await?;
            let body_size = body.len() as u64;

            Ok(Json(RestClientResponse {
                status_code,
                headers,
                body: RestClientResponseBody::Text(body),
                request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
                error: None,
                size: Some(content_length.unwrap_or(body_size)),
            }))
        }
        Err(err) => Ok(Json(RestClientResponse {
            status_code: 0,
            headers: Vec::new(),
            body: RestClientResponseBody::None,
            request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
            error: Some(err.to_string()),
            size: None,
        })),
    }
}

async fn resolve_content_disposition(
    request: &RestClientRequest,
    response: &reqwest::Response,
) -> Result<Option<String>, AppError> {
    let content_disposition =
        response.headers().get(header::CONTENT_DISPOSITION).and_then(|val| val.to_str().ok());

    if let Some(content_disposition) = content_disposition {
        if let Some(file_name) = content_disposition
            .split(';')
            .find(|part| part.trim().starts_with("filename"))
            .and_then(|part| part.split('=').nth(1))
            .map(|name| name.trim().trim_matches('"').to_owned())
        {
            return Ok(Some(file_name));
        } else {
            return Ok(Some(
                Url::parse(&request.url)?
                    .path_segments()
                    .and_then(|ps| ps.last())
                    .and_then(|s| Some(s.to_owned()))
                    .unwrap_or("unknown.file".to_owned()),
            ));
        };
    }
    Ok(None)
}

fn build_request(
    request: &RestClientRequest,
    dump_port: Option<u16>,
) -> Result<RequestBuilder, AppError> {
    let method = Method::from_str(&request.method)?;
    let mut headers = HeaderMap::new();
    for (name, value) in &request.headers {
        headers.insert(HeaderName::from_str(name)?, HeaderValue::from_str(value)?);
    }

    let url;
    if let Some(dump_port) = dump_port {
        let (new_url, old_host) = build_to_dump_receiver_url(request.url.to_owned(), dump_port)?;
        url = new_url;
        if !headers.contains_key(http::header::HOST) {
            headers.insert(http::header::HOST, HeaderValue::from_str(&old_host)?);
        }
    } else {
        url = request.url.to_owned();
    };

    Ok(Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?
        .request(method, url)
        .headers(headers)
        .body(reqwest::Body::from(request.body.to_owned())))
}

fn build_to_dump_receiver_url(
    url_str: String,
    dump_port: u16,
) -> Result<(String, String), ParseError> {
    let mut url = Url::parse(&url_str)?;
    let old_port = match url.port() {
        Some(port) => format!(":{}", port),
        None => "".to_owned(),
    };
    let old_host = format!("{}{}", url.host_str().unwrap_or_default().to_owned(), old_port);

    url.set_scheme("http").expect("Cant set url http scheme");
    url.set_host(Some("127.0.0.1")).expect("Cant set url 127.0.0.1 host");
    url.set_port(Some(dump_port)).unwrap_or_else(|_| panic!("Cant set url port {}", dump_port));

    Ok((url.to_string(), old_host))
}

pub async fn rest_client_attachment_download_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RestClientRequest>,
) -> Result<Response<Body>, AppError> {
    build_request(&request, None)?.send().await?;

    let response = build_request(&request, None)?.send().await?;

    if let Some(content_length) = response.content_length()
        && content_length > app_state.max_content_length
    {
        return Err(AppError::BadRequest(format!(
            "Attachment dowload: The response size is too large (max={}, actual={}).",
            app_state.max_content_length, content_length
        )));
    }

    let response_status = response.status();

    let body = Body::from_stream(response.bytes_stream());
    Ok((response_status, body).into_response())
}
