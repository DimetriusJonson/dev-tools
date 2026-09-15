use std::sync::RwLock;
use std::{collections::HashMap, sync::LazyLock};

use model::constants::RC_BASE_URL_COOKIE_NAME;

static PROXY_CACHE: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn build_proxy_cache_key(base_url: &str, path: &str, query: Option<&str>) -> String {
    let mut query_str = query
        .map(|q| {
            q.split('&')
                .filter(|qv| !qv.starts_with(&format!("{}=", RC_BASE_URL_COOKIE_NAME)))
                .collect::<Vec<&str>>()
                .join("&")
        })
        .unwrap_or_default();
    if !query_str.is_empty() {
        query_str.insert(0, '?');
    }

    format!("{}:{}{}", base_url, path, query_str)
}

pub fn get_proxy_cached_value(base_url: &str, path: &str, query: Option<&str>) -> Option<String> {
    let key = build_proxy_cache_key(base_url, path, query);

    //info!("get cache {}", key);
    if let Ok(cache) = PROXY_CACHE.read() {
        return cache.get(&key).cloned();
    }
    None
}

pub fn set_proxy_cached_value(base_url: &str, path: &str, query: Option<&str>, value: String) {
    let key = build_proxy_cache_key(base_url, path, query);

    //info!("*** set cache {}", key);

    if let Ok(mut cache) = PROXY_CACHE.write() {
        cache.insert(key, value);
    }
}
