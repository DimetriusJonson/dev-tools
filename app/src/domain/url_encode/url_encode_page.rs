use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::copy_to_clipboard;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::components::ui::text_area::TextArea;

#[component]
pub fn UrlEncoderPage() -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (url, set_url) = signal(get_local_store_value("src_url", "".to_owned()));
    let (dst_url, set_dst_url) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(false);

    let on_encode_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/encode_url")
                .body(url.get_untracked())
            {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => set_dst_url.set(response_text),
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
            set_in_progress.set(false);
        });
    };

    let on_decode_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(true);

            match Request::post("/decode_url")
                .body(url.get_untracked())
            {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => set_dst_url.set(response_text),
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
        copy_to_clipboard(&dst_url.get());
        show_info("Ссылка скопирована в буфер обмена.".to_owned(), messages);
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <TextArea
                name="url".to_owned()
                class_name="md:flex-1 h-[27dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                placeholder="Вставьте url".to_owned()
                value=url
                set_value=set_url
                on_change=move |_| {
                    set_local_store_value("src_url", url.get_untracked());
                }
            />

            <div class="flex flex-col gap-4 items-center justify-center">
                <Button
                    label="Encode".to_owned()
                    button_width=ButtonWidth::Md
                    loading=move || in_progress.get()
                    on_click=on_encode_click
                    disabled=move || in_progress.get()
                />
                <Button
                    label="Decode".to_owned()
                    button_width=ButtonWidth::Md
                    loading=move || in_progress.get()
                    on_click=on_decode_click
                    disabled=move || in_progress.get()
                />
            </div>

            <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[38dvh] md:h-[90dvh]">
                { move || view! {
                    <div class="flex-1 min-h-0 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
                        <CodeInner code={dst_url.get()} lang="xml".to_string()/>
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
