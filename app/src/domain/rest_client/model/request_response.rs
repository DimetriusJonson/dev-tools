use leptos::{
    prelude::{Effect, Get, GetUntracked, ReadSignal, ReadUntracked, RwSignal, Set, WriteSignal}, task::spawn_local,
};
use model::restclient::rest_client_response::RestClientResponse;

use crate::{
    common::indexed_local_store::{
        get_prop_value_from_indexed_db, indexed_db_delete_key, indexed_db_save_prop_value,
    },
    domain::rest_client::model::{
        request_params::RequestParams, rest_client_context::RestClientContext,
    },
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
        let signal: RwSignal<Option<RestClientResponse>> = RwSignal::new(None);

        Effect::watch(
            move || signal.get(),
            move |value, _prev, _| {
                if *params.read_untracked().save_response().read_untracked()
                    && let Some(response) = value
                    && response.error().is_none()
                {
                    let json_string = serde_json::to_string(&response).unwrap_or("".to_owned());

                    spawn_local(async move {
                        indexed_db_save_prop_value(
                            "rc_request_response",
                            &format!(
                                "{}{}",
                                rc_context.project.get_untracked(),
                                rc_context.request.get_untracked().id()
                            ),
                            "data",
                            &json_string,
                        )
                        .await
                        .unwrap();
                    });
                } else {
                    spawn_local(async move {
                        indexed_db_delete_key(
                            "rc_request_response",
                            &format!(
                                "{}{}",
                                rc_context.project.get_untracked(),
                                rc_context.request.get_untracked().id()
                            ),
                        )
                        .await
                        .unwrap();
                    });
                }
            },
            false,
        );

        signal
    }

    pub fn read_from_store(&self, rc_context: RestClientContext, request_id: i32) {
        let inner = self.inner.clone();
        spawn_local(async move {
            let data_str = get_prop_value_from_indexed_db(
                "rc_request_response",
                &format!("{}{}", rc_context.project.get_untracked(), request_id),
                "data",
            )
            .await
            .unwrap();

            if let Some(data_str) = data_str
                && !data_str.is_empty()
            {
                let r =
                    Some(serde_json::from_str::<RestClientResponse>(&data_str).unwrap_or_default());
                inner.set(r);
            } else {
                inner.set(None);
            }
        });
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
