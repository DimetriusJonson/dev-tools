use app::common::app_error::AppError;
use axum::{body::Body, extract::Multipart, response::IntoResponse};
use difference::{Changeset, Difference};
use http::{StatusCode, header};

pub async fn compare_text_handler(multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    let (text1, text2) = extract_params(multipart).await;

    let result_left = compare_text(&text1, &text2);
    let result_right = compare_text(&text2, &text1);

    let body = Body::new(vec![result_left, result_right].join("\n$$$---$$$\n"));

    let response = axum::http::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(body)
        .map_err(AppError::system_error)?;

    Ok(response)
}

fn compare_text(text1: &str, text2: &str) -> String {
    let Changeset { diffs, .. } = Changeset::new(&text2, &text1, "\n");
    let mut result_rows = Vec::new();
    for i in 0..diffs.len() {
        match diffs[i] {
            difference::Difference::Same(ref x) => {
                for text in x.split('\n') {
                    result_rows.push(format!(
                        "<pre>{}. {}</pre>",
                        result_rows.len(),
                        html_escape::encode_text(text)
                    ));
                }
            }
            difference::Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        let mut text_row = "".to_owned();
                        text_row.push_str("<span class=\"text-green-500\">+</span>");
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    text_row.push_str(&format!(
                                        "<span class=\"text-green-500\">{} </span>",
                                        html_escape::encode_text(z)
                                    ));
                                }
                                Difference::Add(ref z) => {
                                    text_row.push_str(&format!(
                                        "<span class=\"text-white bg-green-500\">{}</span>",
                                        html_escape::encode_text(z)
                                    ));
                                    text_row.push_str("<span> </span>");
                                }
                                _ => (),
                            }
                        }
                        result_rows.push(format!("<pre>{}. {}</pre>", result_rows.len(), text_row));
                    }
                    _ => {
                        result_rows.push(format!(
                            "<pre class=\"text-green-300\">{}+{}</pre>",
                            result_rows.len(),
                            html_escape::encode_text(x)
                        ));
                    }
                };
            }
            difference::Difference::Rem(ref _x) => {
                /* for text in x.split('\n') {
                    result_rows.push(format!(
                        "<pre class=\"text-red-500\">{}-{}</pre>",
                        result_rows.len(),
                        html_escape::encode_text(text)
                    ));
                }
                 */
            }
        }
    }
    result_rows.join("\n")
}

async fn extract_params(mut multipart: Multipart) -> (String, String) {
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

    (text1, text2)
}