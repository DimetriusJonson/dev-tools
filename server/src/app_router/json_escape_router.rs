use std::io::Cursor;

use async_stream::try_stream;
use axum::{
    body::Body, http::{StatusCode, header}, response::IntoResponse,
};
use bytes::{Bytes, BytesMut};
use futures_util::{Stream, StreamExt};
use json_escape::{escape_str, stream::UnescapeStream, token::UnescapedToken};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::io::StreamReader;

use crate::common::app_error::AppError;

pub async fn escape_json_handler(body: Body) -> Result<impl IntoResponse, AppError> {
    let body = Body::from_stream(create_escape_stream(body));

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

pub async fn unescape_json_handler(body: Body) -> Result<impl IntoResponse, AppError> {
    let body = Body::from_stream(create_unescape_stream(body));

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

fn create_escape_stream(body: Body) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));

    let mut input_reader = BufReader::new(StreamReader::new(request_body_stream));

    try_stream! {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let mut read_buffer = BytesMut::with_capacity(1024);
        loop {
            let count = input_reader.read_buf(&mut read_buffer).await?;
            if count > 0 {
                let text = String::from_utf8_lossy(&read_buffer);
                let escaped_parts = escape_str(&text);

                for part in escaped_parts {
                     writer.write_all(part.as_bytes()).await?;
                }
            } else {
                break;
            }

            let chunk = Bytes::copy_from_slice(writer.get_ref());
            writer.get_mut().clear();
            writer.set_position(0);
            read_buffer.clear();

            yield chunk;
        }

        let chunk = Bytes::copy_from_slice(writer.get_ref());
        writer.get_mut().clear();
        writer.set_position(0);
        read_buffer.clear();

        yield chunk;

    }
}


fn create_unescape_stream(body: Body) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));

    let mut input_reader = BufReader::new(StreamReader::new(request_body_stream));
    let mut unescaper = UnescapeStream::new();

    try_stream! {
        let mut writer = Cursor::new(Vec::<u8>::new());
        let mut read_buffer = BytesMut::with_capacity(1024);
        loop {
            let count = input_reader.read_buf(&mut read_buffer).await?;
            if count > 0 {
                let (boundary_char, rest_of_part) = unescaper.try_unescape_next(&read_buffer)?;

                if let Some(c) = boundary_char {
                    writer.write_u8(c as u8).await?;
                }

                for token in rest_of_part {
                    match token? {
                        UnescapedToken::Literal(literal) => writer.write_all(literal).await?,
                        UnescapedToken::Unescaped(ch) => writer.write_u8(ch as u8).await?,
                    }
                }
            } else {
                break;
            }

            let chunk = Bytes::copy_from_slice(writer.get_ref());
            writer.get_mut().clear();
            writer.set_position(0);
            read_buffer.clear();

            yield chunk;
        }

        let chunk = Bytes::copy_from_slice(writer.get_ref());
        writer.get_mut().clear();
        writer.set_position(0);
        read_buffer.clear();

        yield chunk;

    }
}
