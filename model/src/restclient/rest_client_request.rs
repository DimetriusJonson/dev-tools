use getset::{Getters, WithSetters};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Getters, WithSetters)]
#[get = "pub"]
#[set_with = "pub"]
pub struct RestClientRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
}
