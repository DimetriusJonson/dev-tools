use std::{collections::HashMap, net::SocketAddr};

use http::{HeaderMap, Uri};

pub fn extract_uri_query_params(uri: &Uri) -> HashMap<&str, &str> {
    if let Some(query_str) = uri.query() {
        query_str
            .split('&')
            .filter(|pair| !pair.is_empty())
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or("");
                Some((key, value))
            })
            .collect()
    } else {
        HashMap::new()
    }
}

pub fn is_mime_image(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/bmp" | "image/png" | "image/jpeg" | "image/webp" | "image/gif" | "image/apng"
    )
}

pub fn resolve_request_ip(headers: &HeaderMap, client_addr: SocketAddr) -> String {
    let forwarded_for = headers
        .get("x-forwarded-for")
        .and_then(|val| val.to_str().ok())
        .map(|value| value.to_string())
        .unwrap_or("".to_owned());
    let real_ip = client_addr.ip().to_string();

    if !forwarded_for.is_empty() {
        forwarded_for.split(',').next().unwrap_or(&real_ip).trim().to_owned()
    } else {
        real_ip
    }
}
