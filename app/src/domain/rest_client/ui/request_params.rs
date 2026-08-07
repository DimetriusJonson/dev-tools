use leptos::prelude::{ReadSignal, WriteSignal};
use serde::{Deserialize, Serialize};

use crate::components::layout::property_editor::KeyValueTableItem;

#[derive(Clone, Serialize, Deserialize)]
pub struct RestClientProject {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct RequestParams {
    pub url: ReadSignal<String>,
    pub set_url: WriteSignal<String>,
    pub method: ReadSignal<String>,
    pub set_method: WriteSignal<String>,
    pub body: ReadSignal<String>,
    pub set_body: WriteSignal<String>,
    pub body_type: ReadSignal<String>,
    pub set_body_type: WriteSignal<String>,
    pub body_formencoded: ReadSignal<Vec<RequestBodyFormValue>>,
    pub set_body_formencoded: WriteSignal<Vec<RequestBodyFormValue>>,
    pub headers: ReadSignal<Vec<CustomHeader>>,
    pub set_headers: WriteSignal<Vec<CustomHeader>>,
}

#[derive(Clone, Debug)]
pub struct RequestBodyFormValue {
    pub id: usize,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

impl KeyValueTableItem for RequestBodyFormValue {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> ReadSignal<String> {
        self.name
    }

    fn set_name(&self) -> WriteSignal<String> {
        self.set_name
    }

    fn value(&self) -> ReadSignal<String> {
        self.value
    }

    fn set_value(&self) -> WriteSignal<String> {
        self.set_value
    }
}


#[derive(Clone, Debug)]
pub struct RequestInfo {
    pub id: i32,
    pub project_id: i32,
    pub url: String,
    pub name: String,
    pub method: String,
    pub autorun: bool,
}

#[derive(Clone, Debug)]
pub struct CustomHeader {
    pub id: usize,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

impl KeyValueTableItem for CustomHeader {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> ReadSignal<String> {
        self.name
    }

    fn set_name(&self) -> WriteSignal<String> {
        self.set_name
    }

    fn value(&self) -> ReadSignal<String> {
        self.value
    }

    fn set_value(&self) -> WriteSignal<String> {
        self.set_value
    }
}

impl RequestInfo {
    pub fn new(id: i32, project_id: i32, url: String, name: String, method: String) -> Self {
        Self { id, project_id, url, method, name, autorun: false }
    }

    pub fn clone_and_run(&self) -> Self {
        let mut info = self.clone();
        info.autorun = true;
        info
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            project_id: 0,
            url: "".to_owned(),
            method: "".to_owned(),
            name: "".to_owned(),
            autorun: false,
        }
    }

    pub fn display_name(&self) -> String {
        if !self.name.is_empty() { self.name.to_owned() } else { self.url.to_owned() }
    }
}
