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

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;

#[component]
pub fn ShareFileUploadPage() -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let (shared_url, set_shared_url) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);
    let file_input_ref: NodeRef<html::Input> = NodeRef::new();
    let selected_file: RwSignal<Option<File>> = RwSignal::new(None);
    let (custom_server, set_custom_server) = signal("".to_owned());

    let on_upload_file_click = move |_| {
        if let Some(file) = selected_file.get_untracked() {
            let selected_file = selected_file.clone();
            upload_file(
                file,
                set_in_progress,
                set_shared_url,
                messages,
                custom_server.get(),
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
        show_info("Ссылка скопирована в буфер обмена.".to_owned(), messages);
    };

    let custom_servers_resource = OnceResource::new(get_custom_servers());

    view! {

        <div class="flex justify-center items-center w-full p-4"
            class:hidden=move || !shared_url.get().is_empty()>
            <DragFile
                on_drop_file=move |file| {
                    let selected_file = selected_file.clone();
                    upload_file(file, set_in_progress, set_shared_url, messages, custom_server.get(), move |success| {
                        if success {
                            selected_file.set(None);
                            file_input_ref.write().as_mut().unwrap().set_files(None);
                        }
                    });
                }
                on_paste_file=move |file| {selected_file.set(Some(file));}
                />
        </div>

        <div class="flex flex-col px-[30vw] py-12 gap-4 dark:text-white text-xs md:text-base">
            <div class="flex" class:hidden=move || !shared_url.get().is_empty()>
                <FileInput node_ref=file_input_ref on:change=move |event| {
                    let input_file = event_target::<HtmlInputElement>(&event);
                    if let Some(files) = input_file.files() {
                        if files.length() > 0 {
                            selected_file.set(files.get(0));
                        }
                    }
                }/>
                <Button
                    label="Загрузить".to_owned()
                    button_width=ButtonWidth::Md
                    loading=move || in_progress.get()
                    on_click=on_upload_file_click
                    disabled=move || in_progress.get() || selected_file.read().is_none()
                />
            </div>

            <div>
                <Transition
                    fallback=move || view! { <div>Загрузка...</div> }
                    >
                    {move || custom_servers_resource.get().map(|data|
                        data.map(|custom_servers| {
                                view! {
                                    <div>
                                        <SelectInput
                                            name={"server_addr".to_owned()}
                                            value={custom_server}
                                            set_value={set_custom_server}
                                            label={"Server addr".to_owned()}
                                            options=move || custom_servers.clone()
                                            not_selected_text={"Сервер по умолчанию".to_owned()}
                                            on_change=move |_| {}
                                        />
                                    </div>
                                }
                        })
                    )}
                </Transition>
            </div>

            <div class="flex flex-col gap-4 items-center justify-center">
                <Show when=move || { !shared_url.get().is_empty() }
                    fallback=|| view! {  }>

                    <div>
                        <span class="text-white">Ссылка:</span>
                        <span class="text-sky-500 px-2">{shared_url.get()}</span>
                    </div>

                    <Button
                        label="Скопировать в буфер обмена".to_owned()
                        button_width=ButtonWidth::Auto
                        loading=move || in_progress.get()
                        on_click=on_copy_click
                        disabled=move || in_progress.get()
                    />

                </Show>
            </div>

            <div class="py-4">
                <ul class="list-decimal [&_li]:py-1 text-gray-600 dark:text-gray-400 [&_b]:text-black [&_b]:dark:text-gray-300 [&_b]:p-1">

                    <li>{"Выберите файл, которым хотите поделится. Максимальный размер файла "}<b>{"5 мегабайт"}</b>.</li>
                    <ul class="list-disc pl-4">
                        <li>{"Или перетащите файл в верхнюю область."}</li>
                        <li>{"Или вставьте изображение из буфера обмена, нажав Ctrl+V в верхней области."}</li>
                    </ul>

                    <li>{"Нажмите "}<b>{"Загрузить"}</b>{" для загрузки файла и формирования на него ссылки."}</li>
                    <li>{"Нажмите "}<b>{"Скопировать в буфер обмена"}</b>{"."}</li>
                    <li>{"Вставьте ссылку на файл из буфера обмена."}</li>
                    <li>{"Срок жизни ссылки "}<b>{"3 дня"}</b>{"."}</li>
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
    callback: impl Fn(bool) -> () + Send + Sync + 'static,
) {
    spawn_local(async move {
        set_in_progress.set(true);

        let mut result = false;

        if file.size() <= MAX_FILE_SIZE as f64 {
            match Request::post("/share_file_upload")
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

                            set_shared_url.set(format!(
                                "{}/share_file/view?id={}",
                                if !custom_server_url.is_empty() {
                                    custom_server_url
                                } else {
                                    server_url
                                },
                                response.text().await.unwrap()
                            ));
                            result = true;

                            show_info("Файл загружен!".to_owned(), messages);
                        } else {
                            show_error(response.status_text(), messages);
                        }
                    }
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
        } else {
            show_error("Файл слишком большой!".to_owned(), messages)
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
