use std::str::FromStr;

use getset::{CopyGetters, Getters, Setters};
use leptos::prelude::{Effect, Get, GetUntracked, ReadSignal, ReadUntracked, RwSignal, Set, WriteSignal};
use serde::{Deserialize, Serialize};

use crate::domain::rest_client::{
    model::{
        request_body_form::RequestBodyFormValues, request_body_kind::RequestBodyKind,
        request_header::RequestHeaders, rest_client_context::RestClientContext,
    },
    util::request_store::{RequestFieldKind, get_stored_value, set_stored_value},
};

#[derive(Clone, Serialize, Deserialize, Getters, CopyGetters, Setters)]
pub struct RestClientProject {
    #[getset(get_copy = "pub")]
    id: i32,
    #[getset(get = "pub", set = "pub")]
    name: String,
}

impl RestClientProject {
    pub fn new(id: i32, name: String) -> Self {
        Self { id, name }
    }

    pub fn id_str(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct RequestParams {
    url: RwSignal<String>,
    method: RwSignal<String>,
    params_tab_selected: RwSignal<usize>,
    body: RwSignal<String>,
    body_type: RwSignal<RequestBodyKind>,
    body_formencoded: RwSignal<RequestBodyFormValues>,
    headers: RwSignal<RequestHeaders>,
    save_response: RwSignal<bool>,
    formatting: RwSignal<bool>,
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

    #[inline]
    pub fn url(&self) -> ReadSignal<String> {
        self.url.read_only()
    } 

    #[inline]
    pub fn set_url(&self) -> WriteSignal<String> {
        self.url.write_only()
    } 

    #[inline]
    pub fn method(&self) -> ReadSignal<String> {
        self.method.read_only()
    } 

    #[inline]
    pub fn set_method(&self) -> WriteSignal<String> {
        self.method.write_only()
    } 

    #[inline]
    pub fn body_type(&self) -> ReadSignal<RequestBodyKind> {
        self.body_type.read_only()
    } 

    #[inline]
    pub fn set_body_type(&self) -> WriteSignal<RequestBodyKind> {
        self.body_type.write_only()
    } 

    #[inline]
    pub fn body(&self) -> ReadSignal<String> {
        self.body.read_only()
    } 

    #[inline]
    pub fn set_body(&self) -> WriteSignal<String> {
        self.body.write_only()
    } 

    #[inline]
    pub fn save_response(&self) -> ReadSignal<bool> {
        self.save_response.read_only()
    } 

    #[inline]
    pub fn set_save_response(&self) -> WriteSignal<bool> {
        self.save_response.write_only()
    } 

    #[inline]
    pub fn formatting(&self) -> ReadSignal<bool> {
        self.formatting.read_only()
    } 

    #[inline]
    pub fn set_formatting(&self) -> WriteSignal<bool> {
        self.formatting.write_only()
    } 

    #[inline]
    pub fn params_tab_selected(&self) -> ReadSignal<usize> {
        self.params_tab_selected.read_only()
    } 

    #[inline]
    pub fn set_params_tab_selected(&self) -> WriteSignal<usize> {
        self.params_tab_selected.write_only()
    } 

    #[inline]
    pub fn body_formencoded(&self) -> ReadSignal<RequestBodyFormValues> {
        self.body_formencoded.read_only()
    } 

    #[inline]
    pub fn set_body_formencoded(&self) -> WriteSignal<RequestBodyFormValues> {
        self.body_formencoded.write_only()
    } 

    #[inline]
    pub fn headers(&self) -> ReadSignal<RequestHeaders> {
        self.headers.read_only()
    } 

    #[inline]
    pub fn set_headers(&self) -> WriteSignal<RequestHeaders> {
        self.headers.write_only()
    } 

    pub fn read_from_store(&self, rc_context: RestClientContext, request_id: i32) {
        self.headers
            .set(RequestHeaders::read_from_store(&rc_context.project.read_untracked(), request_id));

        self.params_tab_selected.set(
            get_stored_value(
                RequestFieldKind::ParamsTab,
                "0".to_owned(),
                &rc_context.project.read_untracked(),
                rc_context.request.read_untracked().id(),
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
                rc_context.request.read_untracked().id(),
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

    pub fn get_body(&self) -> Result<String, serde_urlencoded::ser::Error> {
        Ok(match self.body_type.get_untracked() {
            RequestBodyKind::Formencoded => {
                self.body_formencoded.get_untracked().to_urlencoded_string()?
            }
            RequestBodyKind::Text | RequestBodyKind::Json | RequestBodyKind::Xml => {
                self.body.get_untracked()
            }
        })
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
                rc_context.request.read_untracked().id(),
                field,
                &value.to_string(),
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
            .find(|h| h.name().read_untracked().as_str().to_lowercase() == "content-type")
            .map(|h| h.value().get_untracked())
    }
}
