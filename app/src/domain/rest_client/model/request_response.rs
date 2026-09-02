use leptos::prelude::{Effect, Get, ReadSignal, ReadUntracked, RwSignal, Set, WriteSignal};

use crate::{
    domain::rest_client::{
        model::{request_params::RequestParams, rest_client_context::RestClientContext}, util::request_store::{RequestFieldKind, delete_stored_value, get_stored_value, set_stored_value},
    }, model::restclient::rest_client_response::RestClientResponse,
};

#[derive(Clone, Debug)]
pub struct RequestResponse {
    inner: RwSignal<Option<RestClientResponse>>,
}

impl RequestResponse {
    pub fn new(params: ReadSignal<RequestParams>, rc_context: RestClientContext) -> Self {
        Self { inner: Self::create_signal(params, rc_context) }
    }

    fn create_signal(
        params: ReadSignal<RequestParams>,
        rc_context: RestClientContext,
    ) -> RwSignal<Option<RestClientResponse>> {
        let signal = RwSignal::new(None);

        Effect::watch(
            move || signal.get(),
            move |value, _prev, _| {
                if *params.read_untracked().save_response.read_untracked()
                    && let Some(response) = value
                {
                    let json_string = serde_json::to_string(&response).unwrap_or("".to_owned());
                    set_stored_value(
                        rc_context.project.read_only(),
                        rc_context.request.read_untracked().id,
                        RequestFieldKind::SaveResponseData,
                        json_string,
                    )
                } else {
                    delete_stored_value(
                        rc_context.project.read_only(),
                        rc_context.request.read_only(),
                        RequestFieldKind::SaveResponseData,
                    )
                }
            },
            false,
        );

        signal
    }

    pub fn read_from_store(&self,
        rc_context: RestClientContext,
        request_id: i32,
    ) {
        let data_str = get_stored_value(
            RequestFieldKind::SaveResponseData,
            "".to_owned(),
            rc_context.project.read_untracked().as_str(),
            request_id,
        );
        if !data_str.is_empty() {
            let r = Some(serde_json::from_str::<RestClientResponse>(&data_str).unwrap_or_default());
            self.inner.set(r);
        }
    }

    pub fn clear(&self) {
        self.inner.set(None);
    }

    pub fn write_only(&self) -> WriteSignal<Option<RestClientResponse>> {
        self.inner.write_only()
    }

    pub fn read_only(&self) -> ReadSignal<Option<RestClientResponse>> {
        self.inner.read_only()
    }
}
