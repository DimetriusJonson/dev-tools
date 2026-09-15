use std::{net::SocketAddr, str::FromStr};

use crate::{
    app_router::rest_client_proxy_cache::{get_proxy_cached_value, set_proxy_cached_value},
    common::{
        app_error::AppError,
        app_state::AppState,
        dev_utils::{extract_uri_query_params, resolve_request_ip},
        html_previewer::{add_preview_scripts, replace_absolute_links},
    },
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
use model::{
    constants::{RC_BASE_URL_COOKIE_NAME, RC_REQ_DATA_PARAM_NAME, RC_SRC_URL_PARAM_NAME},
    restclient::rest_client_request::RestClientRequest,
};
use reqwest::{Client, Url};
use serde_json::json;

pub async fn rest_client_proxy_middleware(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    routes_paths: Vec<&str>,
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
        .unwrap_or_default()
        .unwrap_or_default();

    if (!routes_paths.contains(&req.uri().path())
        || (referer.is_some() && !routes_paths.contains(&referer.to_owned().unwrap().path())))
        && is_proxy_allow(&req, &app_state, addr)
    {
        let cookie_jar = CookieJar::from_headers(req.headers());
        if let Some(cookie) = cookie_jar.get(RC_BASE_URL_COOKIE_NAME) {
            let rc_base_url = cookie.value().trim_end_matches("/");

            let url_param = extract_uri_query_params(req.uri())
                .get(RC_SRC_URL_PARAM_NAME)
                .map(|url| urlencoding::decode(url).ok().map(|url| url.to_string()))
                .unwrap_or(None);

            let request_data = match extract_uri_query_params(req.uri()).get(RC_REQ_DATA_PARAM_NAME)
            {
                Some(value) => Some(serde_json::from_str::<RestClientRequest>(
                    &urlencoding::decode(value)?.to_string(),
                )?),
                None => None,
            };

            //debug!("url_param={:?} cached_request={:?}", url_param, cached_request);

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
                Url::parse(rc_base_url)?
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

            let mut reqwest_headers = match &request_data {
                Some(request_data) => {
                    let mut headers = HeaderMap::new();
                    for h in &request_data.headers {
                        headers.append(HeaderName::from_str(&h.0)?, HeaderValue::from_str(&h.1)?);
                    }
                    headers
                }
                None => req.headers().clone(),
            };

            reqwest_headers.remove(header::HOST);
            reqwest_headers.remove(header::REFERER);
            reqwest_headers.remove(header::ACCEPT_ENCODING);

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
                    .parse()?,
                );
            }

            if reqwest_headers.get(header::ORIGIN).is_some() {
                reqwest_headers.remove(header::ORIGIN);
                if let Ok(header_value) = base_url.origin().ascii_serialization().parse() {
                    reqwest_headers.append(header::ORIGIN, header_value);
                }
            }

            remove_base_cookie(&mut reqwest_headers);

            let reqwest_method = match &request_data {
                Some(request_data) => Method::from_str(&request_data.method)?,
                None => req.method().to_owned(),
            };

            //debug!("reqwest {} {} \n {:?}", reqwest_method, url, reqwest_headers);
            let request = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?
                .request(reqwest_method, &url)
                .headers(reqwest_headers)
                .body({
                    match &request_data {
                        Some(request_data) => reqwest::Body::from(request_data.body.to_owned()),
                        None => {
                            let body_stream = req.into_body();
                            reqwest::Body::from(body::to_bytes(body_stream, usize::MAX).await?)
                        }
                    }
                });

            let response = request.send().await?;
            //debug!("reqwest status {} for {}", response.status(), url);

            if let Some(content_length) = response.content_length()
                && content_length > app_state.max_content_length
            {
                return Err(AppError::BadRequest("The response size is too large.".to_owned()));
            }

            let response_status = response.status();

            let mut headers = response.headers().clone();
            replace_cookies_domain(&mut headers);

            let body = build_response_body(response, referer, rc_base_url, &mut headers).await?;

            return Ok((response_status, headers, body).into_response());
        }
    }

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!("{}=''; max-age=0; path=/", RC_BASE_URL_COOKIE_NAME))
            .expect("Cant create header value"),
    );
    Ok(response)
}

async fn build_response_body(
    response: reqwest::Response,
    referer: Option<Url>,
    rc_base_url: &str,
    headers: &mut HeaderMap,
) -> Result<Body, AppError> {
    if let Some(content_type) = response.headers().get(http::header::CONTENT_TYPE)
        && let Ok(content_type) = content_type.to_str()
        && content_type.contains("text/html")
        && let Some(referer) = referer
    {
        let mut html = response.text().await?;

        add_preview_scripts(&mut html);

        replace_absolute_links(&mut html, rc_base_url, referer.as_str());

        let body = Body::from(html);
        headers.remove(header::CONTENT_LENGTH);
        headers.remove(header::CONTENT_ENCODING);
        headers.remove(header::TRANSFER_ENCODING);
        Ok(body)
    } else {
        Ok(Body::from_stream(response.bytes_stream()))
    }
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
    let client_ip = resolve_request_ip(req.headers(), client_addr);
    app_state.rest_client_proxy_allow_ips.contains(&client_ip)
}

fn remove_base_cookie(headers: &mut HeaderMap) {
    let cookies = headers
        .get_all(header::COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    let new_cookies: String = cookies
        .filter(|c| c.name() != RC_BASE_URL_COOKIE_NAME)
        .map(|c| c.encoded().to_string())
        .collect::<Vec<String>>()
        .join(";");

    headers.remove(header::COOKIE);
    if !new_cookies.is_empty()
        && let Ok(header_value) = new_cookies.parse::<HeaderValue>()
    {
        headers.append(header::COOKIE, header_value);
    }
}

fn replace_cookies_domain(headers: &mut HeaderMap) {
    let set_cookies = headers
        .get_all(header::SET_COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    let mut new_set_cookies = Vec::new();
    for cookie in set_cookies {
        let mut new_cookie = cookie.clone();
        if new_cookie.domain().is_some() {
            new_cookie.set_domain("");
        }
        new_set_cookies.push(new_cookie);
    }

    headers.remove(header::SET_COOKIE);
    for cookie in new_set_cookies.iter() {
        if let Ok(header_value) = cookie.encoded().to_string().parse::<HeaderValue>() {
            headers.append(header::SET_COOKIE, header_value);
        }
    }
}
