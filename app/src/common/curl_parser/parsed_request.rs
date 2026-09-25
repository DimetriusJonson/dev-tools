use http::{HeaderMap, Method};

#[derive(Debug, Clone)]
pub struct ParsedRequest {
    pub method: Option<Method>,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Vec<String>,
    pub body_urlencode: String,
    pub insecure: bool,
    pub compressed: bool,
}

impl Default for ParsedRequest {
    fn default() -> Self {
        Self {
            method: None,
            url: String::new(),
            headers: HeaderMap::with_capacity(8),
            body: Vec::with_capacity(4),
            body_urlencode: "".to_owned(),
            insecure: false,
            compressed: false,
        }
    }
}
