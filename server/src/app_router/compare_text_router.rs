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
        match diffs[i] {
            difference::Difference::Same(ref x) => {
                result_rows.push(format!("<div>{}</div>", x));
            }
            difference::Difference::Add(ref x) => {
                let mut text_row = "".to_owned();
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        //t.fg(term::color::GREEN).unwrap();
                        //write!(t, "+");
                        text_row.push_str("<span class=\"text-green-500\">+</span>");
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    //t.fg(term::color::GREEN).unwrap();
                                    //write!(t, "{}", z);
                                    //write!(t, " ");
                                    text_row.push_str(&format!(
                                        "<span class=\"text-green-500\">{}&nbsp;</span>",
                                        z
                                    ));
                                }
                                Difference::Add(ref z) => {
                                    //t.fg(term::color::WHITE).unwrap();
                                    //t.bg(term::color::GREEN).unwrap();
                                    //write!(t, "{}", z);
                                    //t.reset().unwrap();
                                    //write!(t, " ");

                                    text_row.push_str(&format!(
                                        "<span class=\"text-white bg-green-500\">{}</span>",
                                        z
                                    ));
                                    text_row.push_str("<span>&nbsp;</span>");
                                }
                                _ => (),
                            }
                        }
                        result_rows.push(format!("<div>{}</div>", text_row));
                    }
                    _ => {
                        //t.fg(term::color::BRIGHT_GREEN).unwrap();
                        //writeln!(t, "+{}", x);
                        result_rows.push(format!("<div class=\"text-green-300\">+{}</div>", x));
                    }
                };
            }
            difference::Difference::Rem(ref x) => {
                //t.fg(term::color::RED).unwrap();
                //writeln!(t, "-{}", x);
                result_rows.push(format!("<div class=\"text-red-500\">-{}</div>", x));
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
