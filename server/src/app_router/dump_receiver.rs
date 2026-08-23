#![warn(rust_2018_idioms)]
use bytes::BufMut;
use futures_util::lock::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{error, info};

use std::error::Error;

const HTTP_200: &str = r#"HTTP/1.1 200 OK
Content-Type: text/html; charset=UTF-8
Connection: close

OK"#;

const BREAK_LINE: &str = "\r\n\r\n";

pub static DUMP_REQUEST: Mutex<Vec<u8>> = Mutex::new(Vec::new());

pub async fn start_dump_receiver() -> Result<(JoinHandle<()>, u16), Box<dyn Error>> {
    const BUFFER_SIZE: usize = 4096;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    info!("Dump receiver listening on: 127.0.0.1:{port}");

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut socket, addr)) => {
                    DUMP_REQUEST.lock().await.clear();
                    tokio::spawn(async move {
                        let mut buf = vec![0; BUFFER_SIZE];
                        let mut max_size = 0;
                        loop {
                            let mut data = DUMP_REQUEST.lock().await;

                            match socket.read(&mut buf).await {
                                Ok(0) => {
                                    break;
                                }
                                Ok(n) => {
                                    data.put_slice(&buf[0..n]);

                                    if max_size != 0 && data.len() >= max_size {
                                        break;
                                    }

                                    if max_size == 0
                                        && let Some(pos_break_line) =
                                            find_pattern(&data, BREAK_LINE)
                                    {
                                        if let Some(content_len) =
                                            find_content_length(&data, pos_break_line)
                                        {
                                            max_size =
                                                pos_break_line + BREAK_LINE.len() + content_len;

                                            if max_size != 0 && data.len() >= max_size {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to read from socket {}: {}", addr, e);
                                    return;
                                }
                            }
                        }

                        if let Err(e) = socket.write_all(HTTP_200.as_bytes()).await {
                            error!("Failed to write to socket {}: {}", addr, e);
                        }
                    });
                }
                Err(err) => {
                    error!("Dump receiver error: {}", err);
                    return;
                }
            }
        }
    });

    Ok((handle, port))
}

fn find_pattern(data: &[u8], pattern: &str) -> Option<usize> {
    let pattern = pattern.to_lowercase();

    let pattern_len = pattern.len();
    for i in 0..data.len() - pattern_len + 1 {
        let part = String::from_utf8_lossy(&data[i..i + pattern_len]).to_string().to_lowercase();
        if part == pattern {
            return Some(i);
        }
    }

    None
}

fn find_content_length(data: &[u8], end_index: usize) -> Option<usize> {
    let name = "\r\ncontent-length:";

    if let Some(pos) = find_pattern(data, name) {
        let len_str = String::from_utf8_lossy(&data[pos + name.len()..end_index]).to_string();
        let len = len_str.trim().parse::<usize>().unwrap();
        return Some(len);
    }

    None
}
