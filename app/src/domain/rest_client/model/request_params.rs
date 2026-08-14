use std::{
    convert::Infallible,
    fmt::Display,
    slice::{Iter, IterMut},
    str::FromStr,
};

use leptos::prelude::{GetUntracked, ReadSignal, ReadUntracked, RwSignal, WriteSignal, signal};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    components::layout::property_editor::KeyValueTableItem,
    domain::rest_client::{
        model::request_params::RequestCommand::None,
        util::request_store::{RequestFieldKind, get_stored_value, set_stored_value},
    },
};

#[derive(Clone, Serialize, Deserialize)]
pub struct RestClientProject {
    pub id: i32,
    pub name: String,
}

#[derive(PartialEq, Eq, Clone, Default)]
pub enum RequestBodyKind {
    #[default]
    Text,
    Json,
    Xml,
    Formencoded,
}

#[derive(Clone, Debug)]
pub struct RequestParams {
    pub url: RwSignal<String>,
    pub method: RwSignal<String>,
    pub params_tab_selected: RwSignal<usize>,
    pub body: RwSignal<String>,
    pub body_type: RwSignal<RequestBodyKind>,
    pub body_formencoded: RwSignal<Vec<RequestBodyFormValue>>,
    pub headers: RwSignal<CustomHeaders>,
    pub save_response: RwSignal<bool>,
    pub formatting: RwSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct RequestBodyFormValue {
    pub id: String,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

impl RequestParams {
    pub fn new() -> Self {
        Self {
            url: RwSignal::new("".to_owned()),
            method: RwSignal::new("".to_owned()),
            params_tab_selected: RwSignal::new(0),
            body: RwSignal::new("".to_owned()),
            body_type: RwSignal::new(RequestBodyKind::Text),
            body_formencoded: RwSignal::new(Vec::new()),
            headers: RwSignal::new(CustomHeaders::new()),
            save_response: RwSignal::new(false),
            formatting: RwSignal::new(false),
        }
    }
}

impl KeyValueTableItem for RequestBodyFormValue {
    fn id(&self) -> String {
        self.id.to_string()
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

#[derive(Clone, Debug, PartialEq)]
pub enum RequestCommand {
    None,
    Run,
    CopyCUrl,
    CopyCUrlWin,
}

#[derive(Clone, Debug)]
pub struct RequestInfo {
    pub id: i32,
    pub project_id: i32,
    pub url: String,
    pub name: String,
    pub method: String,
    pub command: RequestCommand,
}

#[derive(Clone, Debug)]
pub struct CustomHeaders {
    inner: Vec<CustomHeader>,
}

impl CustomHeaders {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, header: CustomHeader) {
        self.inner.push(header)
    }

    pub fn iter(&self) -> Iter<'_, CustomHeader> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, CustomHeader> {
        self.inner.iter_mut()
    }

    pub fn remove_by_id(&mut self, id: String) {
        self.inner.retain(|h| h.id != id);
    }

    pub fn vec_owned(&self) -> Vec<CustomHeader> {
        self.inner.clone()
    }

    pub fn write_to_store(&self, project: ReadSignal<String>, request_id: i32) {
        let s = self
            .inner
            .iter()
            .map(|h| format!("{}:{}", h.name.get_untracked(), h.value.get_untracked()))
            .collect::<Vec<String>>()
            .join("\n");

        set_stored_value(project, request_id, RequestFieldKind::Headers, s);
    }

    pub fn read_from_store(project_id: &str, request_id: i32) -> Self {
        let stored_value =
            get_stored_value(RequestFieldKind::Headers, "".to_owned(), project_id, request_id);
        if stored_value.is_empty() {
            return CustomHeaders::new();
        }

        let mut result = CustomHeaders::new();
        for line in stored_value.lines() {
            if let Some(index) = line.find(":") {
                let (name, set_name) = signal(line[..index].to_owned());
                let (value, set_value) = signal(line[index + 1..].to_owned());

                let header = CustomHeader {
                    id: Uuid::new_v4().to_string(),
                    name,
                    set_name,
                    value,
                    set_value,
                };
                result.push(header);
            }
        }

        result
    }
}

#[derive(Clone, Debug)]
pub struct CustomHeader {
    pub id: String,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
}

impl KeyValueTableItem for CustomHeader {
    fn id(&self) -> String {
        self.id.to_string()
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

impl RequestParams {
    pub fn content_type(&self) -> Option<String> {
        self.headers
            .read_untracked()
            .iter()
            .find(|h| h.name.read_untracked().as_str().to_lowercase() == "content-type")
            .map(|h| h.value.get_untracked())
    }
}

impl RequestInfo {
    pub fn new(id: i32, project_id: i32, url: String, name: String, method: String) -> Self {
        Self { id, project_id, url, method, name, command: None }
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            project_id: 0,
            url: "".to_owned(),
            method: "".to_owned(),
            name: "".to_owned(),
            command: None,
        }
    }

    pub fn display_name(&self) -> String {
        if !self.name.is_empty() { self.name.to_owned() } else { self.url.to_owned() }
    }
}

impl RequestBodyKind {
    pub fn content_type(&self) -> &str {
        match self {
            RequestBodyKind::Text => "text/plain",
            RequestBodyKind::Json => "application/json",
            RequestBodyKind::Xml => "application/xml",
            RequestBodyKind::Formencoded => "application/x-www-form-urlencoded",
        }
    }

    pub fn from_content_type(content_type: &str) -> Self {
        match content_type.to_lowercase().trim() {
            "text/plain" => RequestBodyKind::Text,
            "application/xml" => RequestBodyKind::Xml,
            "application/x-www-form-urlencoded" => RequestBodyKind::Formencoded,
            _ => RequestBodyKind::Json,
        }
    }
}

impl Display for RequestBodyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestBodyKind::Text => write!(f, "text"),
            RequestBodyKind::Formencoded => write!(f, "formencoded"),
            RequestBodyKind::Json => write!(f, "json"),
            RequestBodyKind::Xml => write!(f, "xml"),
        }
    }
}

impl FromStr for RequestBodyKind {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "formencoded" {
            Ok(RequestBodyKind::Formencoded)
        } else if s == "json" {
            Ok(RequestBodyKind::Json)
        } else if s == "xml" {
            Ok(RequestBodyKind::Xml)
        } else {
            Ok(RequestBodyKind::Text)
        }
    }
}
