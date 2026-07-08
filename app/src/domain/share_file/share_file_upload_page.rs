use gloo_net::http::Request;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};

use crate::common::ui_utils::copy_to_clipboard;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::file_input::FileInput;

#[component]
pub fn ShareFileUploadPage() -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let (shared_url, set_shared_url) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);
    let file_input_ref: NodeRef<html::Input> = NodeRef::new();

    let on_upload_file_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let file_input = file_input_ref.get_untracked().expect("input to exist");
            if let Some(files) = file_input.files()
                && let Some(file) = files.get(0)
            {
                match Request::post("/share_file_upload")
                    .header("content-type", &file.type_())
                    .query([("file_name", file.name())])
                    .body(&file)
                {
                    Ok(request) => match request.send().await {
                        Ok(response) => {
                            if response.status() == 200 {
                                let server_url = response
                                    .headers()
                                    .get("remote-server-url")
                                    .unwrap_or_else(|| {
                                        let window =
                                            web_sys::window().expect("No global window exists");
                                        let location = window.location();
                                        location.origin().to_owned().unwrap_or_default()
                                    });

                                set_shared_url.set(format!(
                                    "{}/share_file/view?id={}",
                                    server_url,
                                    response.text().await.unwrap()
                                ));
                                show_info("Файл загружен!".to_owned(), messages);
                            } else {
                                show_error(response.status_text(), messages)
                            }
                        }
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                }
            }

            set_in_progress.set(false);
        });
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&shared_url.get());
        show_info("Ссылка скопирована в буфер обмена.".to_owned(), messages);
    };

    view! {
        <div class="flex flex-col px-[30vw] py-12 gap-4 dark:text-white text-xs md:text-base">
            <div class="flex">
                <FileInput node_ref=file_input_ref />
                <Button
                    label="Загрузить".to_owned()
                    button_width=ButtonWidth::Md
                    loading=move || in_progress.get()
                    on_click=on_upload_file_click
                    disabled=move || in_progress.get()
                />
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
                    <li>Выберите файл, которым хотите поделится. Максимальный размер файла два мегабайта.</li>
                    <li>{"Нажмите "}<b>Загрузить</b>{" для загрузки файла и формирования на него ссылки."}</li>
                    <li>{"Нажмите "}<b>Скопировать в буфер обмена</b>{"."}</li>
                    <li>Вставьте ссылку на файл из буфера обмена.</li>
                    <li>Срок жизни ссылки <b>три дня</b>.</li>
                </ul>
            </div>

        </div>
    }
}
