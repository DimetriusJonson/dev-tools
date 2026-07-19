use gloo_net::http::Request;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};

use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::{copy_to_clipboard, save_file_to_disk};
use crate::common::xml_processor::{escape_xml, format_xml};
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonWidth};
use crate::components::ui::code_inner::CodeInner;
use crate::components::ui::file_input::FileInput;
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TextArea;

use crate::i18n::*;

#[derive(PartialEq, Copy, Clone)]
enum InProgressType {
    None,
    Format,
    FormatFile,
    Escape,
    Unescape,
}

impl InProgressType {
    fn is_active(self) -> bool {
        self != InProgressType::None
    }
}

#[component]
pub fn XmlPage() -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let (xml, set_xml) = signal(get_local_store_value("src_xml", "".to_owned()));
    let (dst_xml, set_dst_xml) = signal("".to_owned());
    let (ident, set_ident) = signal(get_local_store_value("xml_ident", "4".to_owned()));
    let (in_progress, set_in_progress) = signal(InProgressType::None);

    let file_input_ref: NodeRef<html::Input> = NodeRef::new();

    let on_format_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Format);

            match format_xml(xml.read_untracked().as_str(), ident.get_untracked().parse().unwrap())
            {
                Ok(formatted_xml) => set_dst_xml.set(formatted_xml),
                Err(err) => show_error(err.to_string(), messages),
            };

            set_in_progress.set(InProgressType::None);
        });
    };

    let on_format_file_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::FormatFile);

            let file_input = file_input_ref.get_untracked().expect("input to exist");
            if let Some(files) = file_input.files()
                && let Some(file) = files.get(0)
            {
                match Request::post("/format_xml")
                    .header("content-type", "application/xml")
                    .body(&file)
                {
                    Ok(request) => match request.send().await {
                        Ok(response) => match response.binary().await {
                            Ok(bytes) => {
                                let file_name = format!("formatted_{}", file.name());
                                match save_file_to_disk(
                                    bytes.to_vec(),
                                    &file_name,
                                    "application/xml",
                                ) {
                                    Ok(_) => show_info(
                                        t_display!(i18n, file_saved_file_msg, file_name)
                                            .to_string(),
                                        messages,
                                    ),
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

            set_in_progress.set(InProgressType::None);
        });
    };

    let on_unescape_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Unescape);

            match escape_xml(xml.read_untracked().as_str(), false)
            {
                Ok(unescaped_xml) => set_dst_xml.set(unescaped_xml),
                Err(err) => show_error(err.to_string(), messages),
            };

            set_in_progress.set(InProgressType::None);
        });
    };

    let on_escape_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Escape);

            match escape_xml(xml.read_untracked().as_str(), true)
            {
                Ok(unescaped_xml) => set_dst_xml.set(unescaped_xml),
                Err(err) => show_error(err.to_string(), messages),
            };
            set_in_progress.set(InProgressType::None);
        });
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&dst_xml.get());
        show_info(t!(i18n, xml_page_copied_to_clipboard_msg).to_html(), messages);
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[29dvh] md:h-[90dvh]">
                <TextArea
                    name="xml".to_owned()
                    class_name="md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none".to_owned()
                    placeholder=move || {t!(i18n, xml_page_src_placeholder).to_html()}
                    value=xml
                    set_value=set_xml
                    on_change=move |_| {
                        set_local_store_value("src_xml", xml.get_untracked());
                    }
                />
                <div class="flex flex-row">
                    <FileInput node_ref=file_input_ref />
                    <Button
                        label=move || t!(i18n, xml_page_format_btn_label).to_html()
                        button_width=ButtonWidth::Lg
                        loading=move || in_progress.get() == InProgressType::FormatFile
                        on_click=on_format_file_click
                        disabled=move || in_progress.get().is_active()
                    />
                </div>
            </div>

            <div class="flex flex-col gap-4 items-center justify-center">
                <div class="flex flex-row md:flex-col gap-4">
                    <SelectInput
                        name="ident".to_owned()
                        label=move || t!(i18n, ident_label).to_html()
                        not_selected_text=move || "".to_owned()
                        options=move || {vec![
                            (Some("2".to_owned()), t!(i18n, ident_option_label_2).to_html()),
                            (Some("3".to_owned()), t!(i18n, ident_option_label_3).to_html()),
                            (Some("4".to_owned()), t!(i18n, ident_option_label_4).to_html())
                            ]}
                        on_change=move |value| {
                            set_local_store_value("xml_ident", value);
                        }
                        value=ident
                        set_value=set_ident
                    />

                    <Button
                        label=move || t!(i18n, xml_page_format_btn_label).to_html()
                        button_width=ButtonWidth::Lg
                        loading=move || in_progress.get() == InProgressType::Format
                        on_click=on_format_click
                        disabled=move || in_progress.get().is_active()
                    />
                </div>

                <div class="flex flex-row md:flex-col gap-4 md:py-8">
                    <Button
                        label=move || t!(i18n, xml_page_unescape_btn_label).to_html()
                        button_width=ButtonWidth::Lg
                        loading=move || in_progress.get() == InProgressType::Unescape
                        on_click=on_unescape_click
                        disabled=move || in_progress.get().is_active()
                    />

                    <Button
                        label=move || t!(i18n, xml_page_escape_btn_label).to_html()
                        button_width=ButtonWidth::Lg
                        loading=move || in_progress.get() == InProgressType::Escape
                        on_click=on_escape_click
                        disabled=move || in_progress.get().is_active()
                    />
                </div>
            </div>

            <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[35dvh] md:h-[90dvh]">
                { move || view! {
                    <div class="flex-1 min-h-0 overflow-y-auto text-black dark:text-white px-3 py-2 rounded-md shadow-inner border bg-white dark:bg-dark-bg border-gray-300 dark:border-gray-700">
                        <CodeInner code={dst_xml.get()} lang="xml".to_string()/>
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
