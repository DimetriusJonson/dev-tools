use std::{
    collections::HashMap,
    slice::{Iter, IterMut},
};

use leptos::{
    leptos_dom::logging::console_log,
    prelude::{GetUntracked, ReadSignal, RwSignal, WriteSignal},
};
use uuid::Uuid;

use crate::{
    components::layout::property_editor::KeyValueTableItem,
    domain::rest_client::util::{
        request_store::{RequestFieldKind, get_stored_value},
        rest_client_utils::KeyValueVector,
    },
};

#[derive(Clone, Debug)]
pub struct RequestBodyFormValue {
    pub id: String,
    pub name: RwSignal<String>,
    pub value: RwSignal<String>,
}

#[derive(Clone, Debug)]
pub struct RequestBodyFormValues {
    inner: Vec<RequestBodyFormValue>,
}

impl RequestBodyFormValues {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, header: RequestBodyFormValue) {
        self.inner.push(header)
    }

    pub fn iter(&self) -> Iter<'_, RequestBodyFormValue> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, RequestBodyFormValue> {
        self.inner.iter_mut()
    }

    pub fn remove_by_id(&mut self, id: String) {
        self.inner.retain(|h| h.id != id);
    }

    pub fn vec_owned(&self) -> Vec<RequestBodyFormValue> {
        self.inner.clone()
    }

    pub fn read_from_store(project_id: &str, request_id: i32) -> RequestBodyFormValues {
        let stored_value = get_stored_value(
            RequestFieldKind::BodyFormencoded,
            "".to_owned(),
            project_id,
            request_id,
        );
        if stored_value.is_empty() {
            return RequestBodyFormValues::new();
        }

        let mut result = RequestBodyFormValues::new();
        match serde_json::from_str::<KeyValueVector>(&stored_value) {
            Ok(values) => {
                for value in values.iter() {
                    result.push(RequestBodyFormValue::new(value.0.to_owned(), value.1.to_owned()));
                }
            }
            Err(err) => console_log(&format!("Error: {}", err)),
        }
        result
    }

    pub fn to_urlencoded_string(&self) -> Result<String, serde_urlencoded::ser::Error> {
        let map: HashMap<String, String> =
            self.iter().map(|fv| (fv.name.get_untracked(), fv.value.get_untracked())).collect();

        serde_urlencoded::to_string(&map)
    }

    pub fn to_json(&self) -> String {
        let list: KeyValueVector =
            self.iter().map(|h| (h.name.get_untracked(), h.value.get_untracked())).collect();

        serde_json::to_string(&list).unwrap()
    }
}

impl ToString for RequestBodyFormValues {
    fn to_string(&self) -> String {
        self.to_json()
    }
}

impl RequestBodyFormValue {
    pub fn new(name: String, value: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: RwSignal::new(name),
            value: RwSignal::new(value),
        }
    }
}

impl KeyValueTableItem for RequestBodyFormValue {
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
