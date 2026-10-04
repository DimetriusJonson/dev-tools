use getset::{CopyGetters, Getters, MutGetters, Setters};
use http::{HeaderMap, Method};

#[derive(Debug, Clone, Getters, MutGetters, CopyGetters, Setters, Default)]
#[set="pub"]
pub struct ParsedRequest {
    #[get = "pub"]
    method: Option<Method>,
    #[get = "pub"]
    url: String,
    #[getset(get = "pub", get_mut = "pub")]
    headers: HeaderMap,
    #[getset(get = "pub", get_mut = "pub")]
    body: Vec<String>,
    #[getset(get = "pub", get_mut = "pub")]
    body_urlencode: String,
    #[get_copy = "pub"]
    insecure: bool,
    #[get_copy = "pub"]
    compressed: bool,
}
