use base64::{Engine, engine::general_purpose::STANDARD};
use http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderName},
};
use pest::Parser as _;
use pest_derive::Parser;
use std::{error::Error, str::FromStr};

use crate::common::curl_parser::parsed_request::ParsedRequest;

#[derive(Debug, Parser)]
#[grammar = "src/common/curl_parser/curl.pest"]
pub struct CurlParser;

pub fn parse_curl_cmd(input: &str) -> Result<ParsedRequest, Box<dyn Error>> {
    let pairs = CurlParser::parse(Rule::input, input)?;
    let mut parsed = ParsedRequest::default();
    for pair in pairs {
        match pair.as_rule() {
            Rule::method => {
                let method = pair.as_str().parse()?;
                parsed.method = method;
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
                let s = pair.into_inner().next().expect("location string must be present").as_str();
                let location = s.to_string();
                parsed.url = location;
            }
            Rule::header => {
                let s = pair.into_inner().next().expect("header string must be present").as_str();

                // Use split_once for better performance
                if let Some((name, value)) = s.split_once(':') {
                    let header_value = unescape_string(value.trim());
                    parsed.headers.insert(
                        HeaderName::from_str(name.trim())?,
                        HeaderValue::from_str(&header_value)?,
                    );
                } else {
                    // Fallback for malformed headers (should be rare)
                    let mut kv = s.splitn(2, ':');
                    let name = kv.next().expect("key must present").trim();
                    let value = kv.next().expect("value must present").trim();
                    let header_value = unescape_string(value);
                    parsed
                        .headers
                        .insert(HeaderName::from_str(name)?, HeaderValue::from_str(&header_value)?);
                }
            }
            Rule::cookie => {
                let cookie_value = pair.into_inner().next().expect("cookie string must be present").as_str();

                let header_value = unescape_string(cookie_value.trim());
                parsed.headers.insert(
                    HeaderName::from_str("Cookie")?,
                    HeaderValue::from_str(&header_value)?,
                );
            }
            Rule::auth => {
                let s = pair.into_inner().next().expect("header string must be present").as_str();
                let encoded = STANDARD.encode(s.as_bytes());
                // Pre-allocate with known prefix length
                let mut basic_auth = String::with_capacity(6 + encoded.len()); // "Basic " + encoded
                basic_auth.push_str("Basic ");
                basic_auth.push_str(&encoded);
                parsed.headers.insert(AUTHORIZATION, basic_auth.parse()?);
            }
            Rule::body => {
                let s = pair.as_str().trim();
                let s = remove_quote(s);
                parsed.body.push(s.into());
            }
            Rule::data_raw => {
                let data_raw_value = pair.into_inner().next().expect("data-raw string must be present").as_str();
                let data_raw_value = data_raw_value.replace("\\r\\n", "\r\n");
                parsed.body.push(data_raw_value.into());
            }
            Rule::ssl_verify_option => {
                parsed.insecure = true;
            }
            Rule::compressed_option => {
                parsed.compressed = true;
            }
            Rule::url_option => {
            }
            Rule::EOI => break,
            _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
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
    if !parsed.body.is_empty() && parsed.method == Method::GET {
        parsed.method = Method::POST
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
