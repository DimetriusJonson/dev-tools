use std::collections::HashMap;

use http::Uri;

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
