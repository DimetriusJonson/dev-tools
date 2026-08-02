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
                                    //println!("received {}", String::from_utf8_lossy(&buf[0..n]));
                                    //println!("received {:?}", &buf[0..n]);

                                    data.put_slice(&buf[0..n]);

                                    //println!("data_len={} max_size={}", data.len(), max_size);

                                    if max_size != 0 && data.len() >= max_size {
                                        break;
                                    }

                                    if max_size == 0 && let Some(pos_break) = find_break(&data) {
                                        //println!("pos_break={}", pos_break);
                                        if let Some(content_len) =
                                            extract_content_length(&data, pos_break)
                                        {
                                            //println!("content_len={}", content_len);
                                            max_size = pos_break + 4 + content_len;

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
                            return;
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

fn find_str(data: &Vec<u8>, pattern: &str) -> Option<usize> {
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

fn find_break(data: &Vec<u8>) -> Option<usize> {
    find_str(data, "\r\n\r\n")
}

fn extract_content_length(data: &Vec<u8>, end_index: usize) -> Option<usize> {
    let name = "\r\ncontent-length:";

    if let Some(pos) = find_str(data, name) {
        let len_str = String::from_utf8_lossy(&data[pos + name.len()..end_index]).to_string();
        let len = len_str.trim().parse::<usize>().unwrap();
        return Some(len);
    }

    None
}
