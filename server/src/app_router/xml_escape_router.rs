pub async fn unescape_xml_handler(body: String) -> String {
    match quick_xml::escape::unescape(&body) {
        Ok(res) => res.to_string(),
        Err(err) => err.to_string(),
    }
}

pub async fn escape_xml_handler(body: String) -> String {
    quick_xml::escape::escape(&body).to_string()
}
