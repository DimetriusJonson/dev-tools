use crate::components::layout::property_editor::PropertyEditor;
use crate::domain::rest_client::model::request_body_form::RequestBodyFormValue;
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn RequestBodyFormPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <PropertyEditor
            key_label=move || t_display!(i18n, rest_client_param_name_placeholder).to_string()
            value_label=move || t_display!(i18n, rest_client_param_value_placeholder).to_string()
            items=move || params.read_untracked().body_formencoded.get().vec_owned()
            on_add=move |v:(String, String)| {
                let name_converted = v.0.to_lowercase();
                if !name_converted.is_empty() && params.read_untracked().body_formencoded.read_untracked().iter().find(|fv|fv.name.read_untracked().to_lowercase() == name_converted).is_none() {
                    params.read_untracked().body_formencoded.write().push(RequestBodyFormValue::new(v.0, v.1));
                }
            }
            on_delete=move |id:String| {
                params.read_untracked().body_formencoded.write().remove_by_id(id);
            }
            on_change_key=move |v: (String, String)| {
                params.read_untracked().body_formencoded.write().iter_mut()
                    .filter(|fv|fv.id == v.0)
                    .for_each(|fv| {fv.name.set(v.1.to_owned())});
            }
            on_change_value=move |v: (String, String)| {
                params.read_untracked().body_formencoded.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.value.set(v.1.to_owned())});
            }
        />
    }
}
