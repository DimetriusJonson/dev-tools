use std::io::Cursor;

use app::common::app_error::AppError;
use async_stream::try_stream;
use axum::body::Body;
use axum::extract::RawQuery;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::Stream;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use tokio::io::{AsyncBufRead, BufReader};

use crate::common::dev_utils::parse_query_params;

pub async fn format_xml_handler(
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
        .header(header::CONTENT_TYPE, "application/xml")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

async fn create_stream(body: Body, ident: usize) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let mut input_xml_reader = build_reader(body).await;

    try_stream! {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::<u8>::new()), b' ', ident);
        let mut read_buffer = Vec::<u8>::new();
        loop {
            match format_chunk(&mut input_xml_reader, &mut read_buffer, &mut writer).await {
                Ok(Some(chunk)) => yield chunk,
                Ok(None) => break,
                Err(err) => {
                    yield Bytes::from_owner(err.to_string());
                    break;
                },
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
async fn build_reader(body: Body) -> Reader<impl AsyncBufRead + Unpin> {
    use futures_util::StreamExt;

    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));
    let mut input_xml_reader =
        Reader::from_reader(BufReader::new(tokio_util::io::StreamReader::new(request_body_stream)));
    input_xml_reader.config_mut().trim_text(false);
    input_xml_reader
}

#[cfg(target_os = "windows")]
async fn build_reader(body: Body) -> Reader<impl AsyncBufRead + Unpin> {
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();

    let mut input_xml_reader = Reader::from_reader(BufReader::new(Cursor::new(bytes)));
    input_xml_reader.config_mut().trim_text(false);
    input_xml_reader
}

async fn format_chunk<R>(
    input_xml_reader: &mut Reader<R>,
    read_buffer: &mut Vec<u8>,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<Option<Bytes>, anyhow::Error>
where
    R: AsyncBufRead + Unpin,
{
    match input_xml_reader.read_event_into_async(read_buffer).await? {
        Event::Text(ref e) => {
            let text_content = input_xml_reader.decoder().decode(e)?;
            let filtered_lines: Vec<&str> =
                text_content.lines().filter(|line| !line.trim().is_empty()).collect();

            if !filtered_lines.is_empty() {
                let filtered_text = filtered_lines.join("\n");
                writer.write_event(Event::Text(BytesText::new(&filtered_text)))?;
            }
        }
        Event::Comment(e) => writer.write_event(Event::Comment(e))?,
        Event::Eof => return Ok(None),
        event => writer.write_event(event)?,
    };

    let chunk = Bytes::copy_from_slice(writer.get_ref().get_ref());
    writer.get_mut().get_mut().clear();
    writer.get_mut().set_position(0);
    read_buffer.clear();

    Ok(Some(chunk))
}
