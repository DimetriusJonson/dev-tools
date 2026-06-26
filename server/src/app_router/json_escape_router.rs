
pub async fn unescape_json_handler(encoded: String) -> String {
     match unescaper::unescape(&encoded) {
        Ok(res) => res.to_string(),
        Err(err) => err.to_string(),
    }
}