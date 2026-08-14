use crate::components::layout::property_editor::PropertyEditor;
use crate::domain::rest_client::model::request_params::{RequestBodyFormValue, RequestParams};
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::util::request_store::{RequestFieldKind, set_stored_value};
use crate::domain::rest_client::util::rest_client_utils::body_form_to_string;
use crate::i18n::*;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn RequestBodyFormPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().unwrap();

    Effect::watch(
        move || params.read_untracked().body_formencoded.get(),
        move |value, _prev, _| {
            set_stored_value(
                rc_context.project.read_only(),
                rc_context.request.read_untracked().id,
                RequestFieldKind::BodyFormencoded,
                body_form_to_string(value),
            )
        },
        false,
    );

    view! {
        <PropertyEditor
            key_label=move || t_display!(i18n, rest_client_param_name_placeholder).to_string()
            value_label=move || t_display!(i18n, rest_client_param_value_placeholder).to_string()
            items=move || params.read_untracked().body_formencoded.get()
            on_add=move |v:(String, String)| {
                let name_converted = v.0.to_lowercase();
                if !name_converted.is_empty() && params.read_untracked().body_formencoded.read_untracked().iter().find(|fv|fv.name.read_untracked().to_lowercase() == name_converted).is_none() {

                    let id = Uuid::new_v4().to_string();
                    let (name, set_name) = signal(v.0);
                    let (value, set_value) = signal(v.1);

                    params.read_untracked().body_formencoded.write().push(RequestBodyFormValue{ id, name, set_name, value, set_value });
                }
            }
            on_delete=move |id:String| {
                params.read_untracked().body_formencoded.write().retain(|fv| fv.id != id);
            }
            on_change_key=move |v: (String, String)| {
                params.read_untracked().body_formencoded.write().iter_mut()
                    .filter(|fv|fv.id == v.0)
                    .for_each(|fv| {fv.set_name.set(v.1.to_owned())});
            }
            on_change_value=move |v: (String, String)| {
                params.read_untracked().body_formencoded.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_value.set(v.1.to_owned())});
            }
        />
    }
}
