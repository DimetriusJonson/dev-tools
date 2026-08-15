use leptos::prelude::{GetUntracked, ReadUntracked};

use crate::domain::rest_client::model::request_params::{RequestBodyKind, RequestParams};

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
        result.push_str(&format!(" {}-X {}", new_line, request_params.method.read_untracked()));
    }

    let method = request_params.method.get_untracked();

    //headers
    if (&method == "POST" || &method == "PUT")
        && !request_params.body.read_untracked().is_empty()
        && request_params.content_type().is_none()
    {
        add_header_param(
            &mut result,
            "Content-Type",
            request_params.body_type.get_untracked().content_type(),
            quote,
            new_line,
        );
    }

    for header in request_params.headers.read_untracked().iter() {
        add_header_param(
            &mut result,
            &header.name.read_untracked(),
            &header.value.read_untracked(),
            quote,
            new_line,
        );
    }

    // body
    let body = match request_params.body_type.get_untracked() {
        RequestBodyKind::Text | RequestBodyKind::Json | RequestBodyKind::Xml => {
            escape_string(&request_params.body.read_untracked())
        }
        RequestBodyKind::Formencoded => {
            match request_params.body_formencoded.get_untracked().to_urlencoded_string() {
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

fn add_header_param(result: &mut String, name: &str, value: &str, quote: &str, new_line: &str) {
    result.push_str(&format!(" {}", new_line));
    result.push_str(&format!("-H {}{}: {}{}", quote, name, escape_string(value), quote));
}

fn win_cmd_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '&' | '|' | '<' | '>' | '\\' | '"' => result.push('^'),
            '^' => {
                if let Some(&next_ch) = chars.peek()
                    && next_ch != '\r'
                {
                    result.push('^');
                }
            }
            _ => (),
        }
        result.push(ch);
    }

    result.replace("%", "^%^")
}

fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars = s.chars();

    for ch in chars {
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
