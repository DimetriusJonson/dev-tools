use leptos::prelude::*;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::domain::rest_client::req_params_headers_panel::{CustomHeader, ReqParamsHeadersPanel};
use crate::domain::rest_client::req_result_panel::ReqResultData;
use crate::i18n::*;
use leptos::task::spawn_local;
use gloo_net::http::Request;

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::{get_accept_language, single_select_option};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

use crate::components::ui::text_input::TextInput;
use crate::model::rest_client_request::RestClientRequest;
use crate::model::rest_client_response::RestClientResponse;

#[component]
pub fn ReqParamsPanel(
    #[prop(into)] on_result: Callback<ReqResultData>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (url, set_url) = signal(get_local_store_value("rc_url", "".to_owned()));
    let (method, set_method) = signal(get_local_store_value("rc_method", "GET".to_owned()));
    let (body, set_body) = signal(get_local_store_value("rc_body", "".to_owned()));

    let (content_type, set_content_type) =
        signal(get_local_store_value("rc_content_type", "".to_owned()));
    let (accept, set_accept) = signal(get_local_store_value("rc_accept", "".to_owned()));
    let (user_agent, set_user_agent) =
        signal(get_local_store_value("rc_user_agent", "WebDevUsefulTools Client".to_owned()));
    let (accept_lang, set_accept_lang) =
        signal(get_local_store_value("rc_accept_lang", get_accept_language()));
    let (custom_headers, set_custom_headers) = signal(Vec::<CustomHeader>::new());

    let (in_progress, set_in_progress) = signal(false);

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let mut headers = Vec::new();
            if !content_type.read_untracked().is_empty() {
                headers.push(("content-type".to_owned(), content_type.get_untracked()));
            }
            if !accept.read_untracked().is_empty() {
                headers.push(("accept".to_owned(), accept.get_untracked()));
            }
            if !accept_lang.read_untracked().is_empty() {
                headers.push(("accept_language".to_owned(), accept_lang.get_untracked()));
            }
            if !user_agent.read_untracked().is_empty() {
                headers.push(("user-agent".to_owned(), user_agent.get_untracked()));
            }

            for custom_header in custom_headers.get_untracked() {
                headers.push((
                    custom_header.name.get_untracked(),
                    custom_header.value.get_untracked(),
                ));
            }

            let rc_request = RestClientRequest {
                method: method.get_untracked(),
                url: url.get_untracked(),
                headers,
                body: body.get_untracked(),
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

    view!{

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
                    on_change=move |value| {
                        set_local_store_value("rc_method", value);
                    }
                    value=method
                    set_value=set_method
                />

                <TextInput
                    name="url".to_owned()
                    input_type="text".to_owned()
                    class_name="w-full".to_owned()
                    placeholder=move || {t!(i18n, rest_client_url_placeholder).to_html()}
                    value=url
                    set_value=set_url
                    on_change=move |_| {
                        set_local_store_value("rc_url", url.get_untracked());
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

            <ReqParamsHeadersPanel 
                content_type
                set_content_type
                accept
                set_accept
                accept_lang
                set_accept_lang
                user_agent
                set_user_agent
                custom_headers
                set_custom_headers
            />

            <div class="flex-1 flex">
                <TextArea
                    name="body".to_owned()
                    class_name="md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                    placeholder=move || {t!(i18n, rest_client_body_placeholder).to_html()}
                    value=body
                    set_value=set_body
                    on_change=move |_| {
                        set_local_store_value("rc_body", body.get_untracked());
                    }
                />
            </div>
        </div>

    }
}

