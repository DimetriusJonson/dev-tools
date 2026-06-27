use std::io::{self, Cursor, Write};

use async_stream::try_stream;
use axum::{body::Body, extract::RawQuery, response::IntoResponse};
use bytes::Bytes;
use futures_util::Stream;
use tokio::io::AsyncReadExt;

use crate::common::dev_utils::parse_query_params;

pub async fn format_json_handler(RawQuery(query): RawQuery, bytes: Bytes) -> impl IntoResponse {
    let query_str = query.unwrap_or_default();
    let params = parse_query_params(&query_str);
    let ident: usize = params.get("ident").unwrap_or(&"4").parse().unwrap();

    Body::from_stream(create_stream(bytes, ident))
}

fn create_stream(data: Bytes, ident: usize) -> impl Stream<Item = Result<Bytes, anyhow::Error>> {
    let mut reader = Cursor::new(data);

    try_stream! {
        let mut escaped = false;
        let mut in_string = false;
        let mut indent_level = 0usize;
        let mut newline_requested = false; // invalidated if next character is ] or }

        let mut writer = Cursor::new(Vec::<u8>::new());

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
                    b'"' => {
                        if !escaped {
                            in_string = false;
                        }
                    }
                    b'\\' => {
                        if !escaped {
                            escape_here = true;
                        }
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
                            writer.write_all(b"\n")?;
                            write_ident(&mut writer, indent_level, ident)?;
                        }
                    }
                    b':' => {
                        auto_push = false;
                        writer.write_all(&[char])?;
                        writer.write_all(&[b' '])?;
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
            writer.write(b" ")?;
        }
    }

    Ok(())
}
