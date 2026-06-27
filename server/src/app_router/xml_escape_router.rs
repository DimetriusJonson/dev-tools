use std::io::{Cursor, Write};

use async_stream::try_stream;
use axum::body::Body;
use axum::response::IntoResponse;
use bytes::Bytes;
use futures_util::Stream;
use quick_xml::escape::{escape, unescape};
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};

pub async fn unescape_xml_handler(bytes: Bytes) -> impl IntoResponse {
    Body::from_stream(create_stream(bytes, false))
}

pub async fn escape_xml_handler(bytes: Bytes) -> impl IntoResponse {
    Body::from_stream(create_stream(bytes, true))
}

fn create_stream(
    data: Bytes,
    escape_xml: bool,
) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let mut input_xml_reader = Reader::from_reader(Cursor::new(data));
    input_xml_reader.config_mut().trim_text(false);

    try_stream! {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));
        let mut read_buffer = Vec::<u8>::new();
        loop {
            match input_xml_reader.read_event_into_async(&mut read_buffer).await {
                Ok(Event::Eof) => break,
                Ok(event) => writer.write_event(event)?,
                Err(err) => {
                    let err_str = format!("Error at position {}: {:?}", input_xml_reader.error_position(), err);
                    writer.get_mut().write_all(err_str.as_bytes())?;
                    break;
                }
            };

            let str = String::from_utf8_lossy(writer.get_ref().get_ref());
            let converted_str = match escape_xml {
                true => escape(str),
                false => unescape(&str)?,
            };

            let chunk = Bytes::copy_from_slice(converted_str.as_bytes());
            writer.get_mut().get_mut().clear();
            writer.get_mut().set_position(0);
            read_buffer.clear();

            yield chunk;
        }
    }
}
