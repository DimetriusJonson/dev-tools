use crate::components::layout::message_banner::{Messages, show_error};
use crate::domain::rest_client::ui::req_params::{RequestParamKind, RequestParams};
use crate::domain::rest_client::ui::req_params_headers_panel::ReqParamsHeadersPanel;
use crate::domain::rest_client::ui::req_result_panel::ReqResultData;
use crate::i18n::*;
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::ui_utils::single_select_option;
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

use crate::components::ui::text_input::TextInput;
use crate::model::rest_client_request::RestClientRequest;
use crate::model::rest_client_response::RestClientResponse;

#[component]
pub fn ReqParamsPanel(
    params: ReadSignal<RequestParams>,
    #[prop(into)] on_result: Callback<ReqResultData>,
    #[prop(into)] on_change: Callback<RequestParamKind>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (in_progress, set_in_progress) = signal(false);

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let params = params.read_untracked();

            let mut headers = Vec::new();
            if !params.content_type.read_untracked().is_empty() {
                headers.push(("content-type".to_owned(), params.content_type.get_untracked()));
            }
            if !params.accept.read_untracked().is_empty() {
                headers.push(("accept".to_owned(), params.accept.get_untracked()));
            }
            if !params.accept_lang.read_untracked().is_empty() {
                headers.push(("accept_language".to_owned(), params.accept_lang.get_untracked()));
            }
            if !params.user_agent.read_untracked().is_empty() {
                headers.push(("user-agent".to_owned(), params.user_agent.get_untracked()));
            }

            for custom_header in params.custom_headers.get_untracked() {
                headers.push((
                    custom_header.name.get_untracked(),
                    custom_header.value.get_untracked(),
                ));
            }

            let rc_request = RestClientRequest {
                method: params.method.get_untracked(),
                url: params.url.get_untracked(),
                headers,
                body: params.body.get_untracked(),
            };

            match Request::post("/rest_client_send").json(&rc_request) {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.json::<RestClientResponse>().await {
                        Ok(resp) => {
                            on_result.run(ReqResultData {
                                status_code: resp.status_code,
                                headers: resp.headers,
                                body: resp.body,
                            });
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

        <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[29dvh] md:h-[90dvh]">
            <div class="flex gap-4">
                <SelectInput
                    name="method".to_owned()
                    label=move || "Method".to_owned()
                    class_name="max-w-24".to_owned()
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
                    on_change=move |_| {
                        on_change.run(RequestParamKind::Method);
                    }
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
                    on_change=move |_| {
                        on_change.run(RequestParamKind::Url);
                    }
                />

                <Button
                    label=move || t!(i18n, rest_client_send_btn_label).to_html()
                    button_width=ButtonWidth::Lg
                    loading=move || in_progress.get()
                    on_click=on_send_click
                    disabled=move || in_progress.get()
                />


            </div>

            <ReqParamsHeadersPanel params 
                on_change=move |kind| on_change.run(kind)
            />

            <div class="flex-1 flex">
                <TextArea
                    name="body".to_owned()
                    class_name="md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                    placeholder=move || {t!(i18n, rest_client_body_placeholder).to_html()}
                    value=params.read_untracked().body
                    set_value=params.read_untracked().set_body
                    on_change=move |_| {
                        on_change.run(RequestParamKind::Body);
                    }
                />
            </div>
        </div>

    }
}
