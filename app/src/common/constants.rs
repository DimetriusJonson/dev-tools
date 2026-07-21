pub const REMOTE_SERVER_HOST: &str = "dev-tools-rust.vercel.app";

pub const MEDIA_TYPES: [(&str, &str); 4] = [
    ("application/xml", "xml"),
    ("application/json", "json"),
    ("text/html", "html"),
    ("text/xml", "xml"),
];

pub const MEDIA_TYPES_AUTOCOMPLETE: [&str; 23] = [
    "*/*",
    "application/atom+xml",
    "application/cbor",
    "application/x-www-form-urlencoded",
    "application/graphql+json",
    "application/json",
    "application/json;charset=UTF-8",
    "application/problem+json",
    "application/problem+json;charset=UTF-8",
    "application/problem+xml",
    "application/rss+xml",
    "application/x-ndjson",
    "application/stream+json",
    "application/xhtml+xml",
    "application/xml",
    "multipart/form-data",
    "multipart/mixed",
    "multipart/related",
    "text/event-stream",
    "text/html",
    "text/markdown",
    "text/plain",
    "text/xml",
];
