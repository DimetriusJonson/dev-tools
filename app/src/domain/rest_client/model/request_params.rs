use std::{convert::Infallible, fmt::Display, str::FromStr};

use leptos::prelude::{GetUntracked, ReadUntracked, RwSignal};
use serde::{Deserialize, Serialize};

use crate::domain::rest_client::model::{
    request_header::RequestHeaders, request_body_form::RequestBodyFormValues,
    request_params::RequestCommand::None,
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
    pub body_formencoded: RwSignal<RequestBodyFormValues>,
    pub headers: RwSignal<RequestHeaders>,
    pub save_response: RwSignal<bool>,
    pub formatting: RwSignal<bool>,
}

impl RequestParams {
    pub fn new() -> Self {
        Self {
            url: RwSignal::new("".to_owned()),
            method: RwSignal::new("".to_owned()),
            params_tab_selected: RwSignal::new(0),
            body: RwSignal::new("".to_owned()),
            body_type: RwSignal::new(RequestBodyKind::Text),
            body_formencoded: RwSignal::new(RequestBodyFormValues::new()),
            headers: RwSignal::new(RequestHeaders::new()),
            save_response: RwSignal::new(false),
            formatting: RwSignal::new(false),
        }
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
