use std::{net::SocketAddr, str::FromStr};

use crate::{
    app_router::dump_receiver::DUMP_REQUEST,
    common::{app_error::AppError, app_state::AppState},
};
use app::model::restclient::{
    rest_client_request::RestClientRequest,
    rest_client_response::{RestClientResponse, RestClientResponseBody},
};
use axum::{
    Json, body::{self, Body}, extract::{ConnectInfo, Request, State}, middleware::Next, response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use reqwest::{Client, RequestBuilder, Url};
use serde_json::json;
use tracing::{debug, info};

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

            if let Some(content_length) = response.content_length() {
                if content_length > app_state.max_content_length {
                    return Err(AppError::system_error("The response size is too large."));
                }
            }

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
                }));
            }

            let content_disposition = response
                .headers()
                .get(header::CONTENT_DISPOSITION)
                .and_then(|val| val.to_str().ok());

            if let Some(filename) = content_disposition.and_then(|cd| {
                cd.split(';')
                    .find(|part| part.trim().starts_with("filename"))
                    .and_then(|part| part.split('=').nth(1))
                    .map(|name| name.trim().trim_matches('"').to_string())
            }) {
                return Ok(Json(RestClientResponse {
                    status_code,
                    headers,
                    body: RestClientResponseBody::Attachment(filename),
                    request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
                    error: None,
                }));
            }

            let body = response.text().await.map_err(AppError::system_error)?;

            Ok(Json(RestClientResponse {
                status_code,
                headers,
                body: RestClientResponseBody::Text(body),
                request_raw: String::from_utf8_lossy(&DUMP_REQUEST.lock().await).to_string(),
                error: None,
            }))
        }
        Err(err) => Ok(Json(RestClientResponse {
            status_code: 0,
            headers: Vec::new(),
            body: RestClientResponseBody::None,
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

pub async fn rest_client_attachment_download_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RestClientRequest>,
) -> Result<Response<Body>, AppError> {
    build_request(&request, None)?.send().await.map_err(AppError::system_error)?;

    let response = build_request(&request, None)?.send().await.map_err(AppError::system_error)?;

    if let Some(content_length) = response.content_length() {
        if content_length > app_state.max_content_length {
            return Err(AppError::system_error("The response size is too large."));
        }
    }

    let response_status = response.status();

    let body = Body::from_stream(response.bytes_stream());
    Ok((response_status, body).into_response())
}

pub async fn rest_client_remote_proxy(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let cookie_jar = CookieJar::from_headers(req.headers());
    if is_proxy_allow(&req, &app_state, addr)
        && let Some(cookie) = cookie_jar.get("rc_base_url")
    {
        let rc_base_url = cookie.value();

        let uri = req.uri();
        let query_str = match uri.query() {
            Some(query) => format!("?{}", query),
            None => "".to_owned(),
        };

        let rc_base_url = rc_base_url.trim_end_matches("/");
        let mut path = uri.path().trim_end_matches("/");
        if path.starts_with('/') {
            path = &path[1..];
        }

        let url = format!("{}/{}{}", rc_base_url, path, query_str);
        let method = req.method().clone();

        let headers = req.headers().clone();

        let body_stream = req.into_body();
        let body_bytes =
            body::to_bytes(body_stream, usize::MAX).await.map_err(AppError::system_error)?;

        let mut request = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(AppError::system_error)?
            .request(method, url)
            .body(reqwest::Body::from(body_bytes));

        for hv in headers {
            if let Some(name) = hv.0
                && name != "host"
            {
                request = request.header(name, hv.1);
            }
        }

        let response = request.send().await.map_err(AppError::system_error)?;

        if let Some(content_length) = response.content_length() {
            if content_length > app_state.max_content_length {
                return Err(AppError::system_error("The response size is too large."));
            }
        }

        let response_status = response.status();
        let headers = response.headers().clone();
        let body = Body::from_stream(response.bytes_stream());

        return Ok((response_status, headers, body).into_response());
    }

    let response = next.run(req).await;
    return Ok(response);
}

#[axum::debug_handler]
pub async fn rest_client_proxy_allow(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(json! {is_proxy_allow(&req, &app_state, addr) }).into_response())
}

fn is_proxy_allow(req: &Request, app_state: &AppState, client_addr: SocketAddr) -> bool {
    let ip = req
        .headers()
        .get(http::header::FORWARDED)
        .and_then(|val| val.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or(client_addr.ip().to_string());

    debug!("IP:{} allowed={:?}", ip, app_state.rest_client_proxy_allow_ips);

    app_state.rest_client_proxy_allow_ips.contains(&ip)
}
