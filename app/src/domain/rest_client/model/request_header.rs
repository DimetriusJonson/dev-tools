use std::slice::{Iter, IterMut};

use leptos::prelude::{GetUntracked, ReadSignal, RwSignal, WriteSignal};
use uuid::Uuid;

use crate::{
    components::layout::property_editor::KeyValueTableItem,
    domain::rest_client::util::request_store::{RequestFieldKind, get_stored_value},
};

#[derive(Clone, Debug)]
pub struct RequestHeader {
    pub id: String,
    pub name: RwSignal<String>,
    pub value: RwSignal<String>,
}

#[derive(Clone, Debug)]
pub struct RequestHeaders {
    inner: Vec<RequestHeader>,
}

impl RequestHeader {
    pub fn new(name: String, value: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: RwSignal::new(name),
            value: RwSignal::new(value),
        }
    }
}

impl RequestHeaders {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, header: RequestHeader) {
        self.inner.push(header)
    }

    pub fn iter(&self) -> Iter<'_, RequestHeader> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, RequestHeader> {
        self.inner.iter_mut()
    }

    pub fn remove_by_id(&mut self, id: String) {
        self.inner.retain(|h| h.id != id);
    }

    pub fn vec_owned(&self) -> Vec<RequestHeader> {
        self.inner.clone()
    }

    pub fn read_from_store(project_id: &str, request_id: i32) -> Self {
        let stored_value =
            get_stored_value(RequestFieldKind::Headers, "".to_owned(), project_id, request_id);
        if stored_value.is_empty() {
            return RequestHeaders::new();
        }

        let mut result = RequestHeaders::new();
        for line in stored_value.lines() {
            if let Some(index) = line.find(":") {
                result.push(RequestHeader::new(
                    line[..index].to_owned(),
                    line[index + 1..].to_owned(),
                ));
            }
        }

        result
    }
}

impl ToString for RequestHeaders {
    fn to_string(&self) -> String {
        self.inner
            .iter()
            .map(|h| format!("{}:{}", h.name.get_untracked(), h.value.get_untracked()))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl KeyValueTableItem for RequestHeader {
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn name(&self) -> ReadSignal<String> {
        self.name.read_only()
    }

    fn set_name(&self) -> WriteSignal<String> {
        self.name.write_only()
    }

    fn value(&self) -> ReadSignal<String> {
        self.value.read_only()
    }

    fn set_value(&self) -> WriteSignal<String> {
        self.value.write_only()
    }
}
