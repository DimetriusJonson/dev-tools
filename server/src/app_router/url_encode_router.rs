use urlencoding::{decode, encode};

pub async fn encode_url_handler(url: String) -> String {
     encode(&url).to_string()
}
pub async fn decode_url_handler(encoded: String) -> String {
     match decode(&encoded) {
        Ok(decoded) => decoded.to_string(), 
        Err(e) => e.to_string(),
    }
}