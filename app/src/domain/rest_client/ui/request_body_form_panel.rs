use crate::components::layout::property_editor::PropertyEditor;
use crate::domain::rest_client::model::request_params::{RequestBodyFormValue, RequestInfo, RequestParams};
use crate::domain::rest_client::util::request_store::{RequestFieldKind, get_stored_value, set_stored_value};
use crate::i18n::*;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn RequestBodyFormPanel(
    params: ReadSignal<RequestParams>,
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
) -> impl IntoView {
    let i18n = use_i18n();

    Effect::watch(
        move || params.read_untracked().body_formencoded.get(),
        move |value, _prev, _| {
            set_stored_value(
                project,
                request_info.read_untracked().id,
                RequestFieldKind::BodyFormencoded,
                body_form_to_string(value),
            )
        },
        false,
    );

    Effect::watch(
        move || request_info.get(),
        move |value, prev, _| {
            let id = value.id;
            let project_id = project.get_untracked();
            if prev.is_none()
                || id != prev.unwrap().id
                || project_id.parse::<i32>().unwrap_or(0) != prev.unwrap().project_id
            {
                params
                    .read_untracked()
                    .set_body_formencoded
                    .set(load_body_formencoded(&project_id, id));
            }
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

                    params.read_untracked().set_body_formencoded.write().push(RequestBodyFormValue{ id, name, set_name, value, set_value });
                }
            }
            on_delete=move |id:String| {
                params.read_untracked().set_body_formencoded.write().retain(|fv| fv.id != id);
            }
            on_change_key=move |v: (String, String)| {
                params.read_untracked().set_body_formencoded.write().iter_mut()
                    .filter(|fv|fv.id == v.0)
                    .for_each(|fv| {fv.set_name.set(v.1.to_owned())});
            }
            on_change_value=move |v: (String, String)| {
                params.read_untracked().set_body_formencoded.write().iter_mut()
                    .filter(|h|h.id == v.0)
                    .for_each(|h| {h.set_value.set(v.1.to_owned())});
            }
        />
    }
}

fn body_form_to_string(form_values: &[RequestBodyFormValue]) -> String {
    let list: KeyValueVector =
        form_values.iter().map(|h| (h.name.get_untracked(), h.value.get_untracked())).collect();

    serde_json::to_string(&list).unwrap()
}

fn load_body_formencoded(project_id: &str, request_id: i32) -> Vec<RequestBodyFormValue> {
    let stored_value =
        get_stored_value(RequestFieldKind::BodyFormencoded, "".to_owned(), project_id, request_id);
    if stored_value.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    match serde_json::from_str::<KeyValueVector>(&stored_value) {
        Ok(values) => {
            for value in values.iter() {
                let (name, set_name) = signal(value.0.to_owned());
                let (value, set_value) = signal(value.1.to_owned());

                let header = RequestBodyFormValue {
                    id: Uuid::new_v4().to_string(),
                    name,
                    set_name,
                    value,
                    set_value,
                };
                result.push(header);
            }
        }
        Err(err) => console_log(&format!("Error: {}", err)),
    }
    result
}

type KeyValueVector = Vec<(String, String)>;
