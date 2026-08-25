use leptos::prelude::RwSignal;

#[derive(Clone)]
pub struct RequestResult {
    pub status_code: RwSignal<String>,
    pub size: RwSignal<Option<u64>>,
    pub body: RwSignal<String>,
    pub attachment: RwSignal<(String, String)>,
    pub image: RwSignal<String>,
    pub lang: RwSignal<String>,
    pub headers: RwSignal<Vec<(String, String)>>,
    pub request_raw: RwSignal<String>,
}

impl RequestResult {
    pub fn new() -> Self {
        Self {
            status_code: RwSignal::new("".to_owned()),
            size: RwSignal::new(None),
            body: RwSignal::new("".to_owned()),
            lang: RwSignal::new("".to_owned()),
            headers: RwSignal::new(Vec::new()),
            request_raw: RwSignal::new("".to_owned()),
            attachment: RwSignal::new(("".to_owned(), "".to_owned())),
            image: RwSignal::new("".to_owned()),
        }
    }
}
