use crate::common::constants::MEDIA_TYPES;
use crate::common::json_processor::format_json;
use crate::common::ui_utils::{copy_to_clipboard, create_cookie, remove_cookie, save_file_to_disk};
use crate::common::xml_processor::format_xml;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::ui::button::{Button, ButtonColor, ButtonHeight, ButtonWidth};
use crate::components::ui::code_mirror_editor::CodeMirrorEditor;
use crate::domain::rest_client::model::request_params::RequestParams;
use crate::domain::rest_client::model::request_result::RequestResult;
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::ui::request_raw_panel::RequestRawPanel;
use crate::domain::rest_client::util::html_previewer::{add_head_base_tag, build_base_url, replace_absolute_links};
use crate::i18n::*;
use crate::model::restclient::rest_client_request::RestClientRequest;
use crate::model::restclient::rest_client_response::{RestClientResponse, RestClientResponseBody};
use gloo_net::http::Request;
use leptos::html::Div;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[derive(PartialEq, Copy, Clone)]
enum InProgressType {
    None,
    AttachmentDownload,
}

impl InProgressType {
    fn is_active(self) -> bool {
        self != InProgressType::None
    }
}

#[component]
pub fn RequestResultPanel(
    params: ReadSignal<RequestParams>,
    response: ReadSignal<Option<RestClientResponse>>,
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().unwrap();

    let on_copy_click = move |_| {
        if let Some(response) = response.get_untracked() {
            if let RestClientResponseBody::Text(body) = response.body {
                copy_to_clipboard(&body);
            }
            show_info(t!(i18n, rest_client_response_copied_to_clipboard_msg).to_html(), messages);
        }
    };

    let (in_progress, set_in_progress) = signal(InProgressType::None);
    let (proxy_allow, set_proxy_allow) = signal(true);
    let (preview_sandbox, set_preview_sandbox) = signal("");
    let (tab_selected, set_tab_selected) = signal(0);
    let tab_body_ref = NodeRef::<Div>::new();
    let tab_headers_ref = NodeRef::<Div>::new();
    let tab_request_raw_ref = NodeRef::<Div>::new();

    let request_result = RequestResult::new();
    let (show_preview_html, set_show_preview_html) = signal(false);

    Effect::new(move || {
        spawn_local(async move {
            let allow = is_proxy_allow().await;
            set_proxy_allow.set(allow);
            if allow {
                set_preview_sandbox.set("allow-scripts allow-popups allow-same-origin");
            } else {
                set_preview_sandbox.set("allow-scripts allow-popups");
            }
        });
    });

    Effect::watch(
        move || response.get(),
        move |value, _prev, _| {
            set_tab_selected.set(0);
            set_show_preview_html.set(false);
            request_result.status_code.set("".to_owned());
            request_result.body.set("".to_owned());
            request_result.lang.set("".to_owned());
            request_result.headers.set(Vec::new());
            request_result.request_raw.set("".to_owned());
            request_result.attachment.set(("".to_owned(), "".to_owned()));
            request_result.image.set("".to_owned());
            if let Some(response) = value {
                if let Some(error) = &response.error {
                    show_error(error.to_owned(), messages);
                };

                request_result.status_code.set(response.status_code.to_string());
                match &response.body {
                    RestClientResponseBody::Text(body) => {
                        request_result.lang.set(
                            response
                                .headers
                                .iter()
                                .filter(|v| v.0.to_lowercase() == "content-type")
                                .filter_map(|v| get_media_type_code(&v.1))
                                .next()
                                .unwrap_or("html".to_owned()),
                        );

                        request_result.body.set(body.to_owned())
                    }
                    RestClientResponseBody::Attachment(file_name) => {
                        request_result.attachment.set((
                            params.read_untracked().url.get_untracked(),
                            file_name.to_owned(),
                        ));
                    }
                    RestClientResponseBody::Image => {
                        request_result.image.set(params.read_untracked().url.get_untracked())
                    }
                    RestClientResponseBody::None => (),
                }

                request_result.headers.set(response.headers.clone());
                request_result.request_raw.set(response.request_raw.to_owned());
            };

            if params.read_untracked().formatting.get_untracked() {
                if request_result.lang.get_untracked() == "xml" {
                    let formatted_xml = match format_xml(&request_result.body.read_untracked(), 4) {
                        Ok(formatted_text) => formatted_text,
                        Err(err) => {
                            show_error(format!("Cant format xml: {}", err), messages);
                            return;
                        }
                    };
                    request_result.body.set(formatted_xml);
                } else if request_result.lang.get_untracked() == "json" {
                    let formatted_json = format_json(&request_result.body.read_untracked(), 4);
                    request_result.body.set(formatted_json);
                }
            }
        },
        false,
    );

    let get_preview_src_doc = move || {
        let mut html = request_result.body.get();
        if proxy_allow.get_untracked() {
            replace_absolute_links(&mut html, &rc_context.request.read_untracked().url);
            create_cookie(
                "rc_base_url",
                &build_base_url(&rc_context.request.read_untracked().url),
                30,
            );
        } else {
            add_head_base_tag(&mut html, &rc_context.request.read_untracked().url);
        }
        html
    };

    let on_attachment_download_click = move |_| {
        spawn_local(async move {
            let mut headers = Vec::new();
            for header in params.read_untracked().headers.get_untracked().iter() {
                headers.push((header.name.get_untracked(), header.value.get_untracked()));
            }

            let attachment = request_result.attachment.get_untracked();
            let rc_request = RestClientRequest {
                method: "GET".to_owned(),
                url: attachment.0,
                headers,
                body: "".to_owned(),
            };

            set_in_progress.set(InProgressType::AttachmentDownload);
            match Request::post("/rest_client_attachment_download").json(&rc_request) {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.binary().await {
                        Ok(bytes) => {
                            let file_name = attachment.1;
                            match save_file_to_disk(bytes.to_vec(), &file_name, "application/json")
                            {
                                Ok(_) => show_info(
                                    t_display!(i18n, file_saved_file_msg, file_name).to_string(),
                                    messages,
                                ),
                                Err(err) => show_error(
                                    err.as_string().unwrap_or("Error".to_owned()),
                                    messages,
                                ),
                            }
                        }
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }

            set_in_progress.set(InProgressType::None);
        });
    };

    view! {
        <div node_ref=node_ref class="overflow-y-auto flex flex-col gap-4">

            <Tabs tab_selected set_tab_selected items=move || vec![
                    TabItem::new_simple(t_string!(i18n, rest_client_response_body_tab), tab_body_ref),
                    TabItem::new_simple(t_string!(i18n, rest_client_response_headers_tab), tab_headers_ref),
                    TabItem::new_simple(t_string!(i18n, rest_client_response_request_raw_tab), tab_request_raw_ref)
                ] />

            //Tab Content Panels
            <div class="flex-1 overflow-y-auto flex">
                <div node_ref=tab_body_ref class="flex flex-col gap-4 w-full">
                    <div class="flex justify-between">
                        <span class="dark:text-white">{move || format!("Status: {}", request_result.status_code.get())}</span>
                        <div class="flex">
                            <div class="px-4 flex items-center gap-3 cursor-pointer">
                                <input type="checkbox" id="formatting" class="h-4 w-4"
                                    bind:value=(params.read_untracked().formatting, params.read_untracked().formatting)
                                    prop:checked=params.read_untracked().formatting
                                    />
                                <label for="formatting" class="dark:text-white">Format</label>
                            </div>
                            <div class="px-4 flex items-center gap-3 cursor-pointer">
                                <input type="checkbox" id="save-response" class="h-4 w-4"
                                    bind:value=(params.read_untracked().save_response, params.read_untracked().save_response)
                                    prop:checked=params.read_untracked().save_response
                                    />
                                <label for="save-response" class="dark:text-white">Save</label>
                            </div>

                            <Button
                                label=move || "👁".to_owned()
                                title=move || t_string!(i18n, rest_client_response_html_preview).to_owned()
                                class_name="w-6 dark:text-white text-black".to_owned()
                                class:hidden=move || request_result.lang.read().as_str() != "html"
                                class:border=show_preview_html
                                button_width=ButtonWidth::Custom
                                button_height=ButtonHeight::Custom
                                color=ButtonColor::Custom
                                loading=move || false
                                disabled=move || false
                                on_click=move |_| {set_show_preview_html.set(!show_preview_html.get_untracked())}
                            />

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
                    <div class="flex-1 relative flex overflow-auto w-full min-w-0">
                        <CodeMirrorEditor
                            element_id="response-body-code-editor".to_owned()
                            lang=request_result.lang.read_only()
                            value=request_result.body.read_only()
                            set_value=request_result.body.write_only()
                            read_only=true
                            hidden=Box::new(move || request_result.body.read().is_empty() || show_preview_html.get())
                        />

                        // Html preview
                        <Show when=move || { show_preview_html.get() }>
                            <iframe class="w-full"
                                srcdoc=get_preview_src_doc sandbox=preview_sandbox
                                on:load=move |_| {
                                    remove_cookie("rc_base_url", "/");
                                }
                                on:error=move |_| {
                                    remove_cookie("rc_base_url", "/");
                                }
                            >
                            </iframe>
                        </Show>

                        // Attachment
                        <Show when=move || { !request_result.attachment.read().0.is_empty() }>
                            <div class="flex-1 flex items-center justify-center">
                                <Button
                                    title=move || "".to_owned()
                                    label=move || t_display!(i18n, rc_attachment_download_btn_label, file_name = request_result.attachment.get().1).to_string()
                                    button_width=ButtonWidth::Auto
                                    loading=move || in_progress.get() == InProgressType::AttachmentDownload
                                    on_click=on_attachment_download_click
                                    disabled=move || in_progress.get().is_active()
                                />
                            </div>
                        </Show>

                        // Image
                        <Show when=move || { !request_result.image.read().is_empty() }>
                            <div class="flex-1 flex items-center justify-center gap-4">
                                <img class="flex-1" src = move || request_result.image.get() alt="Image" />
                            </div>
                        </Show>

                    </div>
                </div>

                <div node_ref=tab_headers_ref class="flex flex-col md:flex-row gap-4 pt-4 text-xs md:text-base w-full">
                    <div class="overflow-auto rounded-md border border-gray-300 dark:border-gray-700 shadow-sm w-full">
                        <div class="h-0 flex flex-col gap-4 px-4 dark:text-white whitespace-pre-wrap wrap-break-word break-all" inner_html={move || render_headers(request_result.headers.get())}/>
                    </div>
                </div>

                <RequestRawPanel node_ref=tab_request_raw_ref request_raw=request_result.request_raw.read_only() params />
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
        headers.iter().map(|h| format!("<div class=\"flex flex-row gap-4\"><div class=\"w-1/3\">{}</div><div class=\"w-full\">{}</div></div>", h.0, h.1)).collect();
    list.join("\n")
}

async fn is_proxy_allow() -> bool {
    match Request::get("/rest_client_proxy_allow").build() {
        Ok(request) => match request.send().await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(value) => value.as_bool().unwrap_or(false),
                Err(err) => {
                    console_log(&format!("Error: {}", err));
                    false
                }
            },
            Err(err) => {
                console_log(&format!("Error: {}", err));
                false
            }
        },
        Err(err) => {
            console_log(&format!("Error: {}", err));
            false
        }
    }
}
