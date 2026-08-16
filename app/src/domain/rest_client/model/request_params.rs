use std::{convert::Infallible, fmt::Display, str::FromStr};

use leptos::prelude::{Effect, Get, GetUntracked, ReadUntracked, RwSignal, Set};
use serde::{Deserialize, Serialize};

use crate::domain::rest_client::{
    model::{
        request_body_form::RequestBodyFormValues, request_header::RequestHeaders,
        rest_client_context::RestClientContext,
    },
    util::request_store::{RequestFieldKind, get_stored_value, set_stored_value},
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
    pub fn new(rc_context: RestClientContext) -> Self {
        Self {
            url: create_signal("".to_owned(), RequestFieldKind::Url, rc_context.clone()),
            method: create_signal("".to_owned(), RequestFieldKind::Method, rc_context.clone()),
            params_tab_selected: create_signal(0, RequestFieldKind::ParamsTab, rc_context.clone()),
            body: create_signal("".to_owned(), RequestFieldKind::Body, rc_context.clone()),
            body_type: create_signal(
                RequestBodyKind::Text,
                RequestFieldKind::BodyType,
                rc_context.clone(),
            ),
            save_response: create_signal(false, RequestFieldKind::SaveResponse, rc_context.clone()),
            formatting: create_signal(false, RequestFieldKind::Formatting, rc_context.clone()),
            body_formencoded: create_signal(
                RequestBodyFormValues::new(),
                RequestFieldKind::BodyFormencoded,
                rc_context.clone(),
            ),
            headers: create_signal(
                RequestHeaders::new(),
                RequestFieldKind::Headers,
                rc_context.clone(),
            ),
        }
    }

    pub fn read_from_store(&self, rc_context: RestClientContext, request_id: i32) {
        self.headers
            .set(RequestHeaders::read_from_store(&rc_context.project.read_untracked(), request_id));

        self.params_tab_selected.set(
            get_stored_value(
                RequestFieldKind::ParamsTab,
                "0".to_owned(),
                &rc_context.project.read_untracked(),
                rc_context.request.read_untracked().id,
            )
            .parse()
            .unwrap_or(0),
        );

        self.body_formencoded.set(RequestBodyFormValues::read_from_store(
            &rc_context.project.read_untracked(),
            request_id,
        ));
        self.body.set(get_stored_value(
            RequestFieldKind::Body,
            "".to_owned(),
            rc_context.project.read_untracked().as_str(),
            request_id,
        ));
        self.body_type.set(
            RequestBodyKind::from_str(&get_stored_value(
                RequestFieldKind::BodyType,
                "".to_owned(),
                rc_context.project.read_untracked().as_str(),
                request_id,
            ))
            .unwrap_or_default(),
        );

        self.save_response.set(
            get_stored_value(
                RequestFieldKind::SaveResponse,
                "false".to_owned(),
                rc_context.project.read_untracked().as_str(),
                rc_context.request.read_untracked().id,
            )
            .parse::<bool>()
            .unwrap_or_default(),
        );

        self.formatting.set(
            get_stored_value(
                RequestFieldKind::Formatting,
                "true".to_owned(),
                rc_context.project.read_untracked().as_str(),
                request_id,
            )
            .parse::<bool>()
            .unwrap_or(false),
        );
    }
}

fn create_signal<T>(value: T, field: RequestFieldKind, rc_context: RestClientContext) -> RwSignal<T>
where
    T: ToString + Clone + Send + Sync + 'static,
{
    let signal = RwSignal::new(value);

    Effect::watch(
        move || signal.get(),
        move |value, _prev, _| {
            set_stored_value(
                rc_context.project.read_only(),
                rc_context.request.read_untracked().id,
                field,
                value.to_string(),
            )
        },
        false,
    );

    signal
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
