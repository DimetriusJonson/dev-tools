use leptos::prelude::RwSignal;

pub struct RequestResult {
    pub status_code: RwSignal<String>,
    pub body: RwSignal<String>,
    pub lang: RwSignal<String>,
    pub headers: RwSignal<Vec<(String, String)>>,
    pub request_raw: RwSignal<String>,
}

impl RequestResult {
    pub fn new() -> Self {
        Self {
            status_code: RwSignal::new("".to_owned()),
            body: RwSignal::new("".to_owned()),
            lang: RwSignal::new("".to_owned()),
            headers: RwSignal::new(Vec::new()),
            request_raw: RwSignal::new("".to_owned()),
        }
    }
}