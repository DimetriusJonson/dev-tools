use fast_qr::convert::svg::SvgBuilder;
use fast_qr::convert::{Builder, Shape};
use fast_qr::{ECL, QRBuilder};
use gloo_net::http::Request;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};
use leptos_router::hooks::use_query_map;
use model::share_file::share_file_server::ShareFileServerDto;
use web_sys::{File, HtmlInputElement};

use crate::common::ui_utils::copy_to_clipboard;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::drag_file::DragFile;
use crate::components::ui::file_input::FileInput;
use crate::components::ui::select_input::SelectInput;
use crate::domain::share_file::ShareStorage;
use crate::i18n::*;

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

enum UploadResult {
    Success,
    Error(String),
    ExceedSize,
}

#[component]
pub fn ShareFileUploadPage() -> impl IntoView {
    let params = use_query_map();
    let i18n = use_i18n();
    let share_storage = use_context::<ShareStorage>().expect("Cant get share storage context!");

    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let (shared_url, set_shared_url) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);
    let file_input_ref: NodeRef<html::Input> = NodeRef::new();
    let selected_file: RwSignal<Option<File>> = RwSignal::new(None);
    let (custom_server, set_custom_server) = signal("".to_owned());
    let (qr_code_svg, set_qr_code_svg) = signal("".to_owned());

    let upload_success_memo =
        Memo::new(move |_| t_string!(i18n, share_file_upload_success).to_owned());
    let upload_exceed_file_size_memo =
        Memo::new(move |_| t_string!(i18n, share_file_upload_exceed_file_size).to_owned());

    let mode = move || params.read().get("mode").unwrap_or("file".to_owned());

    Effect::watch(
        move || shared_url.get(),
        move |value, _prev, _| {
            let qrcode = QRBuilder::new(value.to_owned()).ecl(ECL::M).build().unwrap();
            set_qr_code_svg.set(SvgBuilder::default().shape(Shape::Square).to_str(&qrcode));
        },
        false,
    );

    let on_upload_file_click = move |_| {
        if let Some(file) = selected_file.get_untracked() {
            upload_file(
                UploadParams::File(file),
                set_in_progress,
                set_shared_url,
                custom_server.get(),
                move |upload_result| match upload_result {
                    UploadResult::Success => {
                        selected_file.set(None);
                        if let Some(input_ref) = file_input_ref.write().as_mut() {
                            input_ref.set_files(None);
                        }

                        show_info(upload_success_memo.get_untracked(), messages);
                    }
                    UploadResult::Error(err) => {
                        show_error(err, messages);
                    }
                    UploadResult::ExceedSize => {
                        show_error(upload_exceed_file_size_memo.get_untracked(), messages)
                    }
                },
            );
        }
    };

    let on_upload_text_click = move |_| {
        upload_file(
            UploadParams::Text(mode(), share_storage.0.get_untracked()),
            set_in_progress,
            set_shared_url,
            custom_server.get(),
            move |upload_result| match upload_result {
                UploadResult::Success => {
                    selected_file.set(None);
                    if let Some(input_ref) = file_input_ref.write().as_mut() {
                        input_ref.set_files(None);
                    }

                    show_info(upload_success_memo.get_untracked(), messages);
                }
                UploadResult::Error(err) => {
                    show_error(err, messages);
                }
                UploadResult::ExceedSize => {
                    show_error(upload_exceed_file_size_memo.get_untracked(), messages)
                }
            },
        );
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&shared_url.get());
        show_info(
            t_string!(i18n, share_file_upload_page_copied_to_clipboard_msg).to_owned(),
            messages,
        );
    };

    let custom_servers_resource = LocalResource::new(async move || {
        match Request::get("/share_file_custom_servers").build() {
            Ok(request) => match request.send().await {
                Ok(response) => match response.json::<Vec<ShareFileServerDto>>().await {
                    Ok(servers) => {
                        let mut result = Vec::new();
                        for server in servers {
                            result
                                .push((Some(server.url.to_owned()), server.description.to_owned()));
                        }
                        result
                    }
                    Err(_err) => Vec::new(),
                },
                Err(_err) => Vec::new(),
            },
            Err(_err) => Vec::new(),
        }
    });

    view! {
        <Show when=move || { mode() == "file" }>
            {
                view! {
                    <div class="flex justify-center items-center w-full p-4"
                        class:hidden=move || !shared_url.get().is_empty()>
                        <DragFile
                            on_drop_file=move |file| {
                                upload_file(UploadParams::File(file), set_in_progress, set_shared_url, custom_server.get(), move |upload_result| {
                                    match upload_result {
                                        UploadResult::Success => {
                                            selected_file.set(None);
                                            if let Some(input_ref) = file_input_ref.write().as_mut() {
                                                input_ref.set_files(None);
                                            }
                                            show_info(upload_success_memo.get_untracked(), messages);
                                        },
                                        UploadResult::Error(err) => {
                                            show_error(err, messages);
                                        },
                                        UploadResult::ExceedSize => {
                                            show_error(upload_exceed_file_size_memo.get_untracked(), messages)
                                        },
                                    }
                                });
                            }
                            on_paste_file=move |file| {selected_file.set(Some(file));}
                            />
                    </div>
                }.into_view()
            }
        </Show>

        <div class="flex flex-col px-4 md:px-[30vw] py-12 gap-4 dark:text-white text-xs md:text-base">
            <Show when=move || { mode() == "file" }>
                {
                    view! {
                        <div class="flex" class:hidden=move || !shared_url.get().is_empty()>
                            <FileInput node_ref=file_input_ref on:change=move |event| {
                                let input_file = event_target::<HtmlInputElement>(&event);
                                if let Some(files) = input_file.files() && files.length() > 0 {
                                    selected_file.set(files.get(0));
                                }
                            }/>
                            <Button
                                title=move || "".to_owned()
                                label=move || t_string!(i18n, share_file_upload_page_upload_btn_label).to_owned()
                                button_width=ButtonWidth::Md
                                loading=move || in_progress.get()
                                on_click=on_upload_file_click
                                disabled=move || in_progress.get() || selected_file.read().is_none()
                            />
                        </div>
                    }.into_view()
                }
            </Show>

            <Show when=move || { mode() != "file" }>
                {
                    view! {
                        <div class="flex items-center justify-center" class:hidden=move || !shared_url.get().is_empty()>
                            <Button
                                title=move || "".to_owned()
                                label=move || format!("{} {}", t_string!(i18n, share_file_upload_page_upload_btn_label), mode())
                                button_width=ButtonWidth::Auto
                                loading=move || in_progress.get()
                                on_click=on_upload_text_click
                                disabled=move || in_progress.get()
                            />
                        </div>
                    }.into_view()
                }
            </Show>

            {move || custom_servers_resource.get().map(|custom_servers| {
                    let hidden = custom_servers.is_empty() || !shared_url.get().is_empty();
                    view! {
                        <div class="flex items-center gap-2"
                            class:hidden=hidden>
                            <label for="server_addr" title=move || {t_string!(i18n, share_file_upload_page_server_addr_title).to_owned()}>{t!(i18n, share_file_upload_page_server_addr_label)}</label>
                            <SelectInput
                                class_name="w-full".to_owned()
                                name={"server_addr".to_owned()}
                                value={custom_server}
                                set_value={set_custom_server}
                                label=move || "Server addr".to_owned()
                                options=move || custom_servers.clone()
                                not_selected_text={move || t_string!(i18n, share_file_upload_page_server_addr_not_selected).to_owned()}
                                on_change=move |_| {}
                            />
                        </div>
                        <p class="text-xs text-gray-600"
                            class:hidden=hidden>
                            {t!(i18n, share_file_upload_page_server_addr_descr, <br/> = || view! { <br/> })}
                        </p>
                    }.into_view()
                }
            )}

            <Show when=move || { !shared_url.get().is_empty() }>
                <div class="flex flex-col gap-4 items-center justify-center">
                    <div class="flex flex-col gap-4 items-center justify-center">
                        <div>
                            <span class="text-white">Ссылка:</span>
                            <span class="text-sky-500 px-2">{shared_url.get()}</span>
                        </div>

                        <Button
                            title=move || "".to_owned()
                            label=move || t_string!(i18n, copy_to_clipboard_btn_label).to_owned()
                            button_width=ButtonWidth::Auto
                            loading=move || in_progress.get()
                            on_click=on_copy_click
                            disabled=move || in_progress.get()
                        />
                    </div>
                    <div class="flex px-4 w-full" style="width:50%;" inner_html=qr_code_svg />
                </div>
            </Show>

            <Show when=move || { mode() == "file" }>
                {
                    view! {
                        <div class="py-4 px-4">
                            <ul class="list-decimal [&_li]:py-1 text-gray-600 dark:text-gray-400 [&_b]:text-black [&_b]:dark:text-gray-300 [&_b]:p-1">

                                <li>{t!(i18n, share_file_upload_info_1, <b> = <b />)}</li>
                                <ul class="list-disc pl-4">
                                    <li>{t!(i18n, share_file_upload_info_2)}</li>
                                    <li>{t!(i18n, share_file_upload_info_3)}</li>
                                </ul>

                                <li>{t!(i18n, share_file_upload_info_4, <b> = <b />)}</li>
                                <li>{t!(i18n, share_file_upload_info_5, <b> = <b />)}</li>
                                <li>{t!(i18n, share_file_upload_info_6)}</li>
                                <li>{t!(i18n, share_file_upload_info_7, <b> = <b />)}</li>
                            </ul>
                        </div>
                    }.into_view()
                }
            </Show>

        </div>
    }
}

fn upload_file(
    params: UploadParams,
    set_in_progress: WriteSignal<bool>,
    set_shared_url: WriteSignal<String>,
    custom_server_url: String,
    callback: impl Fn(UploadResult) + Send + Sync + 'static,
) {
    spawn_local(async move {
        set_in_progress.set(true);

        let mut result = UploadResult::Success;

        let (service_name, max_file_size) = if !custom_server_url.is_empty() {
            ("/share_local_file_upload", usize::MAX)
        } else {
            ("/share_file_upload", MAX_FILE_SIZE)
        };

        let request = match params {
            UploadParams::File(file) => {
                if file.size() <= max_file_size as f64 {
                    Request::post(service_name)
                        .header("content-type", &file.type_())
                        .query([("file_name", file.name())])
                        .body(&file)
                } else {
                    set_in_progress.set(false);
                    callback(UploadResult::ExceedSize);
                    return;
                }
            }
            UploadParams::Text(media_type, text) => Request::post(service_name)
                .header("content-type", get_media_content_type(&media_type))
                .query([("file_name", format!("data.{}", media_type))])
                .body(&text),
        };

        match request {
            Ok(request) => match request.send().await {
                Ok(response) => {
                    if response.status() == 200 {
                        let server_url =
                            response.headers().get("remote-server-url").unwrap_or_else(|| {
                                if let Some(window) = web_sys::window() {
                                    let location = window.location();
                                    return location.origin().to_owned().unwrap_or_default();
                                }
                                "".to_owned()
                            });

                        let mut file_url = None;
                        if !custom_server_url.is_empty() {
                            match response.text().await {
                                Ok(resp_text) => {
                                    file_url = Some(format!(
                                        "{}/share_file/view?id={}&local=true",
                                        custom_server_url, resp_text
                                    ))
                                }
                                Err(err) => result = UploadResult::Error(err.to_string()),
                            }
                        } else {
                            match response.text().await {
                                Ok(resp_text) => {
                                    file_url = Some(format!(
                                        "{}/share_file/view?id={}",
                                        server_url, resp_text
                                    ))
                                }
                                Err(err) => result = UploadResult::Error(err.to_string()),
                            }
                        }

                        if let Some(file_url) = file_url {
                            set_shared_url.set(file_url.to_owned());
                        }
                    } else {
                        result = UploadResult::Error(response.status_text());
                    }
                }
                Err(err) => result = UploadResult::Error(err.to_string()),
            },
            Err(err) => result = UploadResult::Error(err.to_string()),
        }

        set_in_progress.set(false);
        callback(result);
    });
}

fn get_media_content_type(media_type: &str) -> &str {
    match media_type {
        "xml" => "application/xml",
        "json" => "application/json",
        _ => "text/plain",
    }
}

enum UploadParams {
    File(File),
    Text(String, String),
}
