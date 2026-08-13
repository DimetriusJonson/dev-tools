use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button_world::ButtonWorld;
use crate::domain::rest_client::model::request_params::{
    RequestBodyKind, RequestCommand, RequestParams,
};
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::util::rest_client_utils::formencoded_to_str;
use crate::i18n::*;
use crate::model::restclient::rest_client_request::RestClientRequest;
use crate::model::restclient::rest_client_response::RestClientResponse;
use gloo_net::http::Request;
use leptos::html::Button;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::ui_utils::single_select_option;
use crate::components::ui::select_input::SelectInput;

use crate::components::ui::text_input::TextInput;

#[component]
pub fn RequestParamsUrl(
    params: ReadSignal<RequestParams>,
    #[prop(into)] on_result: Callback<RestClientResponse>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let rc_context = use_context::<RestClientContext>().unwrap();

    let send_btn_node_ref = NodeRef::<Button>::new();

    let (in_progress, set_in_progress) = signal(false);

    Effect::watch(
        move || rc_context.request.get(),
        move |value, _prev, _| {
            if value.command == RequestCommand::Run
                && let Some(send_btn) = send_btn_node_ref.get_untracked()
            {
                rc_context.set_request.write_untracked().command = RequestCommand::None;
                send_btn.click();
            }
        },
        false,
    );

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let params = params.read_untracked();
            let method = params.method.get_untracked();

            let mut headers = Vec::new();

            if (&method == "POST" || &method == "PUT")
                && !params.body.read_untracked().is_empty()
                && params.content_type().is_none()
            {
                headers.push((
                    "Content-Type".to_owned(),
                    params.body_type.read_untracked().content_type().to_owned(),
                ));
            }

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
                RequestBodyKind::Text | RequestBodyKind::Json | RequestBodyKind::Xml => {
                    params.body.get_untracked()
                }
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
