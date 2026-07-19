use app::common::app_error::AppError;
use axum::{
    body::Body,
    extract::RawQuery,
    http::{StatusCode, header},
    response::IntoResponse,
};
use futures_util::StreamExt;

use crate::common::dev_utils::parse_query_params;

pub async fn format_json_handler(
    RawQuery(query): RawQuery,
    body: Body,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let ident: usize =
        params.get("ident").unwrap_or(&"4").parse().map_err(AppError::system_error)?;

    let body = process_json_data(body, ident).await;

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

#[cfg(not(target_os = "windows"))]
async fn process_json_data(body: Body, ident: usize) -> Body {
    let mut formatter = app::common::json_formatter::JsonFormatter::new(ident);
    let output_stream = body.into_data_stream().map(move |result| match result {
        Ok(data) => Ok(formatter.format_bytes(data)),
        Err(err) => Err(std::io::Error::other(err)),
    });

    Body::from_stream(output_stream)
}

#[cfg(target_os = "windows")]
async fn process_json_data(body: Body, ident: usize) -> Body {
    use app::common::json_formatter::JsonFormatter;

    let request_body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    let mut formatter = JsonFormatter::new(ident);
    let output_stream = tokio_util::io::ReaderStream::new(std::io::Cursor::new(request_body_bytes))
        .map(move |result| match result {
            Ok(data) => Ok(formatter.format_bytes(data)),
            Err(err) => Err(std::io::Error::other(err)),
        });

    Body::from_stream(output_stream)
}
