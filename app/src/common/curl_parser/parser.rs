use base64::{Engine, engine::general_purpose::STANDARD};
use http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderName, USER_AGENT},
};
use pest::Parser as _;
use pest_derive::Parser;
use std::{error::Error, str::FromStr};

use crate::common::curl_parser::parsed_request::ParsedRequest;

#[derive(Debug, Parser)]
#[grammar = "src/common/curl_parser/curl.pest"]
pub struct CurlParser;

pub fn parse_curl_cmd(input: &str) -> Result<ParsedRequest, Box<dyn Error>> {
    let input = win_cmd_unescape(input.trim()); // win cmd double quote unescape 

    let pairs = CurlParser::parse(Rule::input, &input)?;
    let mut parsed = ParsedRequest::default();
    for pair in pairs {
        match pair.as_rule() {
            Rule::method => {
                if let Some(pair) = pair.into_inner().next() {
                    let method = pair.as_str().parse()?;
                    if ![
                        Method::GET,
                        Method::POST,
                        Method::PUT,
                        Method::DELETE,
                        Method::PATCH,
                        Method::HEAD,
                        Method::OPTIONS,
                    ]
                    .contains(&method)
                    {
                        return Err(format!("Unknown request method {}.", method).into());
                    };

                    parsed.method = Some(method);
                } else {
                    return Err("method value must be present".into());
                }
            }
            Rule::url => {
                let url = pair.into_inner().as_str();

                let url = if url.contains("://") {
                    url.to_string()
                } else {
                    // Pre-allocate with known prefix length
                    let mut full_url = String::with_capacity(8 + url.len()); // "http://" + url + "/"
                    full_url.push_str("http://");
                    full_url.push_str(url);
                    full_url.push('/');
                    full_url
                };

                parsed.url = url;
            }
            Rule::location => {
                if let Some(pair) = pair.into_inner().next() {
                    parsed.url = pair.to_string();
                } else {
                    return Err("location value must be present".into());
                }
            }
            Rule::header => {
                if let Some(pair) = pair.into_inner().next() {
                    // Use split_once for better performance
                    if let Some((name, value)) = pair.as_str().split_once(':') {
                        let header_value = unescape_string(value.trim());
                        parsed.headers.insert(
                            HeaderName::from_str(name.trim())?,
                            HeaderValue::from_str(&header_value)?,
                        );
                    } else {
                        // Fallback for malformed headers (should be rare)
                        let mut kv = pair.as_str().splitn(2, ':');

                        if let Some(name) = kv.next() {
                            if let Some(value) = kv.next() {
                                let header_value = unescape_string(value.trim());
                                parsed.headers.insert(
                                    HeaderName::from_str(name.trim())?,
                                    HeaderValue::from_str(&header_value)?,
                                );
                            } else {
                                return Err("value must present".into());
                            }
                        } else {
                            return Err("key must present".into());
                        }
                    }
                } else {
                    return Err("header value must be present".into());
                }
            }
            Rule::cookie => {
                if let Some(pair) = pair.into_inner().next() {
                    let header_value = unescape_string(pair.as_str().trim());
                    parsed.headers.insert(
                        HeaderName::from_str("Cookie")?,
                        HeaderValue::from_str(&header_value)?,
                    );
                } else {
                    return Err("cookie value must be present".into());
                }
            }
            Rule::auth => {
                if let Some(pair) = pair.into_inner().next() {
                    let encoded = STANDARD.encode(pair.as_str().as_bytes());
                    // Pre-allocate with known prefix length
                    let mut basic_auth = String::with_capacity(6 + encoded.len()); // "Basic " + encoded
                    basic_auth.push_str("Basic ");
                    basic_auth.push_str(&encoded);
                    parsed.headers.insert(AUTHORIZATION, basic_auth.parse()?);
                } else {
                    return Err("auth value must be present".into());
                }
            }
            Rule::body => {
                let s = pair.as_str().trim();
                let s = remove_quote(s);
                parsed.body.push(s.into());
            }
            Rule::data_raw => {
                if let Some(pair) = pair.into_inner().next() {
                    parsed.body.push(pair.as_str().replace("\\r\\n", "\r\n").replace("\\n", "\n"));
                } else {
                    return Err("data-raw value must be present".into());
                }
            }
            Rule::ssl_verify_option => {
                parsed.insecure = true;
            }
            Rule::compressed_option => {
                parsed.compressed = true;
            }
            Rule::url_option
            | Rule::verbose_option
            | Rule::output_option
            | Rule::head_option
            | Rule::fail_option
            | Rule::silent_option
            | Rule::show_headers_option => {}
            Rule::user_agent => {
                if let Some(pair) = pair.into_inner().next() {
                    parsed.headers.insert(USER_AGENT, pair.as_str().parse()?);
                } else {
                    return Err("user-agent value must be present".into());
                }
            }
            Rule::EOI => break,
            _ => return Err(format!("Unexpected rule: {:?}", pair.as_rule()).into()),
        }
    }

    if parsed.headers.get(CONTENT_TYPE).is_none() && !parsed.body.is_empty() {
        parsed
            .headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
    }
    if parsed.headers.get(ACCEPT).is_none() {
        parsed.headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    }
    if !parsed.body.is_empty() && parsed.method.is_none() {
        parsed.method = Some(Method::POST)
    }
    Ok(parsed)
}

fn remove_quote(s: &str) -> &str {
    let bytes = s.as_bytes();

    // Check if string is long enough and has matching quotes
    if bytes.len() >= 2 {
        match (bytes[0], bytes[bytes.len() - 1]) {
            (b'\'', b'\'') => &s[1..s.len() - 1],
            (b'"', b'"') => &s[1..s.len() - 1],
            _ => s,
        }
    } else {
        s
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    '"' | '\\' | '/' => result.push(next_ch),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => {
                        // If it's not a recognized escape sequence, keep both characters
                        result.push(ch);
                        result.push(next_ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn win_cmd_unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '^' {
            if let Some(next_ch) = chars.next() {
                match next_ch {
                    '&' | '|' | '<' | '>' | '^' | '\\' | '"' => result.push(next_ch),
                    _ => {
                        // If it's not a recognized escape sequence, keep both characters
                        result.push(ch);
                        result.push(next_ch);
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result.replace("^%^", "%")
}
