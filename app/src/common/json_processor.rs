use bytes::Bytes;

use crate::common::json_formatter::JsonFormatter;

pub fn format_json(json: &str, ident: usize) -> String {
    let mut formatter = JsonFormatter::new(ident);
    let formatted_bytes = formatter.format_bytes(Bytes::from(json.to_owned()));

    String::from_utf8_lossy(&formatted_bytes).to_string()
}
