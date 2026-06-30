use std::io::{self, Cursor};

use async_stream::stream;
use axum::{
    body::Body,
    extract::RawQuery,
    http::{StatusCode, header},
    response::IntoResponse,
};
use bytes::Bytes;
use futures_util::Stream;
use tokio::io::BufReader;
use tokio::io::{AsyncBufRead, AsyncReadExt};

use crate::common::{app_error::AppError, dev_utils::parse_query_params};

const BUFF_SIZE: usize = 1024;

pub async fn format_json_handler(
    RawQuery(query): RawQuery,
    body: Body,
) -> Result<impl IntoResponse, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let ident: usize =
        params.get("ident").unwrap_or(&"4").parse().map_err(AppError::system_error)?;

    let body = Body::from_stream(create_stream(body, ident).await);

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

#[cfg(not(target_os = "windows"))]
async fn build_reader(body: Body) -> impl AsyncBufRead + Unpin {
    use futures_util::StreamExt;

    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));
    BufReader::new(tokio_util::io::StreamReader::new(request_body_stream))
}

#[cfg(target_os = "windows")]
async fn build_reader(body: Body) -> impl AsyncBufRead + Unpin {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    BufReader::new(Cursor::new(bytes))
}

async fn create_stream(
    body: Body,
    ident: usize,
) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let mut reader = build_reader(body).await;

    stream! {
        let mut escaped = false;
        let mut in_string = false;
        let mut indent_level = 0usize;
        let mut newline_requested = false; // invalidated if next character is ] or }

        let mut formatted_bytes = Vec::<u8>::with_capacity(BUFF_SIZE);
        loop {
            let mut char: u8 = 0;
            match reader.read_u8().await {
                Ok(b) => char = b,
                Err(err) => {
                    match err.kind() {
                        io::ErrorKind::UnexpectedEof => break,
                        _ => yield Err(err)?,
                    }
                },
            }

            if char == 0 {
                break;
            }

            if in_string {
                let mut escape_here = false;
                match char {
                    b'"' if !escaped => {
                            in_string = false;
                    }
                    b'\\' if !escaped => {
                            escape_here = true;
                    }
                    _ => {}
                }
                formatted_bytes.push(char);
                escaped = escape_here;
            } else {
                let mut auto_push = true;
                let mut request_newline = false;
                let old_level = indent_level;

                match char {
                    b'"' => in_string = true,
                    b' ' | b'\n' | b'\r' | b'\t' => continue,
                    b'[' => {
                        indent_level += 1;
                        request_newline = true;
                    }
                    b'{' => {
                        indent_level += 1;
                        request_newline = true;
                    }
                    b'}' | b']' => {
                        indent_level = indent_level.saturating_sub(1);
                        if !newline_requested {
                            // see comment below about newline_requested
                            formatted_bytes.push(b'\n');
                            write_ident(&mut formatted_bytes, indent_level, ident);
                        }
                    }
                    b':' => {
                        auto_push = false;
                        formatted_bytes.push(char);
                        formatted_bytes.push(b' ');
                    }
                    b',' => {
                        request_newline = true;
                    }
                    _ => {}
                }
                if newline_requested && char != b']' && char != b'}' {
                    // newline only happens after { [ and ,
                    // this means we can safely assume that it being followed up by } or ]
                    // means an empty object/array
                    formatted_bytes.push(b'\n');
                    write_ident(&mut formatted_bytes, old_level, ident);
                }

                if auto_push {
                    formatted_bytes.push(char);
                }

                newline_requested = request_newline;

                if formatted_bytes.len() > BUFF_SIZE {
                    let chunk = Bytes::copy_from_slice(&formatted_bytes);
                    formatted_bytes.clear();

                    yield Ok(chunk);
                }
            }
        }

        // trailing newline
        formatted_bytes.push(b'\n');

        let chunk = Bytes::copy_from_slice(&formatted_bytes);
        formatted_bytes.clear();

        yield Ok(chunk);
    }
}

fn write_ident(write_buffer: &mut Vec<u8>, level: usize, ident: usize) -> () {
    for _ in 0..level * ident {
        write_buffer.push(b' ');
    }
}
