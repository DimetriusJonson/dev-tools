use crate::components::layout::message_banner::{Messages, show_error, show_warning};
use crate::components::ui::button_world::ButtonWorld;
use crate::domain::rest_client::model::request_body_kind::RequestBodyKind;
use crate::domain::rest_client::model::request_info::RequestCommand;
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::util::html_previewer::clear_html_previewer;
use crate::i18n::*;
use crate::model::restclient::rest_client_request::RestClientRequest;
use crate::model::restclient::rest_client_response::RestClientResponse;
use gloo_net::http::Request;
use leptos::html::Button;
use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::AbortController;

use crate::common::ui_utils::single_select_option;
use crate::components::ui::select_input::SelectInput;

use crate::components::ui::text_input::TextInput;

#[component]
pub fn RequestParamsUrl(
    params: ReadSignal<RequestParams>,
    set_response: WriteSignal<Option<RestClientResponse>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let rc_context = use_context::<RestClientContext>().expect("Failed get rc_context");

    let send_btn_node_ref = NodeRef::<Button>::new();

    let (in_progress, set_in_progress) = signal(false);
    let cancel_signal = RwSignal::<Option<AbortController>>::new(None);

    Effect::watch(
        move || rc_context.request.get(),
        move |value, _prev, _| {
            if let Some(cancel_controller) = cancel_signal.get_untracked() {
                cancel_controller.abort();
                return;
            }

            if value.command == RequestCommand::Run
                && let Some(send_btn) = send_btn_node_ref.get_untracked()
            {
                rc_context.request.write_untracked().command = RequestCommand::None;
                send_btn.click();
            }
        },
        false,
    );

    let cancelled_msg_memo =
        Memo::new(move |_| t_string!(i18n, rest_client_request_cancelled).to_owned());

    let on_send_click = move |_| {
        if let Some(cancel_controller) = cancel_signal.get_untracked() {
            cancel_controller.abort();
            return;
        }

        spawn_local(async move {
            set_in_progress.set(true);
            clear_html_previewer();

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

            for header in params.headers.get_untracked().iter() {
                headers.push((header.name.get_untracked(), header.value.get_untracked()));
            }

            let body = match params.body_type.get_untracked() {
                RequestBodyKind::Formencoded => {
                    match params.body_formencoded.get_untracked().to_urlencoded_string() {
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

            match AbortController::new() {
                Ok(abort_controller) => {
                    cancel_signal.set(Some(abort_controller.clone()));
                    match Request::post("/rest_client_send")
                        .abort_signal(Some(&abort_controller.signal()))
                        .json(&rc_request)
                    {
                        Ok(request) => match request.send().await {
                            Ok(response) => match response.json::<RestClientResponse>().await {
                                Ok(resp) => {
                                    set_response.set(Some(resp));
                                }
                                Err(err) => {
                                    show_error(format!("Cant get response: {}", err), messages)
                                }
                            },
                            Err(err) => {
                                let error_str = err.to_string();
                                if error_str.contains("AbortError") {
                                    show_warning(cancelled_msg_memo.get_untracked(), messages)
                                } else {
                                    show_error(format!("Failed send request: {}", err), messages)
                                }
                            }
                        },
                        Err(err) => show_error(format!("Failed build request: {}", err), messages),
                    }
                }
                Err(err) => show_error(
                    format!(
                        "Failed build request: {}",
                        err.as_string().unwrap_or_else(|| "Unknown JS error".into())
                    ),
                    messages,
                ),
            }

            set_in_progress.set(false);
            cancel_signal.set(None);
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
                value=params.read_untracked().method.read_only()
                set_value=params.read_untracked().method.write_only()
            />

            <TextInput
                name="url".to_owned()
                input_type="text".to_owned()
                class_name="w-full".to_owned()
                placeholder=move || {t_string!(i18n, rest_client_url_placeholder).to_owned()}
                value=params.read_untracked().url.read_only()
                set_value=params.read_untracked().url.write_only()
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
                disabled=move || false
            />

        </div>
    }
}
