use std::{
    collections::HashMap,
    net::SocketAddr,
    str::FromStr,
    sync::{LazyLock, RwLock},
};

use crate::{
    app_router::dump_receiver::DUMP_REQUEST,
    common::{app_error::AppError, app_state::AppState, dev_utils::parse_query_params},
};
use app::model::restclient::{
    rest_client_request::RestClientRequest,
    rest_client_response::{RestClientResponse, RestClientResponseBody},
};
use axum::{
    Json,
    body::{self, Body},
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use reqwest::{Client, RequestBuilder, Url};
use serde_json::json;
use tracing::debug;

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

            if let Some(content_length) = content_length
                && content_length > app_state.max_content_length
            {
                return Err(AppError::system_error("The response size is too large."));
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
                    size: content_length,
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
                    size: content_length,
                }));
            }

            let body = response.text().await.map_err(AppError::system_error)?;
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

    if let Some(content_length) = response.content_length()
        && content_length > app_state.max_content_length
    {
        return Err(AppError::system_error("The response size is too large."));
    }

    let response_status = response.status();

    let body = Body::from_stream(response.bytes_stream());
    Ok((response_status, body).into_response())
}

static PROXY_CACHE: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn build_proxy_cache_key(base_url: &str, path: &str, query: Option<&str>) -> String {
    let mut query_str = query
        .map(|q| {
            q.split('&')
                .filter(|qv| !qv.starts_with("rc_base_url="))
                .collect::<Vec<&str>>()
                .join("&")
        })
        .unwrap_or_default();
    if !query_str.is_empty() {
        query_str.insert(0, '?');
    }

    format!("{}:{}{}", base_url, path, query_str)
}

fn get_proxy_cached_value(base_url: &str, path: &str, query: Option<&str>) -> Option<String> {
    let key = build_proxy_cache_key(base_url, path, query);

    //info!("get {}", key);
    let cache = PROXY_CACHE.read().unwrap();
    cache.get(&key).cloned()
}

fn set_proxy_cached_value(base_url: &str, path: &str, query: Option<&str>, value: String) {
    let key = build_proxy_cache_key(base_url, path, query);

    //info!("*** set {}", key);

    let mut cache = PROXY_CACHE.write().unwrap();
    cache.insert(key, value);
}

pub async fn rest_client_html_previewer_middleware(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    if !req.uri().path().starts_with("/rest_client")
        && !req.uri().path().starts_with("/pkg/")
        && is_proxy_allow(&req, &app_state, addr)
    {
        let cookie_jar = CookieJar::from_headers(req.headers());
        if let Some(cookie) = cookie_jar.get("rc_base_url") {
            let rc_base_url = cookie.value().to_owned();

            let url_param = if let Some(Some(url)) = req.uri().query().map(|query_str| {
                let params = parse_query_params(query_str);
                params.get("rc_base_url").map(|v| v.to_owned())
            }) {
                Some(urlencoding::decode(url).map_err(AppError::system_error)?.into_owned())
            } else {
                None
            };

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

            let mut base_url = Url::parse(rc_base_url).map_err(AppError::system_error)?;

            let headers = req.headers().clone();
            let referer = headers
                .get(header::REFERER)
                .map(|hv| {
                    hv.to_str().ok().map(|referer| {
                        urlencoding::decode(referer).ok().map(|referer| Url::parse(&referer).ok())
                    })
                })
                .unwrap_or_default()
                .unwrap_or_default()
                .unwrap_or_default();

            if let Some(referer) = &referer
                && let Some(parent_base_url) =
                    get_proxy_cached_value(rc_base_url, referer.path(), referer.query())
                && let Ok(parent_base_url) = Url::parse(&parent_base_url)
            {
                base_url = parent_base_url;
            }

            let url = match &url_param {
                Some(url_param) => url_param.to_owned(),
                None => format!(
                    "{}://{}/{}{}",
                    base_url.scheme(),
                    base_url.host_str().unwrap_or_default(),
                    path,
                    query_str
                ),
            };
            let method = req.method().clone();

            let req_uri = &req.uri().clone();

            let body_stream = req.into_body();
            let body_bytes =
                body::to_bytes(body_stream, usize::MAX).await.map_err(AppError::system_error)?;

            //info!("**** request {}", url);
            let mut request = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(AppError::system_error)?
                .request(method, url.to_owned())
                .body(reqwest::Body::from(body_bytes));

            for hv in headers {
                if let Some(name) = hv.0
                    && name.as_str().to_lowercase() != "host"
                    && name.as_str().to_lowercase() != "referer"
                {
                    request = request.header(name, hv.1);
                }
            }

            if let Some(referer) = &referer {
                let referer = if referer.path() == "/rest_client" { &base_url } else { referer };
                let referer_value = format!(
                    "{}://{}{}{}",
                    base_url.scheme(),
                    base_url.host_str().unwrap_or_default(),
                    referer.path(),
                    referer.query().map(|query| format!("?{}", query)).unwrap_or_default()
                );
                request = request.header(header::REFERER, referer_value);
            }

            let response = request.send().await.map_err(AppError::system_error)?;

            if let Some(content_length) = response.content_length()
                && content_length > app_state.max_content_length
            {
                return Err(AppError::system_error("The response size is too large."));
            }

            if let Some(url_param) = url_param {
                set_proxy_cached_value(
                    rc_base_url,
                    req_uri.path(),
                    req_uri.query(),
                    url_param.to_owned(),
                );
            }

            let response_status = response.status();
            let headers = response.headers().clone();
            let body = Body::from_stream(response.bytes_stream());

            return Ok((response_status, headers, body).into_response());
        }
    }

    let response = next.run(req).await;
    Ok(response)
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
    let forwarded_for = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|val| val.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or("".to_owned());
    let real_ip = client_addr.ip().to_string();

    debug!(
        "forwarded_for={forwarded_for} real_ip={real_ip} allowed={:?}",
        app_state.rest_client_proxy_allow_ips
    );

    let client_ip = if !forwarded_for.is_empty() {
        forwarded_for.split(',').next().unwrap_or(&real_ip).trim().to_owned()
    } else {
        real_ip
    };

    app_state.rest_client_proxy_allow_ips.contains(&client_ip)
}
