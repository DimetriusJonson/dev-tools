use std::io::Cursor;

use async_stream::try_stream;
use axum::body::Body;
use axum::extract::{Multipart, RawQuery};
use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use tokio::io::{AsyncBufRead, BufReader};
use tokio_util::io::StreamReader;

use crate::common::dev_utils::parse_query_params;

pub async fn format_xml_file_handler(mut multipart: Multipart) -> impl IntoResponse {
    let mut ident = 0;
    while let Some(field) = multipart.next_field().await.expect("Failed read multipart!") {
        let name = field.name().unwrap_or("unknown").to_string();

        if name == "ident" {
            ident = field.text().await.unwrap().parse::<usize>().unwrap();
        } else if name == "file_data" {
            let output_stream = create_stream_from_bytes(field.bytes().await.unwrap(), ident).await;
            return Body::from_stream(output_stream);
        }
    }

    Body::from("No multipart data!")
}

pub async fn format_xml_handler(RawQuery(query): RawQuery, body: Body) -> impl IntoResponse {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);

    let ident: usize = params.get("ident").unwrap_or(&"4").parse().unwrap();

    Body::from_stream(create_stream_from_body(body, ident))
}

fn create_stream_from_body(
    body: Body,
    ident: usize,
) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));

    let mut input_xml_reader =
        Reader::from_reader(BufReader::new(StreamReader::new(request_body_stream)));
    input_xml_reader.config_mut().trim_text(false);

    try_stream! {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::<u8>::new()), b' ', ident);
        let mut read_buffer = Vec::<u8>::new();
        loop {
            if let Some(chunk) = format_chunk(&mut input_xml_reader, &mut read_buffer, &mut writer).await? {
                yield chunk;
            } else {
                break;
            }
        }
    }
}

async fn create_stream_from_bytes(
    data: Bytes,
    ident: usize,
) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let mut input_xml_reader = Reader::from_reader(Cursor::new(data));
    input_xml_reader.config_mut().trim_text(false);
    try_stream! {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::<u8>::new()), b' ', ident);
        let mut read_buffer = Vec::<u8>::new();
        loop {
            if let Some(chunk) = format_chunk(&mut input_xml_reader, &mut read_buffer, &mut writer).await? {
                yield chunk;
            } else {
                break;
            }
        }
    }
}

async fn format_chunk<R>(
    input_xml_reader: &mut Reader<R>,
    read_buffer: &mut Vec<u8>,
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<Option<Bytes>, anyhow::Error>
where
    R: AsyncBufRead + Unpin,
{
    match input_xml_reader.read_event_into_async(read_buffer).await {
        Ok(Event::Text(ref e)) => {
            let text_content = input_xml_reader.decoder().decode(e)?;
            let filtered_lines: Vec<&str> =
                text_content.lines().filter(|line| !line.trim().is_empty()).collect();

            if !filtered_lines.is_empty() {
                let filtered_text = filtered_lines.join("\n");
                writer.write_event(Event::Text(BytesText::new(&filtered_text)))?;
            }
        }
        Ok(Event::Comment(e)) => {
            writer.write_event(Event::Comment(e))?;
        }
        Ok(Event::Eof) => return Ok(None),
        Ok(event) => writer.write_event(event)?,
        Err(err) => {
            eprintln!("Error at position {}: {:?}", input_xml_reader.error_position(), err);
        }
    };

    let chunk = Bytes::copy_from_slice(writer.get_ref().get_ref());
    writer.get_mut().get_mut().clear();
    writer.get_mut().set_position(0);
    read_buffer.clear();

    return Ok(Some(chunk));
}
