use app::common::app_error::AppError;
use axum::{body::Body, extract::Multipart, response::IntoResponse};
use difference::{Changeset, Difference};
use http::{StatusCode, header};

pub async fn compare_text_handler(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let mut text1 = "".to_owned();
    let mut text2 = "".to_owned();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let value = String::from_utf8_lossy(&data).to_string();

        if name == "text1" {
            text1 = value;
        } else if name == "text2" {
            text2 = value;
        }
    }

    let Changeset { diffs, .. } = Changeset::new(&text1, &text2, "\n");
    let mut result_rows = Vec::new();
    for i in 0..diffs.len() {
        println!("diff {}", i);
        match diffs[i] {
            difference::Difference::Same(ref x) => {
                result_rows.push(format!("<div><pre>{}</pre></div>", html_escape::encode_text(x)));
            }
            difference::Difference::Add(ref x) => {
                let mut text_row = "".to_owned();
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        text_row.push_str("<span class=\"text-green-500\">+</span>");
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    text_row.push_str(&format!(
                                        "<span class=\"text-green-500\">{}&nbsp;</span>",
                                        html_escape::encode_text(z)
                                    ));
                                }
                                Difference::Add(ref z) => {
                                    text_row.push_str(&format!(
                                        "<span class=\"text-white bg-green-500\">{}</span>",
                                        html_escape::encode_text(z)
                                    ));
                                    text_row.push_str("<span>&nbsp;</span>");
                                }
                                _ => (),
                            }
                        }
                        println!("text={}", text_row);
                        result_rows.push(format!("<div><pre>{}</pre></div>", text_row));
                    }
                    _ => {
                        result_rows.push(format!("<div class=\"text-green-300\">+{}</div>", html_escape::encode_text(x)));
                    }
                };
            }
            difference::Difference::Rem(ref x) => {
                result_rows.push(format!("<div class=\"text-red-500\">-<pre>{}</pre></div>", html_escape::encode_text(x)));
            }
        }
    }

    let body = Body::new(result_rows.join("\n"));

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}
