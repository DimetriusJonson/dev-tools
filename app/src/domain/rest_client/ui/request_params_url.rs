use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button_world::ButtonWorld;
use crate::domain::rest_client::model::request_params::{RequestBodyKind, RequestParams};
use crate::domain::rest_client::util::rest_client_utils::formencoded_to_str;
use crate::i18n::*;
use crate::model::restclient::rest_client_request::RestClientRequest;
use crate::model::restclient::rest_client_response::RestClientResponse;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::ui_utils::single_select_option;
use crate::components::ui::select_input::SelectInput;

use crate::components::ui::text_input::TextInput;

#[component]
pub fn RequestParamsUrl(
    params: ReadSignal<RequestParams>,
    #[prop(into)] on_result: Callback<RestClientResponse>,
    send_btn_node_ref: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (in_progress, set_in_progress) = signal(false);

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let params = params.read_untracked();

            let mut headers = Vec::new();
            for header in params.headers.get_untracked() {
                headers.push((header.name.get_untracked(), header.value.get_untracked()));
            }

            let body = match params.body_type.get_untracked() {
                RequestBodyKind::Formencoded => {
                    match formencoded_to_str(params.body_formencoded.get_untracked()) {
                        Ok(url) => url,
                        Err(err) => {
                            show_error(format!("Error: {}", err), messages);
                            return;
                        }
                    }
                }
                RequestBodyKind::Text => params.body.get_untracked(),
                RequestBodyKind::Json => params.body_json.get_untracked(),
            };

            let rc_request = RestClientRequest {
                method: params.method.get_untracked(),
                url: params.url.get_untracked(),
                headers,
                body,
            };

            match Request::post("/rest_client_send").json(&rc_request) {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.json::<RestClientResponse>().await {
                        Ok(resp) => {
                            on_result.run(resp);
                        }
                        Err(err) => show_error(format!("Cant get response: {}", err), messages),
                    },
                    Err(err) => show_error(format!("Failed send request: {}", err), messages),
                },
                Err(err) => show_error(format!("Failed build request: {}", err), messages),
            }

            set_in_progress.set(false);
        });
    };

    view! {
        <div class="flex flex-row gap-2">
            <SelectInput
                name="method".to_owned()
                label=move || "Method".to_owned()
                class_name="max-w-16 md:max-w-24".to_owned()
                not_selected_text=move || "".to_owned()
                options=move || {vec![
                    single_select_option("GET"),
                    single_select_option("POST"),
                    single_select_option("PUT"),
                    single_select_option("DELETE"),
                    single_select_option("PATCH"),
                    single_select_option("HEAD"),
                    single_select_option("OPTIONS"),
                    ]}
                on_change=move |_| {}
                value=params.read_untracked().method
                set_value=params.read_untracked().set_method
            />

            <TextInput
                name="url".to_owned()
                input_type="text".to_owned()
                class_name="w-full".to_owned()
                placeholder=move || {t!(i18n, rest_client_url_placeholder).to_html()}
                value=params.read_untracked().url
                set_value=params.read_untracked().set_url
                on_press_enter=move |_| {
                    if let Some(send_btn) = send_btn_node_ref.get() {
                        send_btn.click();
                    }
                }
            />

            <ButtonWorld node_ref=send_btn_node_ref
                title=move || t_string!(i18n, rest_client_send_btn_label).to_owned()
                loading=move || in_progress.get()
                on_click=on_send_click
                disabled=move || in_progress.get()
            />

        </div>
    }
}
