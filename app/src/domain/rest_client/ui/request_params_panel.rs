use std::collections::HashMap;

use crate::components::layout::drag_splitter::DragSplitter;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::layout::tabs::Tabs;
use crate::domain::rest_client::ui::build_rc_req_store_key;
use crate::domain::rest_client::ui::request_body_form_panel::RequestBodyFormPanel;
use crate::domain::rest_client::ui::request_headers_panel::RequestHeadersPanel;
use crate::domain::rest_client::ui::request_params::{RequestBodyFormValue, RequestInfo, RequestParams};
use crate::domain::rest_client::ui::request_result_panel::ReqResultData;
use crate::i18n::*;
use crate::model::restclient::rest_client_request::RestClientRequest;
use crate::model::restclient::rest_client_response::RestClientResponse;
use gloo_net::http::Request;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::ui_utils::single_select_option;
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

use crate::components::ui::text_input::TextInput;

#[component]
pub fn RequestParamsPanel(
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    body_tab_selected: ReadSignal<usize>,
    set_body_tab_selected: WriteSignal<usize>,
    params: ReadSignal<RequestParams>,
    #[prop(into)] on_result: Callback<ReqResultData>,
    send_btn_node_ref: NodeRef<leptos::html::Button>,
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (in_progress, set_in_progress) = signal(false);

    let tab_body_text_ref = NodeRef::<Div>::new();
    let tab_body_form_encoded_ref = NodeRef::<Div>::new();
    let headers_ref = NodeRef::<Div>::new();

    Effect::watch(
        move || body_tab_selected.get(),
        move |value, _prev, _| match value {
            1 => params.read_untracked().set_body_type.set("formencoded".to_owned()),
            _ => params.read_untracked().set_body_type.set("text".to_owned()),
        },
        false,
    );

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let params = params.read_untracked();

            let mut headers = Vec::new();
            for header in params.headers.get_untracked() {
                headers.push((header.name.get_untracked(), header.value.get_untracked()));
            }

            let body = match params.body_type.read_untracked().as_str() {
                "formencoded" => {
                    match formencoded_to_str(params.body_formencoded.get_untracked()) {
                        Ok(url) => url,
                        Err(err) => {
                            show_error(format!("Error: {}", err), messages);
                            return;
                        }
                    }
                }
                _ => params.body.get_untracked(),
            };

            let rc_request = RestClientRequest {
                method: params.method.get_untracked(),
                url: params.url.get_untracked(),
                headers,
                body,
                insecure: params.insecure.get_untracked()
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

        <div node_ref=node_ref class="min-h-0 overflow-y-auto flex flex-col gap-2 md:gap-4">
            <div class="flex flex-col sm:flex-row gap-2">
                <SelectInput
                    name="method".to_owned()
                    label=move || "Method".to_owned()
                    class_name="max-w-12 md:max-w-24".to_owned()
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
                    on_change=move |_| {}
                />

                <div class="px-0 flex items-center gap-0 cursor-pointer">
                    <input title=move || t_string!(i18n, rest_client_insecure_title) type="checkbox" id="insecure" class="h-4 w-4" 
                        bind:value=(params.read_untracked().insecure, params.read_untracked().set_insecure) prop:checked=params.read_untracked().insecure 
                        on:change=move |_| {}/>
                    <label for="insecure" class="dark:text-white">{"⚠️"}</label>
                </div>

                <Button node_ref=send_btn_node_ref
                    label=move || t!(i18n, rest_client_send_btn_label).to_html()
                    button_width=ButtonWidth::Lg
                    loading=move || in_progress.get()
                    on_click=on_send_click
                    disabled=move || in_progress.get()
                />


            </div>

            <div class="flex-1 flex flex-col">
                <div node_ref=headers_ref class="flex flex-col overflow-y-auto gap-y-4 ">
                    <RequestHeadersPanel params />
                </div>

                <DragSplitter 
                    class_name="hidden md:block".to_owned()
                    target_ref=headers_ref 
                    horizontal=true
                    local_store_prop_name=move || build_rc_req_store_key(project.read_untracked().as_str(), request_info.read_untracked().id, "headers_height")
                    min_scr_ration={1.0 / 6.0} 
                    max_scr_ration={2.0 / 3.0}
                    default_scr_ration={1.0 / 6.0} 
                    allow_mobile=true
                />

                <div class="flex-1 flex flex-col">
                    <Tabs class_name="".to_owned()
                        tab_selected=body_tab_selected set_tab_selected=set_body_tab_selected
                        items=move || vec![
                            ("Text", tab_body_text_ref),
                            ("Form Encoded", tab_body_form_encoded_ref),
                        ] />

                    <div node_ref=tab_body_text_ref class="flex-1 flex overflow-y-auto">
                        <TextArea
                            name="body".to_owned()
                            class_name="w-full resize-none".to_owned()
                            placeholder=move || {t!(i18n, rest_client_body_placeholder).to_html()}
                            value=params.read_untracked().body
                            set_value=params.read_untracked().set_body
                            on_change=move |_| {}
                        />
                    </div>

                    <div node_ref=tab_body_form_encoded_ref class="flex-1 flex flex-col overflow-y-auto pt-4 gap-4">
                        <RequestBodyFormPanel params/>
                    </div>
                </div>
            </div>
        </div>

    }
}

fn formencoded_to_str(form_values: Vec<RequestBodyFormValue>) -> Result<String, Error> {
    let map: HashMap<String, String> = form_values
        .into_iter()
        .map(|fv| (fv.name.get_untracked(), fv.value.get_untracked()))
        .collect();

    Ok(serde_urlencoded::to_string(&map)?)
}
