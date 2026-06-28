use std::io::{self, Cursor, Write};

use async_stream::try_stream;
use axum::{body::Body, extract::RawQuery};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use tokio::io::{AsyncReadExt, BufReader};
use tokio_util::io::StreamReader;

use crate::common::{app_error::AppError, dev_utils::parse_query_params};

pub async fn format_json_handler(RawQuery(query): RawQuery, body: Body) -> Result<Body, AppError> {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let ident: usize =
        params.get("ident").unwrap_or(&"4").parse().map_err(AppError::system_error)?;

    Ok(Body::from_stream(create_stream(body, ident)))
}

fn create_stream(body: Body, ident: usize) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let request_body_stream =
        body.into_data_stream().map(|result| result.map_err(std::io::Error::other));

    let mut reader = BufReader::new(StreamReader::new(request_body_stream));

    try_stream! {
        let mut escaped = false;
        let mut in_string = false;
        let mut indent_level = 0usize;
        let mut newline_requested = false; // invalidated if next character is ] or }

        let mut writer = Cursor::new(Vec::<u8>::new());

        let mut found_first_brace = false;
        let mut need_open_brace = false;

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

            if !found_first_brace && char == b'{' {
                found_first_brace = true;
            }

            if !found_first_brace {
                continue;
            }

            if need_open_brace && char != b'{' && char != b'[' && char != b'}' && char != b']' {
                continue;
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
                writer.write_all(&[char])?;
                escaped = escape_here;
            } else {
                let mut auto_push = true;
                let mut request_newline = false;
                let old_level = indent_level;

                match char {
                    b'"' => in_string = true,
                    b' ' | b'\n' | b'\r' | b'\t' => continue,
                    b'[' => {
                        need_open_brace = false;
                        indent_level += 1;
                        request_newline = true;
                    }
                    b'{' => {
                        need_open_brace = false;
                        indent_level += 1;
                        request_newline = true;
                    }
                    b'}' | b']' => {
                        need_open_brace = true;
                        indent_level = indent_level.saturating_sub(1);
                        if !newline_requested {
                            // see comment below about newline_requested
                            writer.write_all(b"\n")?;
                            write_ident(&mut writer, indent_level, ident)?;
                        }
                    }
                    b':' => {
                        auto_push = false;
                        writer.write_all(&[char])?;
                        writer.write_all(b" ")?;
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
                    writer.write_all(b"\n")?;
                    write_ident(&mut writer, old_level, ident)?;
                }

                if auto_push {
                    writer.write_all(&[char])?;
                }

                newline_requested = request_newline;

                if writer.position() > 1024 {
                    let chunk = Bytes::copy_from_slice(writer.get_ref());
                    writer.get_mut().clear();
                    writer.set_position(0);

                    yield chunk;
                }
            }
        }

        // trailing newline
        writer.write_all(b"\n")?;

        let chunk = Bytes::copy_from_slice(writer.get_ref());
        writer.get_mut().clear();
        writer.set_position(0);

        yield chunk;
    }
}

fn write_ident<W>(writer: &mut W, level: usize, ident: usize) -> io::Result<()>
where
    W: Write,
{
    for _ in 0..level {
        for _ in 0..ident {
            writer.write_all(b" ")?;
        }
    }

    Ok(())
}
