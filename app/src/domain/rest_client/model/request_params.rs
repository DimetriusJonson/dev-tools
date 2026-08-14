use std::{convert::Infallible, fmt::Display, str::FromStr};

use leptos::prelude::{GetUntracked, ReadSignal, ReadUntracked, WriteSignal};
use serde::{Deserialize, Serialize};

use crate::{components::layout::property_editor::KeyValueTableItem, domain::rest_client::model::request_params::RequestCommand::None};

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
    pub url: ReadSignal<String>,
    pub set_url: WriteSignal<String>,
    pub method: ReadSignal<String>,
    pub set_method: WriteSignal<String>,
    pub params_tab_selected: ReadSignal<usize>,
    pub set_params_tab_selected: WriteSignal<usize>,
    pub body: ReadSignal<String>,
    pub set_body: WriteSignal<String>,
    pub body_type: ReadSignal<RequestBodyKind>,
    pub set_body_type: WriteSignal<RequestBodyKind>,
    pub body_formencoded: ReadSignal<Vec<RequestBodyFormValue>>,
    pub set_body_formencoded: WriteSignal<Vec<RequestBodyFormValue>>,
    pub headers: ReadSignal<Vec<CustomHeader>>,
    pub set_headers: WriteSignal<Vec<CustomHeader>>,
    pub save_response: ReadSignal<bool>,
    pub set_save_response: WriteSignal<bool>,
    pub formatting: ReadSignal<bool>,
    pub set_formatting: WriteSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct RequestBodyFormValue {
    pub id: String,
    pub name: ReadSignal<String>,
    pub set_name: WriteSignal<String>,
    pub value: ReadSignal<String>,
    pub set_value: WriteSignal<String>,
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
    None, Run, CopyCUrl, CopyCUrlWin,
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
            _ => RequestBodyKind::Json
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
