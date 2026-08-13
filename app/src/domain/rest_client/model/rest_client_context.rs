use leptos::prelude::{ReadSignal, ReadUntracked, WriteSignal};

use crate::domain::rest_client::model::request_params::RequestInfo;

#[derive(Clone)]
pub struct RestClientContext {
    pub project: ReadSignal<String>,
    pub request: ReadSignal<RequestInfo>,
    pub set_request: WriteSignal<RequestInfo>,
}

impl RestClientContext {
    pub fn project_id(&self) -> i32 {
        self.project.read_untracked().parse::<i32>().unwrap_or(0)
    }
}