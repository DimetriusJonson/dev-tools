use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::copy_to_clipboard;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::components::ui::text_area::TextArea;
use crate::i18n::use_i18n;
use crate::i18n::*;

#[derive(PartialEq, Copy, Clone)]
enum InProgressType {
    None, 
    Encode,
    Decode
}

impl InProgressType {
    fn is_active(self) -> bool {
        self != InProgressType::None
    }
}

#[component]
pub fn UrlEncoderPage() -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (url, set_url) = signal(get_local_store_value("src_url", "".to_owned()));
    let (dst_url, set_dst_url) = signal("".to_owned());
    let (in_progress, set_in_progress) = signal(InProgressType::None);

    let on_encode_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Encode);

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
            set_in_progress.set(InProgressType::None);
        });
    };

    let on_decode_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Decode);

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
            set_in_progress.set(InProgressType::None);
        });
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&dst_url.get());
        show_info(t!(i18n, url_page_copied_to_clipboard_msg).to_html(), messages);
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <TextArea
                name="url".to_owned()
                class_name="md:flex-1 h-[27dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                placeholder=move || t!(i18n, url_page_src_placeholder).to_html()
                value=url
                set_value=set_url
                on_change=move |_| {
                    set_local_store_value("src_url", url.get_untracked());
                }
            />

            <div class="flex flex-col gap-4 items-center justify-center">
                <Button
                    label=move || t!(i18n, url_page_encode_btn_label).to_html()
                    button_width=ButtonWidth::Auto
                    loading=move || in_progress.get() == InProgressType::Encode
                    on_click=on_encode_click
                    disabled=move || in_progress.get().is_active()
                />
                <Button
                    label=move || t!(i18n, url_page_decode_btn_label).to_html()
                    button_width=ButtonWidth::Auto
                    loading=move || in_progress.get() == InProgressType::Decode
                    on_click=on_decode_click
                    disabled=move || in_progress.get().is_active()
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
                    label=move || t!(i18n, copy_to_clipboard_btn_label).to_html()
                    button_width=ButtonWidth::Auto
                    loading=move || false
                    on_click=on_copy_click
                    disabled=move || in_progress.get().is_active()
                />

            </div>
        </div>
    }
}
