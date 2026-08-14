use crate::common::constants::MEDIA_TYPES;
use crate::common::json_processor::format_json;
use crate::common::ui_utils::copy_to_clipboard;
use crate::common::xml_processor::format_xml;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::ui::button::{Button, ButtonColor, ButtonHeight, ButtonWidth};
use crate::components::ui::code_mirror_editor::CodeMirrorEditor;
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::ui::request_raw_panel::RequestRawPanel;
use crate::domain::rest_client::util::html_utils::make_absolute_links;
use crate::domain::rest_client::util::request_store::{RequestFieldKind, get_stored_value};
use crate::i18n::*;
use crate::model::restclient::rest_client_response::RestClientResponse;
use leptos::html::Div;
use leptos::prelude::*;

#[component]
pub fn RequestResultPanel(
    params: ReadSignal<RequestParams>,
    response: ReadSignal<Option<RestClientResponse>>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().unwrap();

    let (formatting, set_formatting) = signal(
        get_stored_value(
            RequestFieldKind::Formatting,
            "true".to_owned(),
            rc_context.project.read_untracked().as_str(),
            rc_context.request.read_untracked().id,
        )
        .parse::<bool>()
        .unwrap_or(false),
    );

    let on_copy_click = move |_| {
        if response.read_untracked().is_some() {
            copy_to_clipboard(&response.get_untracked().unwrap().body);
            show_info(t!(i18n, rest_client_response_copied_to_clipboard_msg).to_html(), messages);
        }
    };

    let (tab_selected, set_tab_selected) = signal(0);
    let tab_body_ref = NodeRef::<Div>::new();
    let tab_headers_ref = NodeRef::<Div>::new();
    let tab_request_raw_ref = NodeRef::<Div>::new();

    let (response_status_code, set_response_status_code) = signal("".to_owned());
    let (response_body, set_response_body) = signal("".to_owned());
    let (response_lang, set_response_lang) = signal("".to_owned());
    let (response_headers, set_response_headers) = signal(Vec::new());
    let (request_raw, set_request_raw) = signal("".to_owned());
    let (show_preview_html, set_show_preview_html) = signal(false);

    Effect::watch(
        move || response.get(),
        move |value, _prev, _| {
            set_tab_selected.set(0);
            set_show_preview_html.set(false);
            match value {
                Some(response) => {
                    if let Some(error) = &response.error {
                        show_error(error.to_owned(), messages);
                    };

                    set_response_status_code.set(response.status_code.to_string());
                    set_response_lang.set(
                        response
                            .headers
                            .iter()
                            .filter(|v| v.0.to_lowercase() == "content-type")
                            .filter_map(|v| get_media_type_code(&v.1))
                            .next()
                            .unwrap_or("html".to_owned()),
                    );
                    set_response_body.set(response.body.to_owned());
                    set_response_headers.set(response.headers.clone());
                    set_request_raw.set(response.request_raw.to_owned());
                }
                None => {
                    set_response_status_code.set("".to_owned());
                    set_response_body.set("".to_owned());
                    set_response_lang.set("".to_owned());
                    set_response_headers.set(Vec::new());
                    set_request_raw.set("".to_owned());
                }
            };

            if formatting.get_untracked() {
                if response_lang.get_untracked() == "xml" {
                    let formatted_xml = match format_xml(&response_body.read_untracked(), 4) {
                        Ok(formatted_text) => formatted_text,
                        Err(err) => {
                            show_error(format!("Cant format xml: {}", err), messages);
                            return;
                        }
                    };
                    set_response_body.set(formatted_xml);
                } else if response_lang.get_untracked() == "json" {
                    let formatted_json = format_json(&response_body.read_untracked(), 4);
                    set_response_body.set(formatted_json);
                }
            }
        },
        false,
    );

    let get_preview_src_doc = move || {
        let mut html = response_body.get_untracked();
        make_absolute_links(&mut html, &rc_context.request.read_untracked().url);
        html
    };

    view! {
        <div class="flex-1 overflow-y-auto flex flex-col gap-4">

            <Tabs tab_selected set_tab_selected items=move || vec![
                    TabItem::new_simple(t_string!(i18n, rest_client_response_body_tab), tab_body_ref),
                    TabItem::new_simple(t_string!(i18n, rest_client_response_headers_tab), tab_headers_ref),
                    TabItem::new_simple(t_string!(i18n, rest_client_response_request_raw_tab), tab_request_raw_ref)
                ] />

            //Tab Content Panels
            <div class="flex-1 overflow-y-auto flex">
                <div node_ref=tab_body_ref class="flex flex-col gap-4 w-full">
                    <div class="flex justify-between">
                        <span class="dark:text-white">{move || format!("Status: {}", response_status_code.get())}</span>
                        <div class="flex">
                            <div class="px-4 flex items-center gap-3 cursor-pointer">
                                <input type="checkbox" id="formatting" class="h-4 w-4" bind:value=(formatting, set_formatting) prop:checked=formatting
                                    on:change=move |e| {
                                        let value = event_target_value(&e);
                                        get_stored_value(RequestFieldKind::Formatting, value, rc_context.project.read_untracked().as_str(), rc_context.request.read_untracked().id);
                                    }/>
                                <label for="formatting" class="dark:text-white">Format</label>
                            </div>
                            <div class="px-4 flex items-center gap-3 cursor-pointer">
                                <input type="checkbox" id="save-response" class="h-4 w-4"
                                    bind:value=(params.read_untracked().save_response, params.read_untracked().set_save_response)
                                    prop:checked=params.read_untracked().save_response
                                    on:change=move |_| {}/>
                                <label for="save-response" class="dark:text-white">Save</label>
                            </div>

                            <Button
                                label=move || "⧉".to_owned()
                                title=move || t!(i18n, copy_to_clipboard_btn_label).to_html()
                                class_name="text-bold w-8 px-2 text-gray-500 hover:text-green-500".to_owned()
                                button_width=ButtonWidth::Custom
                                button_height=ButtonHeight::Custom
                                color=ButtonColor::Custom
                                loading=move || false
                                on_click=on_copy_click
                                disabled=move || false
                        />

                        </div>
                    </div>
                    <div class="flex-1 relative flex overflow-auto w-full min-w-0"
                        class:hidden=move || response_body.read().is_empty() >
                        <CodeMirrorEditor
                            element_id="response-body-code-editor".to_owned()
                            lang=response_lang
                            value=response_body
                            set_value=set_response_body
                            read_only=true
                            hidden=Box::new(move || show_preview_html.get())
                        />

                        <Show when=move || { show_preview_html.get() }>
                            <iframe class="w-full px-1 md:px-4 py-2"
                                srcdoc=get_preview_src_doc sandbox
                            >
                            </iframe>
                        </Show>

                        <Button
                            label=move || "👁".to_owned()
                            title=move || t_string!(i18n, rest_client_response_html_preview).to_owned()
                            class_name="absolute right-8 top-2 text-bold w-8 px-2 text-gray-500 hover:text-green-500 z-50".to_owned()
                            class:hidden=move || response_lang.read().as_str() != "html"
                            button_width=ButtonWidth::Custom
                            button_height=ButtonHeight::Custom
                            color=ButtonColor::Custom
                            loading=move || false
                            disabled=move || false
                            on_click=move |_| {set_show_preview_html.set(!show_preview_html.get_untracked())}
                        />

                    </div>
                </div>

                <div node_ref=tab_headers_ref class="flex flex-col md:flex-row gap-4 pt-4 text-xs md:text-base w-full">
                    <div class="overflow-auto rounded-md border border-gray-300 dark:border-gray-700 shadow-sm w-full">
                        <div class="grid grid-cols-2 gap-4 px-4 dark:text-white" inner_html={move || render_headers(response_headers.get())}/>
                    </div>
                </div>

                <RequestRawPanel node_ref=tab_request_raw_ref request_raw=request_raw params />
            </div>
        </div>
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
