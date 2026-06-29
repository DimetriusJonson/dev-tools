use axum::{
    body::Body,
    http::{StatusCode, header},
    response::IntoResponse,
};
use bytes::Bytes;
use futures_util::StreamExt;
use json_escape::{escape_str, stream::UnescapeStream, token::UnescapedToken};

use crate::common::app_error::AppError;

pub async fn escape_json_handler(body: Body) -> Result<impl IntoResponse, AppError> {
    let request_body_stream = body.into_data_stream().map(|result| match result {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            let escaped_parts = escape_str(&text);

            let mut escaped_str = String::new();
            for part in escaped_parts {
                escaped_str.push_str(part);
            }
            Ok(Bytes::from(escaped_str))
        }
        Err(err) => Err(std::io::Error::other(err)),
    });
    let body = Body::from_stream(request_body_stream);

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

pub async fn unescape_json_handler(body: Body) -> Result<impl IntoResponse, AppError> {
    let mut unescaper = UnescapeStream::new();
    let request_body_stream = body.into_data_stream().map(move |result| match result {
        Ok(bytes) => match unescaper.try_unescape_next(&bytes) {
            Ok((boundary_char, rest_of_part)) => {
                let mut unescaped_str = String::new();
                if let Some(c) = boundary_char {
                    unescaped_str.push(c);
                }

                for token in rest_of_part {
                    match token {
                        Ok(UnescapedToken::Literal(literal)) => {
                            unescaped_str.push_str(&String::from_utf8_lossy(&literal).to_string())
                        }
                        Ok(UnescapedToken::Unescaped(ch)) => unescaped_str.push(ch),
                        Err(err) => return Err(std::io::Error::other(err.to_string())),
                    }
                }
                Ok(Bytes::from(unescaped_str))
            }
            Err(err) => return Err(std::io::Error::other(err.to_string())),
        },
        Err(err) => Err(std::io::Error::other(err)),
    });
    let body = Body::from_stream(request_body_stream);

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}
