use gloo_net::http::Request;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};
use web_sys::{File, HtmlInputElement};

use crate::common::ui_utils::copy_to_clipboard;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::drag_file::DragFile;
use crate::components::ui::file_input::FileInput;
use crate::components::ui::select_input::{SelectInput, SelectOption};
use crate::i18n::*;

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

#[component]
pub fn ShareFileUploadPage() -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let (shared_url, set_shared_url) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);
    let file_input_ref: NodeRef<html::Input> = NodeRef::new();
    let selected_file: RwSignal<Option<File>> = RwSignal::new(None);
    let (custom_server, set_custom_server) = signal("".to_owned());

    let on_upload_file_click = move |_| {
        if let Some(file) = selected_file.get_untracked() {
            upload_file(
                file,
                set_in_progress,
                set_shared_url,
                messages,
                custom_server.get(),
                i18n,
                move |success| {
                    if success {
                        selected_file.set(None);
                        file_input_ref.write().as_mut().unwrap().set_files(None);
                    }
                },
            );
        }
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&shared_url.get());
        show_info(t!(i18n, share_file_upload_page_copied_to_clipboard_msg).to_html(), messages);
    };

    let custom_servers_resource = OnceResource::new(get_custom_servers());

    view! {

        <div class="flex justify-center items-center w-full p-4"
            class:hidden=move || !shared_url.get().is_empty()>
            <DragFile
                on_drop_file=move |file| {
                    upload_file(file, set_in_progress, set_shared_url, messages, custom_server.get(), i18n, move |success| {
                        if success {
                            selected_file.set(None);
                            file_input_ref.write().as_mut().unwrap().set_files(None);
                        }
                    });
                }
                on_paste_file=move |file| {selected_file.set(Some(file));}
                />
        </div>

        <div class="flex flex-col px-4 md:px-[30vw] py-12 gap-4 dark:text-white text-xs md:text-base">
            <div class="flex" class:hidden=move || !shared_url.get().is_empty()>
                <FileInput node_ref=file_input_ref on:change=move |event| {
                    let input_file = event_target::<HtmlInputElement>(&event);
                    if let Some(files) = input_file.files() && files.length() > 0 {
                        selected_file.set(files.get(0));
                    }
                }/>
                <Button
                    label=move || t!(i18n, share_file_upload_page_upload_btn_label).to_html()
                    button_width=ButtonWidth::Md
                    loading=move || in_progress.get()
                    on_click=on_upload_file_click
                    disabled=move || in_progress.get() || selected_file.read().is_none()
                />
            </div>

            <Transition
                fallback=move || view! { <div>{t!(i18n, loading_progress)}</div> }
                >
                {move || custom_servers_resource.get().map(|data|
                    data.map(|custom_servers| {
                        let hidden = custom_servers.is_empty() || !shared_url.get().is_empty();
                        view! {
                            <div class="flex items-center"
                                class:hidden=hidden>
                                <label for="server_addr" title=move || {t!(i18n, share_file_upload_page_server_addr_title).to_html()}>{t!(i18n, share_file_upload_page_server_addr_label)}</label>
                                <SelectInput
                                    class_name="px-2".to_owned()
                                    name={"server_addr".to_owned()}
                                    value={custom_server}
                                    set_value={set_custom_server}
                                    label=move || "Server addr".to_owned()
                                    options=move || custom_servers.clone()
                                    not_selected_text={move || t!(i18n, share_file_upload_page_server_addr_not_selected).to_html()}
                                    on_change=move |_| {}
                                />
                            </div>
                            <p class="text-xs text-gray-600"
                                class:hidden=hidden>
                                {t!(i18n, share_file_upload_page_server_addr_descr, <br/> = || view! { <br/> })}
                            </p>
                        }.into_view()
                    })
                )}
            </Transition>

            <div class="flex flex-col gap-4 items-center justify-center">
                <Show when=move || { !shared_url.get().is_empty() }>

                    <div>
                        <span class="text-white">Ссылка:</span>
                        <span class="text-sky-500 px-2">{shared_url.get()}</span>
                    </div>

                    <Button
                        label=move || t!(i18n, copy_to_clipboard_btn_label).to_html()
                        button_width=ButtonWidth::Auto
                        loading=move || in_progress.get()
                        on_click=on_copy_click
                        disabled=move || in_progress.get()
                    />

                </Show>
            </div>

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

        </div>
    }
}

fn upload_file(
    file: File,
    set_in_progress: WriteSignal<bool>,
    set_shared_url: WriteSignal<String>,
    messages: Messages,
    custom_server_url: String,
    i18n: leptos_i18n::I18nContext<Locale, I18nKeys>,
    callback: impl Fn(bool) + Send + Sync + 'static,
) {
    spawn_local(async move {
        set_in_progress.set(true);

        let mut result = false;

        let (service_name, max_file_size) = if !custom_server_url.is_empty() {
            ("/share_local_file_upload", usize::MAX)
        } else {
            ("/share_file_upload", MAX_FILE_SIZE)
        };

        if file.size() <= max_file_size as f64 {
            match Request::post(service_name)
                .header("content-type", &file.type_())
                .query([("file_name", file.name())])
                .body(&file)
            {
                Ok(request) => match request.send().await {
                    Ok(response) => {
                        if response.status() == 200 {
                            let server_url =
                                response.headers().get("remote-server-url").unwrap_or_else(|| {
                                    let window =
                                        web_sys::window().expect("No global window exists");
                                    let location = window.location();
                                    location.origin().to_owned().unwrap_or_default()
                                });

                            if !custom_server_url.is_empty() {
                                set_shared_url.set(format!(
                                    "{}/share_file/view?id={}&local=true",
                                    custom_server_url,
                                    response.text().await.unwrap()
                                ));
                            } else {
                                set_shared_url.set(format!(
                                    "{}/share_file/view?id={}",
                                    server_url,
                                    response.text().await.unwrap()
                                ));
                            }
                            result = true;

                            show_info(t!(i18n, share_file_upload_success).to_html(), messages);
                        } else {
                            show_error(response.status_text(), messages);
                        }
                    }
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
        } else {
            show_error(t!(i18n, share_file_upload_exceed_file_size).to_html(), messages)
        }

        set_in_progress.set(false);
        callback(result);
    });
}

#[server]
pub async fn get_custom_servers() -> Result<Vec<SelectOption>, ServerFnError> {
    use crate::common::app_state::ssr::AppState;
    use crate::common::net_utils::ssr::get_local_addrs;

    let app_state =
        use_context::<AppState>().ok_or_else(|| ServerFnError::new("App state missing."))?;

    if app_state.pool.is_some() {
        return Ok(Vec::new());
    }

    let site_addr = app_state.leptos_options.site_addr;

    let addrs =
        get_local_addrs().map_err(|e| ServerFnError::new(format!("Request failed: {e}")))?;

    Ok(addrs
        .iter()
        .map(|a| {
            (
                Some(format!("http://{}:{}", a.0.to_owned(), site_addr.port())),
                format!("{} ({})", a.1, a.0),
            )
        })
        .collect())
}
