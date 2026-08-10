use leptos::prelude::{GetUntracked, ReadUntracked};

use crate::domain::rest_client::{
    model::request_params::{RequestBodyKind, RequestParams},
    util::rest_client_utils::formencoded_to_str,
};

pub fn build_curl_bash_cmd(request_params: &RequestParams) -> String {
    build_curl_cmd(request_params, "curl", "\'", "\\\r\n")
}

pub fn build_curl_win_cmd(request_params: &RequestParams) -> String {
    win_cmd_escape(&build_curl_cmd(request_params, "curl.exe", "\"", "^\r\n"))
}

fn build_curl_cmd(
    request_params: &RequestParams,
    process_name: &str,
    quote: &str,
    new_line: &str,
) -> String {
    let mut result = String::new();

    result.push_str(&format!("{} ", process_name));

    // url
    result.push_str(&format!("{}{}{}", quote, request_params.url.read_untracked(), quote));

    //method
    if request_params.method.read_untracked() != "GET".to_owned() {
        result.push_str(&format!(" {}-X {}", new_line, &request_params.method.read_untracked()));
    }

    //headers
    for header in request_params.headers.read_untracked().iter() {
        result.push_str(&format!(" {}", new_line));
        result.push_str(&format!(
            "-H {}{}: {}{}",
            quote,
            &header.name.read_untracked(),
            &escape_string(&header.value.read_untracked()),
            quote
        ));
    }

    // body
    let body = match request_params.body_type.get_untracked() {
        RequestBodyKind::Text => request_params.body.get_untracked(),
        RequestBodyKind::Formencoded => {
            match formencoded_to_str(request_params.body_formencoded.get_untracked()) {
                Ok(url) => url,
                Err(_err) => "".to_owned(),
            }
        }
    };

    if !body.is_empty() {
        result.push_str(&format!(" {}", new_line));
        result.push_str(&format!("--data-raw {}{}{}", quote, body, quote));
    }

    result
}

fn win_cmd_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    let mut chars = s.chars();
    let mut ch = chars.next();
    while ch.is_some() {
        let current_ch = ch.unwrap();
        match current_ch {
            '&' | '|' | '<' | '>' | '\\' | '"' => result.push('^'),
            '^' => {
                if let Some(next_ch) = chars.next() {
                    if next_ch != '\r' {
                        result.push('^');
                    }
                    result.push(current_ch);
                    ch = Some(next_ch);
                    continue;
                }
            }
            _ => (),
        }
        result.push(current_ch);
        ch = chars.next();
    }

    result
}

fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\\' => {
                result.push('\\');
                result.push(ch);
            }
            '\n' => {
                result.push('\\');
                result.push('n');
            }
            '\r' => {
                result.push('\\');
                result.push('r');
            }
            '\t' => {
                result.push('\\');
                result.push('t');
            }
            _ => result.push(ch),
        }
    }

    result
}
