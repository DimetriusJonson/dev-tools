use std::collections::HashMap;
use std::io::Cursor;

use axum::body::Body;
use axum::extract::RawQuery;
use axum::http::{StatusCode, header};
use axum::response::Response;
use bytes::Bytes;
use futures_util::StreamExt;
use log::error;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use tokio::io::BufReader;
use tokio::sync::mpsc::{self, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::io::StreamReader;

pub async fn format_xml_handler(RawQuery(query): RawQuery, body: Body) -> Response {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);

    let ident: usize = params.get("ident").unwrap_or(&"4").parse().unwrap();

    let (tx, rx) = mpsc::channel::<Result<Bytes, axum::Error>>(16);
    tokio::spawn(async move {
        if let Err(err) = format_xml(body, tx.clone(), ident).await
            && let Err(err) = tx.send(Ok(format!("Error: {}", err).into())).await
        {
            error!("Error: {}", err);
        }
    });

    let stream = ReceiverStream::new(rx);

    let body = Body::from_stream(stream);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .body(body)
        .unwrap()
}

async fn format_xml(
    body: Body,
    tx: Sender<Result<Bytes, axum::Error>>,
    ident: usize,
) -> anyhow::Result<()> {
    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));

    let mut input_xml_reader =
        Reader::from_reader(BufReader::new(StreamReader::new(request_body_stream)));
    input_xml_reader.config_mut().trim_text(false);

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', ident);

    let mut read_buffer = Vec::new();
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
            Event::Eof => return Ok(()),
            event => writer.write_event(event)?,
        }

        if tx.send(Ok(Bytes::copy_from_slice(writer.get_ref().get_ref()))).await.is_err() {
            return Ok(()); // Client disconnected
        }

        writer.get_mut().get_mut().clear();
        writer.get_mut().set_position(0);
        read_buffer.clear();
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
