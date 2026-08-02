use crate::common::constants::MEDIA_TYPES;
use crate::common::json_processor::format_json;
use crate::common::ui_utils::copy_to_clipboard;
use crate::common::xml_processor::format_xml;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::layout::tabs::Tabs;
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::domain::rest_client::ui::request_store::{RequestFieldKind, get_stored_value};
use crate::domain::rest_client::ui::request_params::RequestInfo;
use crate::i18n::*;
use leptos::html::Div;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ReqResultData {
    pub status_code: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

#[component]
pub fn RequestResultPanel(
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    save_response: ReadSignal<bool>,
    set_save_response: WriteSignal<bool>,
    data: ReadSignal<Option<ReqResultData>>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();

    let (formatting, set_formatting) =
        signal(get_stored_value(RequestFieldKind::Formatting, "true".to_owned(), project.read_untracked().as_str(), request_info.read_untracked().id).parse::<bool>().unwrap());

    let on_copy_click = move |_| {
        if data.read_untracked().is_some() {
            copy_to_clipboard(&data.get_untracked().unwrap().body);
            show_info(t!(i18n, rest_client_response_copied_to_clipboard_msg).to_html(), messages);
        }
    };

    let (tab_selected, set_tab_selected) = signal(0);
    let tab_body_ref = NodeRef::<Div>::new();
    let tab_headers_ref = NodeRef::<Div>::new();

    {
        move || {
            set_tab_selected.set(0);
            let (response_status, mut response_text, response_headers, resp_code_lang) =
                match data.get() {
                    Some(response) => (
                        response.status_code.to_string(),
                        response.body.to_owned(),
                        response.headers.clone(),
                        response
                            .headers
                            .iter()
                            .filter(|v| v.0.to_lowercase() == "content-type")
                            .filter_map(|v| get_media_type_code(&v.1))
                            .next()
                            .unwrap_or("html".to_owned()),
                    ),
                    None => ("".to_owned(), "".to_owned(), Vec::new(), "html".to_owned()),
                };

            if formatting.get_untracked() {
                if resp_code_lang == "xml" {
                    match format_xml(&response_text, 4) {
                        Ok(formatted_text) => response_text = formatted_text,
                        Err(err) => show_error(format!("Cant format xml: {}", err), messages),
                    }
                } else if resp_code_lang == "json" {
                    response_text = format_json(&response_text, 4);
                }
            }

            view! {
                <div class="flex-1 overflow-y-auto flex flex-col gap-4">

                    <Tabs tab_selected set_tab_selected items=move || vec![
                            (t_string!(i18n, rest_client_response_body_tab), tab_body_ref), 
                            (t_string!(i18n, rest_client_response_headers_tab), tab_headers_ref)
                        ] />

                    //Tab Content Panels
                    <div class="flex-1 overflow-y-auto flex">
                        <div node_ref=tab_body_ref class="flex flex-col gap-4 w-full">
                            <div class="flex justify-between">
                                <span class="dark:text-white">{format!("Status: {}", response_status)}</span>
                                <div class="flex">
                                    <div class="px-4 flex items-center gap-3 cursor-pointer">
                                        <input type="checkbox" id="formatting" class="h-4 w-4" bind:value=(formatting, set_formatting) prop:checked=formatting
                                            on:change=move |e| {
                                                let value = event_target_value(&e);
                                                get_stored_value(RequestFieldKind::Formatting, value, project.read_untracked().as_str(), request_info.read_untracked().id);
                                            }/>
                                        <label for="formatting" class="dark:text-white">Format</label>
                                    </div>
                                    <div class="px-4 flex items-center gap-3 cursor-pointer">
                                        <input type="checkbox" id="save-response" class="h-4 w-4" bind:value=(save_response, set_save_response) prop:checked=save_response on:change=move |_| {}/>
                                        <label for="save-response" class="dark:text-white">Save</label>
                                    </div>
                                </div>
                            </div>
                            <div class="flex-1 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
                                <CodeInner code={response_text} lang={move || resp_code_lang.to_owned()}/>
                            </div>
                            <div class="flex">
                                <Button
                                    label=move || t!(i18n, copy_to_clipboard_btn_label).to_html()
                                    class_name="w-full".to_owned()
                                    button_width=ButtonWidth::Auto
                                    loading=move || false
                                    on_click=on_copy_click
                                    disabled=move || false
                                />
                            </div>
                        </div>

                        <div node_ref=tab_headers_ref class="flex flex-col md:flex-row gap-4 pt-4 text-xs md:text-base w-full">
                            <div class="overflow-auto rounded-md border border-gray-300 dark:border-gray-700 shadow-sm w-full">
                                <div class="grid grid-cols-2 gap-4 px-4 dark:text-white" inner_html={render_headers(response_headers)}/>
                            </div>
                        </div>
                    </div>
                </div>
            }
        }
    }
}

fn get_media_type_code(media_type: &str) -> Option<String> {
    MEDIA_TYPES
        .iter()
        .filter(|v| media_type.to_uppercase().contains(&v.0.to_uppercase()))
        .map(|v| v.1.to_owned())
        .next()
}

fn render_headers(headers: Vec<(String, String)>) -> String {
    let list: Vec<String> =
        headers.iter().map(|h| format!("<div>{}</div><div>{}</div>", h.0, h.1)).collect();
    list.join("\n")
}
