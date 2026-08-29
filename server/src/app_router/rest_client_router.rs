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
use axum_extra::extract::{CookieJar, cookie::Cookie};
use http::{HeaderMap, HeaderName, HeaderValue, Method, Uri, header};
use reqwest::{Client, RequestBuilder, Url};
use serde_json::json;

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

    //info!("get cache {}", key);
    let cache = PROXY_CACHE.read().unwrap();
    cache.get(&key).cloned()
}

fn set_proxy_cached_value(base_url: &str, path: &str, query: Option<&str>, value: String) {
    let key = build_proxy_cache_key(base_url, path, query);

    //info!("*** set cache {}", key);

    let mut cache = PROXY_CACHE.write().unwrap();
    cache.insert(key, value);
}

pub async fn rest_client_html_previewer_middleware(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    routes_paths: Vec<String>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let referer_raw =
        req.headers().get(header::REFERER).map(|hv| hv.to_str().ok()).unwrap_or_default();
    let referer_uri = referer_raw.map(|referer| Uri::from_str(referer).ok()).unwrap_or_default();

    let referer = referer_raw
        .map(|referer_raw| {
            urlencoding::decode(referer_raw).ok().map(|referer| Url::parse(&referer).ok())
        })
        .unwrap_or_default().unwrap_or_default();

    if (!routes_paths.contains(&req.uri().path().to_owned())
        || (referer.is_some()
            && !routes_paths.contains(&referer.to_owned().unwrap().path().to_owned())))
        && is_proxy_allow(&req, &app_state, addr)
    {
        let cookie_jar = CookieJar::from_headers(req.headers());
        if let Some(cookie) = cookie_jar.get("rc_base_url") {
            let rc_base_url = cookie.value().trim_end_matches("/");

            let url_param = req
                .uri()
                .query()
                .map(|query_str| {
                    parse_query_params(query_str)
                        .get("rc_src_url")
                        .map(|url| urlencoding::decode(url).ok().map(|url| url.to_string()))
                })
                .unwrap_or(None)
                .unwrap_or(None);

            let mut path = req.uri().path();
            if path.starts_with('/') {
                path = &path[1..];
            }

            let base_url = if let Some(referer) = &referer_uri
                && let Some(parent_base_url) =
                    get_proxy_cached_value(rc_base_url, referer.path(), referer.query())
                && let Ok(parent_base_url) = Url::parse(&parent_base_url)
            {
                parent_base_url
            } else {
                Url::parse(rc_base_url).map_err(AppError::system_error)?
            };

            let url = match &url_param {
                Some(url_param) => url_param.to_owned(),
                None => format!(
                    "{}://{}/{}{}",
                    base_url.scheme(),
                    base_url.host_str().unwrap_or_default(),
                    path,
                    req.uri().query().map(|query| format!("?{}", query)).unwrap_or_default()
                ),
            };

            if let Some(url_param) = url_param {
                set_proxy_cached_value(
                    rc_base_url,
                    req.uri().path(),
                    req.uri().query(),
                    url_param.to_owned(),
                );
            }

            let mut reqwest_headers = req.headers().clone();
            reqwest_headers.remove(header::HOST);
            reqwest_headers.remove(header::REFERER);

            if let Some(referer) = &referer {
                let referer = if referer.path() == "/rest_client" { &base_url } else { referer };
                reqwest_headers.append(
                    header::REFERER,
                    format!(
                        "{}://{}{}{}",
                        base_url.scheme(),
                        base_url.host_str().unwrap_or_default(),
                        referer.path(),
                        referer.query().map(|query| format!("?{}", query)).unwrap_or_default()
                    )
                    .parse()
                    .map_err(AppError::system_error)?,
                );
            }

            //info!("{} {}", req.method(), url);
            //info!("headers: {:?}", reqwest_headers);

            let request = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(AppError::system_error)?
                .request(req.method().to_owned(), &url)
                .headers(reqwest_headers)
                .body({
                    let body_stream = req.into_body();
                    reqwest::Body::from(
                        body::to_bytes(body_stream, usize::MAX)
                            .await
                            .map_err(AppError::system_error)?,
                    )
                });

            let response = request.send().await.map_err(AppError::system_error)?;
            //            info!("response {} for {}", response.status(), url);

            if let Some(content_length) = response.content_length()
                && content_length > app_state.max_content_length
            {
                return Err(AppError::system_error("The response size is too large."));
            }

            let response_status = response.status();

            let mut headers = response.headers().clone();
            replace_cookies_domain(&mut headers);

            let body = Body::from_stream(response.bytes_stream());

            return Ok((response_status, headers, body).into_response());
        }
    }

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str("rc_base_url=''; max-age=0; path=/").unwrap(),
    );
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

    let client_ip = if !forwarded_for.is_empty() {
        forwarded_for.split(',').next().unwrap_or(&real_ip).trim().to_owned()
    } else {
        real_ip
    };

    app_state.rest_client_proxy_allow_ips.contains(&client_ip)
}

fn replace_cookies_domain(headers: &mut HeaderMap) {
    let set_cookies = headers
        .get_all(header::SET_COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    let mut new_set_cookies = Vec::new();
    for cookie in set_cookies {
        let mut new_cookie = cookie.clone();
        if let Some(_) = new_cookie.domain() {
            new_cookie.set_domain("");
        }
        new_set_cookies.push(new_cookie);
    }

    headers.remove(header::SET_COOKIE);
    for cookie in new_set_cookies.iter() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            headers.append(header::SET_COOKIE, header_value);
        }
    }
}
