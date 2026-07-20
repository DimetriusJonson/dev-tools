use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

use crate::components::ui::text_input::TextInput;
use crate::i18n::*;
use crate::model::rest_client_request::RestClientRequest;
use crate::model::rest_client_response::RestClientResponse;

#[derive(PartialEq, Copy, Clone)]
enum ResponceTabKind {
    Body,
    Headers,
}

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

#[component]
pub fn RestClientPage() -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (resp_tab_selected, set_resp_tab_selected) = signal(ResponceTabKind::Headers);

    let (method, set_method) =
        signal(get_local_store_value("rest_client_method", "GET".to_owned()));
    let (url, set_url) = signal(get_local_store_value("rest_client_url", "".to_owned()));
    let (body, set_body) = signal(get_local_store_value("rest_client_body", "".to_owned()));
    let (response_text, set_response_text) = signal("".to_owned());
    let (response_status, set_response_status) = signal(None);
    let (response_headers, set_response_headers) = signal(/*Vec::new()*/ vec![
        ("host".to_owned(), "google".to_owned()),
        ("content-type".to_owned(), "application/xml".to_owned()),
        ("accept".to_owned(), "application/json".to_owned()),
        ("x-forwarded-for".to_owned(), "192.168.0.1".to_owned()),
    ]);
    let (in_progress, set_in_progress) = signal(InProgressType::None);

    let on_send_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Send);

            let mut headers = Vec::new();
            headers.push(("content-type".to_owned(), "application/xml".to_owned()));

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
                            set_response_status.set(Some(resp.status_code));
                            set_response_text.set(resp.body);
                            set_response_headers.set(resp.headers);
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
                        not_selected_text=move || "".to_owned()
                        options=move || {vec![
                            (Some("GET".to_owned()), "GET".to_owned()),
                            (Some("POST".to_owned()), "POST".to_owned()),
                            (Some("PUT".to_owned()), "PUT".to_owned()),
                            (Some("DELETE".to_owned()), "DELETE".to_owned()),
                            (Some("PATCH".to_owned()), "PATCH".to_owned()),
                            (Some("HEAD".to_owned()), "HEAD".to_owned()),
                            (Some("OPTIONS".to_owned()), "OPTIONS".to_owned()),
                            ]}
                        on_change=move |value| {
                            set_local_store_value("xml_ident", value);
                        }
                        value=method
                        set_value=set_method
                    />

                    <TextInput
                        name="url".to_owned()
                        input_type="text".to_owned()
                        class_name="".to_owned()
                        placeholder=move || {t!(i18n, rest_client_url_placeholder).to_html()}
                        value=url
                        set_value=set_url
                        on_change=move |_| {
                            set_local_store_value("rest_client_url", url.get_untracked());
                        }
                    />

                </div>
                <div class="flex-1 flex">
                    <TextArea
                        name="body".to_owned()
                        class_name="md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                        placeholder=move || {t!(i18n, rest_client_body_placeholder).to_html()}
                        value=body
                        set_value=set_body
                        on_change=move |_| {
                            set_local_store_value("rest_client_body", body.get_untracked());
                        }
                    />
                </div>
            </div>

            <div class="flex flex-col gap-4 items-center justify-center">
                <div class="flex flex-row md:flex-col gap-4">

                    <Button
                        label=move || t!(i18n, rest_client_send_btn_label).to_html()
                        button_width=ButtonWidth::Lg
                        loading=move || in_progress.get() == InProgressType::Send
                        on_click=on_send_click
                        disabled=move || in_progress.get().is_active()
                    />
                </div>

            </div>


            // ************** Response
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
                        { move || view! {
                            <div class="flex">
                                <span class="dark:text-white">{format!("Status: {}", response_status.get().map(|s|s.to_string()).unwrap_or("".to_owned()))}</span>
                            </div>
                            <div class="flex-1 min-h-0 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
                                <CodeInner code={response_text.get()} lang="xml".to_string()/>
                            </div>
                            }
                        }
                    </div>

                    <div class="flex flex-col md:flex-row gap-4 pt-4 text-xs md:text-base min-h-0"
                        class:block=move || resp_tab_selected.get() == ResponceTabKind::Headers
                        class:hidden=move || resp_tab_selected.get() != ResponceTabKind::Headers
                    >
                        <div class="overflow-x-auto rounded-md border border-gray-300 dark:border-gray-700 shadow-sm">
                            <table class="w-full table-fixed border-collapse text-left text-sm text-gray-500">
                                <thead class="text-xs font-semibold uppercase text-gray-900 dark:text-gray-50 bg-gray-100 dark:bg-gray-900">
                                    <tr>
                                        <th scope="col" class="w-1/2 px-6 py-4">Header</th>
                                        <th scope="col" class="w-1/2 px-6 py-4">Value</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-300 dark:divide-gray-700 border-t font-medium">
                                { move || view! {
                                        <ForEnumerate
                                            each=move || response_headers.get()
                                            key=|header| header.0.to_owned()
                                            let(idx, header)
                                        >
                                                <tr class=(["hover:bg-gray-200", "dark:hover:bg-gray-800", "dark:text-gray-300", "text-gray-900"], move || idx.get() % 2 == 0)
                                                    class=(["bg-gray-100", "hover:bg-gray-200", "dark:bg-gray-900", "dark:hover:bg-gray-800", "text-gray-900", "dark:text-gray-50"], move || idx.get() % 2 == 1)
                                                    >
                                                    <td class="px-6 py-4">{header.0}</td>
                                                    <td class="px-6 py-4">{header.1}</td>
                                                </tr>
                                        </ ForEnumerate>
                                }}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}
