use std::collections::HashMap;
use std::io::Cursor;

use async_stream::try_stream;
use axum::body::Body;
use axum::extract::RawQuery;
use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use tokio::io::BufReader;
use tokio_util::io::StreamReader;

pub async fn format_xml_handler(RawQuery(query): RawQuery, body: Body) -> impl IntoResponse {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);

    let ident: usize = params.get("ident").unwrap_or(&"4").parse().unwrap();

    Body::from_stream(create_stream(body, ident))
}

fn create_stream(body: Body, ident: usize) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));

    let mut input_xml_reader =
        Reader::from_reader(BufReader::new(StreamReader::new(request_body_stream)));
    input_xml_reader.config_mut().trim_text(false);

    try_stream! {
        let mut writer = Writer::new_with_indent(Cursor::new(Vec::<u8>::new()), b' ', ident);
        let mut read_buffer = Vec::<u8>::new();
        loop {
            match input_xml_reader.read_event_into_async(&mut read_buffer).await? {
                Event::Text(ref e) => {
                    let text_content = input_xml_reader.decoder().decode(e)?;
                    let filtered_lines: Vec<&str> =
                        text_content.lines().filter(|line| !line.trim().is_empty()).collect();

                    if !filtered_lines.is_empty() {
                        let filtered_text = filtered_lines.join("\n");
                        writer.write_event(Event::Text(BytesText::new(&filtered_text)))?;
                    }
                }
                Event::Comment(e) => {
                    writer.write_event(Event::Comment(e))?;
                }
                Event::Eof => break,
                event => writer.write_event(event)?,
            };

            let chunk = Bytes::copy_from_slice(writer.get_ref().get_ref());
            writer.get_mut().get_mut().clear();
            writer.get_mut().set_position(0);
            read_buffer.clear();

            yield chunk;
        }
    }
}

fn parse_query_params<'a>(query_str: &'a str) -> HashMap<&'a str, &'a str> {
    query_str
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((key, value))
        })
        .collect()
}
