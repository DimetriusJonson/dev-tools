use std::{convert::Infallible, fmt::Display, str::FromStr};

#[derive(PartialEq, Eq, Clone, Default)]
pub enum RequestBodyKind {
    #[default]
    Text,
    Json,
    Xml,
    Formencoded,
}

impl RequestBodyKind {
    pub fn content_type(&self) -> &str {
        match self {
            RequestBodyKind::Text => "text/plain",
            RequestBodyKind::Json => "application/json",
            RequestBodyKind::Xml => "application/xml",
            RequestBodyKind::Formencoded => "application/x-www-form-urlencoded",
        }
    }

    pub fn from_content_type(content_type: &str) -> Self {
        match content_type.to_lowercase().trim() {
            "text/plain" => RequestBodyKind::Text,
            "application/xml" => RequestBodyKind::Xml,
            "application/x-www-form-urlencoded" => RequestBodyKind::Formencoded,
            _ => RequestBodyKind::Json,
        }
    }
}

impl Display for RequestBodyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestBodyKind::Text => write!(f, "text"),
            RequestBodyKind::Formencoded => write!(f, "formencoded"),
            RequestBodyKind::Json => write!(f, "json"),
            RequestBodyKind::Xml => write!(f, "xml"),
        }
    }
}

impl FromStr for RequestBodyKind {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "formencoded" {
            Ok(RequestBodyKind::Formencoded)
        } else if s == "json" {
            Ok(RequestBodyKind::Json)
        } else if s == "xml" {
            Ok(RequestBodyKind::Xml)
        } else {
            Ok(RequestBodyKind::Text)
        }
    }
}
