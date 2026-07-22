use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::constants::{HEADERS_AUTOCOMPLETE, MEDIA_TYPES_AUTOCOMPLETE};
use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::get_accept_language;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::autocomplete_input::AutocompleteInput;
use crate::components::ui::button::{Button, ButtonColor, ButtonWidth};
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

use crate::components::ui::text_input::TextInput;
use crate::domain::rest_client::rest_client_response_panel::{
    RestClientPanelData, RestClientResponsePanel,
};
use crate::i18n::*;
use crate::model::rest_client_request::RestClientRequest;
use crate::model::rest_client_response::RestClientResponse;

#[derive(PartialEq, Copy, Clone)]
enum InProgressType {
    None,
    Send,
}

impl InProgressType {
    fn is_active(self) -> bool {
        self != InProgressType::None
    }
}

#[derive(Clone, Debug)]
struct CustomHeader {
    id: usize,
    name: ReadSignal<String>,
    set_name: WriteSignal<String>,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
}

#[component]
pub fn RestClientPage() -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (content_type, set_content_type) =
        signal(get_local_store_value("rc_content_type", "".to_owned()));
    let (accept, set_accept) = signal(get_local_store_value("rc_accept", "".to_owned()));
    let (user_agent, set_user_agent) =
        signal(get_local_store_value("rc_user_agent", "WebDevUsefulTools Client".to_owned()));
    let (accept_lang, set_accept_lang) =
        signal(get_local_store_value("rc_accept_lang", get_accept_language()));

    let (custom_header_name, set_custom_header_name) = signal("".to_owned());
    let (custom_header_value, set_custom_header_value) = signal("".to_owned());
    let (custom_headers, set_custom_headers) = signal(Vec::new());

    let (url, set_url) = signal(get_local_store_value("rc_url", "".to_owned()));
    let (method, set_method) = signal(get_local_store_value("rc_method", "GET".to_owned()));
    let (body, set_body) = signal(get_local_store_value("rc_body", "".to_owned()));
    let (response, set_response) = signal(None);
    let (in_progress, set_in_progress) = signal(InProgressType::None);

    let _ = Effect::new(move || {
        set_custom_headers.set(restore_custom_headers());
    });

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Send);

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
                            set_response.set(Some(RestClientPanelData {
                                status_code: resp.status_code,
                                headers: resp.headers,
                                body: resp.body,
                            }));
                        }
                        Err(err) => show_error(format!("Cant get response: {}", err), messages),
                    },
                    Err(err) => show_error(format!("Failed send request: {}", err), messages),
                },
                Err(err) => show_error(format!("Failed build request: {}", err), messages),
            }

            set_in_progress.set(InProgressType::None);
        });
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
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
                        loading=move || in_progress.get() == InProgressType::Send
                        on_click=on_send_click
                        disabled=move || in_progress.get().is_active()
                    />


                </div>

                <div class="flex gap-4">
                    <AutocompleteInput
                        class_name="min-w-36".to_owned()
                        placeholder=move || "Content-Type".to_owned()
                        options={MEDIA_TYPES_AUTOCOMPLETE}
                        value=content_type
                        set_value=set_content_type
                        on_change=move |value| {
                            set_local_store_value("rc_content_type", value);
                        }
                    />

                    <AutocompleteInput
                        class_name="min-w-36".to_owned()
                        placeholder=move || "Accept".to_owned()
                        options={MEDIA_TYPES_AUTOCOMPLETE}
                        on_change=move |value| {
                            set_local_store_value("rc_accept", value);
                        }
                        value=accept
                        set_value=set_accept
                    />

                    <TextInput
                        name="accept-lang".to_owned()
                        input_type="text".to_owned()
                        class_name="w-max-36".to_owned()
                        placeholder=|| "Accept-Language".to_owned()
                        value=accept_lang
                        set_value=set_accept_lang
                        on_change=move |_| {
                            set_local_store_value("rc_accept_lang", accept_lang.get_untracked());
                        }
                    />
                    <TextInput
                        name="user-agent".to_owned()
                        input_type="text".to_owned()
                        class_name="w-full".to_owned()
                        placeholder=|| "User-Agent".to_owned()
                        value=user_agent
                        set_value=set_user_agent
                        on_change=move |_| {
                            set_local_store_value("rc_user_agent", user_agent.get_untracked());
                        }
                    />
                </div>

                <div class="flex gap-4">
                    <div class="flex gap-4 w-full">
                        <AutocompleteInput
                            class_name="min-w-36".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                            options={HEADERS_AUTOCOMPLETE}
                            on_change=move |_| {}
                            value=custom_header_name
                            set_value=set_custom_header_name
                        />
                        <AutocompleteInput
                            class_name="w-full".to_owned()
                            placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                            options={MEDIA_TYPES_AUTOCOMPLETE}
                            on_change=move |_| {}
                            value=custom_header_value
                            set_value=set_custom_header_value
                        />
                        <Button
                            label=move || "+".to_owned()
                            class_name="text-bold".to_owned()
                            button_width=ButtonWidth::OneSymbol
                            color=ButtonColor::Success
                            loading=move || false
                            disabled=move || false
                            on_click=move |_| {
                                let name_converted = custom_header_name.get_untracked().trim().to_lowercase();
                                if !name_converted.is_empty() &&
                                    !is_base_header_name(&name_converted) &&
                                    !custom_header_value.read_untracked().is_empty() {
                                    if custom_headers.read_untracked().iter().find(|h|h.name.read_untracked().trim().to_lowercase() == name_converted).is_none() {
                                        let id = custom_headers.read_untracked().iter().map(|h|h.id).max().unwrap_or_default() + 1;
                                        let (name, set_name) = signal(custom_header_name.get_untracked());
                                        let (value, set_value) = signal(custom_header_value.get_untracked());

                                        set_custom_headers.write().push(CustomHeader{ id, name, set_name, value, set_value });
                                        set_custom_header_name.set("".to_owned());
                                        set_custom_header_value.set("".to_owned());
                                        save_custom_headers(&custom_headers.read_untracked());
                                    }
                                }
                            }
                        />
                    </div>
                </div>

                <For
                    each=move || custom_headers.get()
                    key=|custom_header| custom_header.id
                    children=move |custom_header| {
                        view! {
                            <div class="flex gap-4 w-full">
                                <AutocompleteInput
                                    class_name="min-w-36".to_owned()
                                    placeholder=move || t_display!(i18n, rest_client_header_name).to_string()
                                    options={HEADERS_AUTOCOMPLETE}
                                    on_change=move |_| {
                                        save_custom_headers(&custom_headers.read_untracked());
                                    }
                                    value=custom_header.name
                                    set_value=custom_header.set_name
                                />
                                <AutocompleteInput
                                    class_name="w-full".to_owned()
                                    placeholder=move || t_display!(i18n, rest_client_header_value).to_string()
                                    options={MEDIA_TYPES_AUTOCOMPLETE}
                                    on_change=move |_| {
                                        save_custom_headers(&custom_headers.read_untracked());
                                    }
                                    value=custom_header.value
                                    set_value=custom_header.set_value
                                />
                                <Button
                                    label=move || "-".to_owned()
                                    class_name="text-bold".to_owned()
                                    button_width=ButtonWidth::OneSymbol
                                    color=ButtonColor::Danger
                                    loading=move || false
                                    disabled=move || false
                                    on_click=move |_| {
                                        set_custom_headers.write().retain(|h| h.id != custom_header.id);
                                        save_custom_headers(&custom_headers.read_untracked());
                                    }
                                />
                            </div>
                        }
                    }
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

            <RestClientResponsePanel data=response/>

        </div>
    }
}

fn single_select_option(value: &str) -> (Option<String>, String) {
    (Some(value.to_owned()), value.to_owned())
}

fn is_base_header_name(name: &str) -> bool {
    name == "content-type" || name == "accept" || name == "accept-language" || name == "user-agent"
}

fn save_custom_headers(headers: &Vec<CustomHeader>) {
    let value = headers
        .iter()
        .map(|h| format!("{}:{}", h.name.get_untracked(), h.value.get_untracked()))
        .collect::<Vec<String>>()
        .join("\n");

    set_local_store_value("rc_custom_headers", value);
}

fn restore_custom_headers() -> Vec<CustomHeader> {
    let stored_value = get_local_store_value("rc_custom_headers", "".to_owned());
    if stored_value.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for (i, line) in stored_value.lines().enumerate() {
        if let Some(index) = line.find(":") {
            let (name, set_name) = signal(line[..index].to_owned());
            let (value, set_value) = signal(line[index + 1..].to_owned());

            let header = CustomHeader { id: i + 1, name, set_name, value, set_value };
            result.push(header);
        }
    }

    result
}
