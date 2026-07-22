use crate::common::constants::MEDIA_TYPES;
use crate::common::json_processor::format_json;
use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::copy_to_clipboard;
use crate::common::xml_processor::format_xml;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::i18n::*;
use leptos::prelude::*;

#[derive(PartialEq, Copy, Clone)]
enum ResponceTabKind {
    Body,
    Headers,
}

#[derive(Clone)]
pub struct RestClientPanelData {
    pub status_code: u16,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

#[component]
pub fn RestClientResponsePanel(data: ReadSignal<Option<RestClientPanelData>>) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();

    let (formatting, set_formatting) =
        signal(get_local_store_value("rc_formatting", "true".to_owned()).parse::<bool>().unwrap());

    let on_copy_click = move |_| {
        if data.read_untracked().is_some() {
            copy_to_clipboard(&data.get_untracked().unwrap().body);
            show_info(t!(i18n, rest_client_response_copied_to_clipboard_msg).to_html(), messages);
        }
    };

    let (resp_tab_selected, set_resp_tab_selected) = signal(ResponceTabKind::Body);
    {
        move || {
            set_resp_tab_selected.set(ResponceTabKind::Body);
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
                            .map(|v| get_media_type_code(&v.1))
                            .filter(|v| v.is_some())
                            .map(|v| v.unwrap_or("html".to_owned()))
                            .nth(0)
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
            <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[35dvh] md:h-[90dvh]">
                // Tab Headers
                <div class="flex border-b border-gray-200 text-sm font-medium text-center focus:outline-none" role="tablist">
                    <button role="tab"
                        aria-selected=move || resp_tab_selected.get() == ResponceTabKind::Body
                        class="flex-1 py-2.5 border-b-2 cursor-pointer"
                        class=(["border-blue-600", "text-black", "dark:text-white"], move || resp_tab_selected.get() == ResponceTabKind::Body)
                        class=(["text-gray-500"], move || resp_tab_selected.get() != ResponceTabKind::Body)
                        on:click=move |_event| {
                            set_resp_tab_selected.set(ResponceTabKind::Body)
                        }
                    >
                    {t!(i18n, rest_client_response_body_tab)}
                    </button>
                    <button role="tab"
                        aria-selected=move || resp_tab_selected.get() == ResponceTabKind::Headers
                        class="flex-1 py-2.5 border-b-2 cursor-pointer"
                        class=(["border-blue-600", "text-black", "dark:text-white"], move || resp_tab_selected.get() == ResponceTabKind::Headers)
                        class=(["text-gray-500"], move || resp_tab_selected.get() != ResponceTabKind::Headers)
                        on:click=move |_event| {
                            set_resp_tab_selected.set(ResponceTabKind::Headers)
                        }
                        >
                    {t!(i18n, rest_client_response_headers_tab)}
                    </button>
                </div>

                //Tab Content Panels
                <div class="md:flex-1 min-h-0 overflow-y-auto flex">
                    <div class="flex flex-col gap-4 w-full"
                        class:block=move || resp_tab_selected.get() == ResponceTabKind::Body
                        class:hidden=move || resp_tab_selected.get() != ResponceTabKind::Body
                    >
                        <div class="flex justify-between">
                            <span class="dark:text-white">{format!("Status: {}", response_status)}</span>
                            <div class="px-4 flex items-center gap-3 cursor-pointer">
                                <input type="checkbox" id="formatting" class="h-4 w-4" bind:value=(formatting, set_formatting) prop:checked=formatting
                                    on:change=move |e| {
                                        let value = event_target_value(&e);
                                        set_local_store_value("rc_formatting", value);
                                    }/>
                                <label for="formatting" class="dark:text-white">Format</label>
                            </div>
                        </div>
                        <div class="flex-1 min-h-0 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
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

                    <div class="flex flex-col md:flex-row gap-4 pt-4 text-xs md:text-base min-h-0"
                        class:block=move || resp_tab_selected.get() == ResponceTabKind::Headers
                        class:hidden=move || resp_tab_selected.get() != ResponceTabKind::Headers
                    >
                        <div class="overflow-auto rounded-md border border-gray-300 dark:border-gray-700 shadow-sm ">
                            <div class="grid grid-cols-2 gap-4 px-4 dark:text-white" inner_html={render_headers(response_headers)}/>
                        </div>
                    </div>
                </div>
            </div>
        }.into_view()
        }
    }
}

fn get_media_type_code(media_type: &str) -> Option<String> {
    MEDIA_TYPES
        .iter()
        .filter(|v| media_type.to_uppercase().contains(&v.0.to_uppercase()))
        .map(|v| v.1.to_owned())
        .nth(0)
}

fn render_headers(headers: Vec<(String, String)>) -> String {
    let list: Vec<String> =
        headers.iter().map(|h| format!("<div>{}</div><div>{}</div>", h.0, h.1)).collect();
    list.join("\n")
}
