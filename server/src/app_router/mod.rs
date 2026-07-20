use app::common::app_error::AppError;
use axum::{body::Body, extract::Request, response::{IntoResponse, Response}};
use http::HeaderValue;
use reqwest::Client;

pub mod build_app_router;
pub mod json_format_router;
pub mod share_file_router;
pub mod share_local_file_router;
pub mod xml_format_router;
#[cfg(feature = "rest_client")]
pub mod rest_client_router;

pub async fn proxy_request_to_remote(
    remote_server_url: String,
    request: Request,
) -> Result<Response<Body>, AppError> {
    let target_url = format!(
        "{}{}",
        remote_server_url,
        request.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("")
    );

    let method = request.method().clone();
    let headers = request.headers().clone();

    let req_body_stream = request.into_body().into_data_stream();
    let reqwest_body = reqwest::Body::wrap_stream(req_body_stream);

    let mut upstream_req = Client::new().request(method, &target_url).body(reqwest_body);

    for hv in headers {
        if let Some(name) = hv.0 && name != "host" {
            upstream_req = upstream_req.header(name, hv.1);
        }
    }

    let upstream_response = upstream_req.send().await.map_err(AppError::system_error)?;

    let response_status = upstream_response.status();
    let response_headers = upstream_response.headers().clone();

    let body = Body::from_stream(upstream_response.bytes_stream());

    let mut response = (response_status, body).into_response();
    *response.headers_mut() = response_headers;
    response
        .headers_mut()
        .insert("remote-server-url", HeaderValue::from_str(&remote_server_url).unwrap());

    Ok(response)
}

