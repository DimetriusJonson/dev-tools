use crate::components::layout::key_value_table::KeyValueTable;
use crate::domain::rest_client::ui::request_params::{RequestBodyFormValue, RequestParams};
use crate::i18n::*;
use leptos::prelude::*;

#[component]
pub fn RequestBodyFormPanel(params: ReadSignal<RequestParams>) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <KeyValueTable
            key_label=move || t_display!(i18n, rest_client_param_name_placeholder).to_string()
            value_label=move || t_display!(i18n, rest_client_param_value_placeholder).to_string()
            items=move || params.read_untracked().body_formencoded.get()
            on_add=move |v:(String, String)| {
                let name_converted = v.0.to_lowercase();
                if !name_converted.is_empty() && params.read_untracked().body_formencoded.read_untracked().iter().find(|fv|fv.name.read_untracked().to_lowercase() == name_converted).is_none() {

                    let id = params.read_untracked().body_formencoded.read_untracked().iter().map(|fv|fv.id).max().unwrap_or_default() + 1;
                    let (name, set_name) = signal(v.0);
                    let (value, set_value) = signal(v.1);

                    params.read_untracked().set_body_formencoded.write().push(RequestBodyFormValue{ id, name, set_name, value, set_value });
                }
            }
            on_delete=move |id| {
                params.read_untracked().set_body_formencoded.write().retain(|fv| fv.id != id);
            }
            on_change_key=move |v: (usize, String)| {
                params.read_untracked().set_body_formencoded.write().iter_mut()
                    .filter(|fv|fv.id == v.0)
                    .for_each(|fv| {fv.set_name.set(v.1.to_owned())});
            }
            on_change_value=move |v: (usize, String)| {
                params.read_untracked().set_body_formencoded.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_value.set(v.1.to_owned())});
            }
        />
    }
}
