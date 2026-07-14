use gloo_net::http::Request;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::{copy_to_clipboard, save_file_to_disk};
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::components::ui::file_input::FileInput;
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

#[component]
pub fn JsonPage() -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (json, set_json) = signal(get_local_store_value("src_json", "".to_owned()));
    let (dst_json, set_dst_json) = signal("".to_owned());
    let (ident, set_ident) = signal(get_local_store_value("json_ident", "4".to_owned()));
    let (in_progress, set_in_progress) = signal(false);

    let file_input_ref: NodeRef<html::Input> = NodeRef::new();

    let on_format_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/format_json")
                .query([("ident", ident.get_untracked())])
                .header("content-type", "application/json")
                .body(json.get_untracked())
            {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => set_dst_json.set(response_text),
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
            set_in_progress.set(false);
        });
    };

    let on_format_file_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            let file_input = file_input_ref.get_untracked().expect("input to exist");
            if let Some(files) = file_input.files()
                && let Some(file) = files.get(0)
            {
                match Request::post("/format_json")
                    .header("content-type", "application/json")
                    .body(&file)
                {
                    Ok(request) => match request.send().await {
                        Ok(response) => match response.binary().await {
                            Ok(bytes) => {
                                let file_name = format!("formatted_{}", file.name());
                                match save_file_to_disk(
                                    bytes.to_vec(),
                                    &file_name,
                                    "application/json",
                                ) {
                                    Ok(_) => {
                                        show_info(format!("Файл {} сохранен", file_name), messages)
                                    }
                                    Err(err) => show_error(err.as_string().unwrap(), messages),
                                }
                            }
                            Err(err) => show_error(err.to_string(), messages),
                        },
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                }
            }

            set_in_progress.set(false);
        });
    };

    let on_escape_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/escape_json").body(json.get_untracked()) {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => set_dst_json.set(response_text),
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
            set_in_progress.set(false);
        });
    };

    let on_unescape_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/unescape_json").body(json.get_untracked()) {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => set_dst_json.set(response_text),
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
            set_in_progress.set(false);
        });
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&dst_json.get());
        show_info("JSON скопирован в буфер обмена.".to_owned(), messages);
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[35dvh] md:h-[90dvh]">
                <TextArea
                    name="json".to_owned()
                    class_name="md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                    placeholder="Вставьте Json".to_owned()
                    value=json
                    set_value=set_json
                    on_change=move |_| {
                        set_local_store_value("src_json", json.get_untracked());
                    }
                />
                <div class="flex flex-row">
                    <FileInput node_ref=file_input_ref />
                    <Button
                        label="Format".to_owned()
                        button_width=ButtonWidth::Md
                        loading=move || in_progress.get()
                        on_click=on_format_file_click
                        disabled=move || in_progress.get()
                    />
                </div>

            </div>

            <div class="flex flex-col gap-4 items-center justify-center">
                <div class="flex flex-row md:flex-col gap-4">
                    <SelectInput
                        name="ident".to_owned()
                        label="Отступ".to_owned()
                        options=move || {vec![(Some("2".to_owned()), "2 отступа".to_owned()), (Some("3".to_owned()), "3 отступа".to_owned()), (Some("4".to_owned()), "4 отступа".to_owned())]}
                        on_change=move |value| {
                            set_local_store_value("json_ident", value);
                        }
                        value=ident
                        set_value=set_ident
                    />

                    <Button
                        label="Format".to_owned()
                        button_width=ButtonWidth::Md
                        loading=move || in_progress.get()
                        on_click=on_format_click
                        disabled=move || in_progress.get()
                    />
                </div>

                <div class="flex flex-row md:flex-col gap-4 md:py-8">
                    <Button
                        label="Unescape".to_owned()
                        button_width=ButtonWidth::Md
                        loading=move || in_progress.get()
                        on_click=on_unescape_click
                        disabled=move || in_progress.get()
                    />

                    <Button
                        label="Escape".to_owned()
                        button_width=ButtonWidth::Md
                        loading=move || in_progress.get()
                        on_click=on_escape_click
                        disabled=move || in_progress.get()
                    />

                </div>


            </div>

            <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[35dvh] md:h-[90dvh]">
                { move || view! {
                    <div class="flex-1 min-h-0 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
                        <CodeInner code={dst_json.get()} lang="json".to_string()/>
                    </div>
                    }
                }

                <Button
                    label="Скопировать в буфер обмена".to_owned()
                    button_width=ButtonWidth::Auto
                    loading=move || in_progress.get()
                    on_click=on_copy_click
                    disabled=move || in_progress.get()
                />

            </div>
        </div>
    }
}
