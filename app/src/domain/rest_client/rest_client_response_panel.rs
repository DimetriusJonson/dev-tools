use crate::common::constants::MEDIA_TYPES;
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
    let i18n = use_i18n();
    let (resp_tab_selected, set_resp_tab_selected) = signal(ResponceTabKind::Body);
    {
        move || {
            set_resp_tab_selected.set(ResponceTabKind::Body);
            let (response_status, response_text, response_headers, resp_code_lang) =
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
                            .map(|v|v.unwrap_or("html".to_owned()))
                            .nth(0)
                            .unwrap_or("html".to_owned()),
                    ),
                    None => ("".to_owned(), "".to_owned(), Vec::new(), "html".to_owned()),
                };

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
                        <div class="flex">
                            <span class="dark:text-white">{format!("Status: {}", response_status)}</span>
                        </div>
                        <div class="flex-1 min-h-0 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
                            <CodeInner code={response_text} lang={move || resp_code_lang.to_owned()}/>
                        </div>
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
                                    <ForEnumerate
                                        each=move || response_headers.clone()
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
                                </tbody>
                            </table>
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
