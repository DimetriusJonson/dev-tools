use std::time::Duration;

use gloo_net::http::Request;
use json_escape::unescape;
use leptos::html::Div;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};
use leptos_router::hooks::use_navigate;

use crate::common::json_processor::format_json;
use crate::common::local_store::{get_local_store_value, set_local_store_value};
use crate::common::ui_utils::{copy_to_clipboard, save_file_to_disk};
use crate::components::layout::drag_splitter::DragSplitter;
use crate::components::layout::message_banner::{Messages, show_error, show_info};
use crate::components::ui::button::{Button, ButtonLabel, ButtonWidth};
use crate::components::ui::checkbox_with_label::CheckboxWithLabel;
use crate::components::ui::code_mirror_editor::CodeMirrorEditor;
use crate::components::ui::file_input::FileInput;
use crate::components::ui::select_input::SelectInput;
use crate::components::ui::text_area::TEXTAREA_BORDER_CLASSES;
use crate::domain::share_file::ShareStorage;
use crate::i18n::use_i18n;
use crate::i18n::*;
use json_escape::escape_str;

#[derive(PartialEq, Copy, Clone)]
enum InProgressType {
    None,
    Format,
    FormatFile,
    Escape,
}

impl InProgressType {
    fn is_active(self) -> bool {
        self != InProgressType::None
    }
}

#[component]
pub fn JsonPage() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let share_storage = use_context::<ShareStorage>().expect("Cant get share storage context!");

    let code_lang = RwSignal::new("json".to_owned());
    let (json, set_json) = signal(get_local_store_value("src_json", "".to_owned()));
    let (dst_json, set_dst_json) = signal("".to_owned());
    let (ident, set_ident) = signal(get_local_store_value("json_ident", "4".to_owned()));
    let (in_progress, set_in_progress) = signal(InProgressType::None);
    let formatting = RwSignal::new(
        get_local_store_value("json_format", "true".to_owned())
            .parse::<bool>()
            .unwrap_or_default(),
    );
    let unescaping = RwSignal::new(
        get_local_store_value("json_unescape", "false".to_owned())
            .parse::<bool>()
            .unwrap_or_default(),
    );

    let file_input_ref: NodeRef<html::Input> = NodeRef::new();
    let left_panel_ref = NodeRef::<Div>::new();
    let right_panel_ref = NodeRef::<Div>::new();

    Effect::new(move |_| {
        let src_json = get_local_store_value("src_json", "".to_owned());
        set_timeout(
            move || {
                set_json.set(src_json);
            },
            Duration::from_millis(50),
        );
    });

    Effect::watch(
        move || json.get(),
        move |value, _prev, _| {
            set_local_store_value("src_json", &value);
        },
        false,
    );

    Effect::watch(
        move || formatting.get(),
        move |value, _prev, _| {
            set_local_store_value("json_format", &value.to_string());
        },
        false,
    );

    Effect::watch(
        move || unescaping.get(),
        move |value, _prev, _| {
            set_local_store_value("json_unescape", &value.to_string());
        },
        false,
    );

    let on_process_json_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Format);

            let mut json_result = json.get_untracked();

            if unescaping.get_untracked() {
                match unescape(&json_result).decode_utf8() {
                    Ok(unescaped_json) => json_result = unescaped_json.to_string(),
                    Err(err) => {
                        show_error(err.to_string(), messages);
                        set_in_progress.set(InProgressType::None);
                        return;
                    }
                }
            }

            if formatting.get_untracked() {
                json_result = format_json(&json_result, ident.get_untracked().parse().unwrap_or(4));
            }
            set_dst_json.set(json_result);

            set_in_progress.set(InProgressType::None);
        });
    };

    let on_escape_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::Escape);

            let json_str = json.read_untracked();
            let escaped_parts = escape_str(json_str.as_str());
            let mut escaped_str = String::new();
            for part in escaped_parts {
                escaped_str.push_str(part);
            }
            set_dst_json.set(escaped_str);

            set_in_progress.set(InProgressType::None);
        });
    };

    let on_format_file_click = move |_| {
        spawn_local(async move {
            set_in_progress.set(InProgressType::FormatFile);

            if let Some(file_input) = file_input_ref.get_untracked()
                && let Some(files) = file_input.files()
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
                                    Ok(_) => show_info(
                                        t_display!(i18n, file_saved_file_msg, file_name)
                                            .to_string(),
                                        messages,
                                    ),
                                    Err(err) => show_error(err, messages),
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

    let on_copy_click = move |_| {
        copy_to_clipboard(&dst_json.get());
        show_info(t_string!(i18n, json_page_copied_to_clipboard_msg).to_owned(), messages);
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            // LEFT SIDE
            <div node_ref=left_panel_ref class="sm:flex-none flex flex-col md:flex-row h-[29dvh] md:h-[90dvh] gap-4">
                <div class="md:flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 w-full">
                    { move || view! {
                        <div class=format!("md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none {}", TEXTAREA_BORDER_CLASSES)>
                            <CodeMirrorEditor
                                element_id="json-src-code-editor".to_owned()
                                placeholder=Box::new(move || {t_string!(i18n, json_page_src_placeholder).to_owned()})
                                lang=code_lang.read_only()
                                value=json
                                set_value=set_json
                                read_only=false
                            />
                        </div>
                        }
                    }

                    <div class="flex flex-row">
                        <FileInput node_ref=file_input_ref />
                        <Button
                            title=move || "".to_owned()
                            label=move || t_string!(i18n, format_file_btn_label).to_owned()
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
                            label=move || t_string!(i18n, ident_label).to_owned()
                            not_selected_text=move || "".to_owned()
                            options=move || {vec![
                                (Some("2".to_owned()), t_string!(i18n, ident_option_label_2).to_owned()),
                                (Some("3".to_owned()), t_string!(i18n, ident_option_label_3).to_owned()),
                                (Some("4".to_owned()), t_string!(i18n, ident_option_label_4).to_owned())
                                ]}
                            on_change=move |value: String| {
                                set_local_store_value("json_ident", &value);
                            }
                            value=ident
                            set_value=set_ident
                        />

                        <CheckboxWithLabel
                            name="format".to_owned()
                            value=formatting
                            label=move || t_string!(i18n, formatting_label).to_owned()
                        />

                        <CheckboxWithLabel
                            name="unescape".to_owned()
                            value=unescaping
                            label=move || "Unescape".to_owned()
                        />

                        <Button
                            title=move || "".to_owned()
                            label=move || "".to_owned()
                            label_type=ButtonLabel::RightArrow
                            button_width=ButtonWidth::Lg
                            loading=move || in_progress.get() == InProgressType::Format
                            on_click=on_process_json_click
                            disabled=move || in_progress.get().is_active()
                        />
                    </div>

                    <div class="flex flex-row md:flex-col gap-4 md:py-8">
                        <Button
                            title=move || "".to_owned()
                            label=move || "Escape".to_owned()
                            button_width=ButtonWidth::Lg
                            loading=move || in_progress.get() == InProgressType::Escape
                            on_click=on_escape_click
                            disabled=move || in_progress.get().is_active()
                        />
                    </div>
                </div>
            </div>

            <DragSplitter
                target_ref=left_panel_ref
                second_target_ref=right_panel_ref
                class_name="hidden md:block".to_owned()
                local_store_prop_name=move || "json_left_panel_width".to_owned()
                min_ratio={25.0}
                max_ratio={75.0}
                default_ratio={50.0}/>

            // RIGHT SIDE
            <div node_ref=right_panel_ref class="min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[35dvh] md:h-[90dvh]">
                { move || view! {
                    <div class=format!("flex-1 min-h-0 overflow-y-auto text-black dark:text-white {}", TEXTAREA_BORDER_CLASSES)>
                        <CodeMirrorEditor
                            element_id="json-code-editor".to_owned()
                            lang=code_lang.read_only()
                            value=dst_json
                            set_value=set_dst_json
                            read_only=true
                        />
                    </div>
                    }
                }

                <div class="flex gap-4 justify-between" class:hidden=move || dst_json.read().is_empty()>
                    <Button
                        title=move || "".to_owned()
                        label=move || t_string!(i18n, copy_to_clipboard_btn_label).to_owned()
                        button_width=ButtonWidth::Auto
                        loading=move || false
                        on_click=on_copy_click
                        disabled=move || in_progress.get().is_active()
                    />

                    <Button
                        title=move || "".to_owned()
                        label=move || t_display!(i18n, share_file_btn_label).to_string()
                        button_width=ButtonWidth::Lg
                        loading=move || false
                        disabled=move || false
                        on_click=move |_| {
                            share_storage.set_text(&dst_json.read_untracked());
                            navigate("/share_file?mode=xml", Default::default());
                        }
                    />
                </div>

            </div>
        </div>
    }
}
