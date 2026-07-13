use std::collections::HashMap;

pub fn parse_query_params(query_str: &str) -> HashMap<&str, &str> {
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
}

pub fn is_mime_image(mime_type: &str) -> bool {
    matches!(mime_type, "image/bmp" | "image/png" | "image/jpeg" | "image/webp" | "image/gif" | "image/apng")
}

