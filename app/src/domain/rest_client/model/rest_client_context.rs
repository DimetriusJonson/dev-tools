use leptos::prelude::{ReadUntracked, RwSignal};

use crate::domain::rest_client::model::request_params::RequestInfo;

#[derive(Clone)]
pub struct RestClientContext {
    pub project: RwSignal<String>,
    pub request: RwSignal<RequestInfo>,
}

impl RestClientContext {
    pub fn project_id(&self) -> i32 {
        self.project.read_untracked().parse::<i32>().unwrap_or(0)
    }
}
