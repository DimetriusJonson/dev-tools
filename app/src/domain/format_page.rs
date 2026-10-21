use std::time::Duration;

use gloo_net::http::Request;
use leptos::html::Div;
use leptos::task::spawn_local;
use leptos::{html, prelude::*};
use leptos_router::hooks::use_navigate;

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
use crate::i18n::*;

#[derive(PartialEq, Copy, Clone)]
enum InProgressType {
    None,
    Process,
    Escape,
    FormatFile,
}

impl InProgressType {
    fn is_active(self) -> bool {
        self != InProgressType::None
    }
}

#[component]
pub fn FormatPage(
    name: &'static str,
    lang: &'static str,
    mime_type: &'static str,
    format_file_service: &'static str,
    src_placeholder: impl Fn() -> String + Send + Sync + 'static,
    format_func: impl Fn(String, usize) -> Result<String, Box<dyn std::error::Error>> + 'static + Clone,
    escape_func: impl Fn(String) -> Result<String, Box<dyn std::error::Error>> + 'static + Clone,
    unescape_func: impl Fn(String) -> Result<String, Box<dyn std::error::Error>> + 'static + Clone,
) -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let share_storage = use_context::<ShareStorage>().expect("Cant get share storage context!");

    let src_placeholder_memo = Memo::new(move |_| src_placeholder());

    let code_lang = RwSignal::new(lang.to_owned());
    let (src, set_src) = signal("".to_owned());
    let (dst, set_dst) = signal("".to_owned());
    let (ident, set_ident) =
        signal(get_local_store_value(&build_store_key(name, "ident"), "4".to_owned()));
    let (in_progress, set_in_progress) = signal(InProgressType::None);
    let formatting = RwSignal::new(
        get_local_store_value(&build_store_key(name, "format"), "true".to_owned())
            .parse::<bool>()
            .unwrap_or_default(),
    );
    let unescaping = RwSignal::new(
        get_local_store_value(&build_store_key(name, "unescape"), "false".to_owned())
            .parse::<bool>()
            .unwrap_or_default(),
    );

    let file_input_ref: NodeRef<html::Input> = NodeRef::new();
    let left_panel_ref = NodeRef::<Div>::new();
    let right_panel_ref = NodeRef::<Div>::new();

    Effect::new(move |_| {
        let src_value = get_local_store_value(&build_store_key(name, "src"), "".to_owned());
        set_timeout(
            move || {
                set_src.set(src_value);
            },
            Duration::from_millis(50),
        );
    });

    Effect::watch(
        move || src.get(),
        move |value, _prev, _| {
            set_local_store_value(&build_store_key(name, "src"), &value);
        },
        false,
    );

    Effect::watch(
        move || formatting.get(),
        move |value, _prev, _| {
            set_local_store_value(&build_store_key(name, "format"), &value.to_string());
        },
        false,
    );

    Effect::watch(
        move || unescaping.get(),
        move |value, _prev, _| {
            set_local_store_value(&build_store_key(name, "unescape"), &value.to_string());
        },
        false,
    );

    let on_process_click = move |_| {
        let unescape_func = unescape_func.clone();
        let format_func = format_func.clone();
        spawn_local(async move {
            set_in_progress.set(InProgressType::Process);

            let mut result = src.get_untracked();
            if unescaping.get_untracked() {
                match unescape_func(result) {
                    Ok(unescaped_value) => result = unescaped_value,
                    Err(err) => {
                        show_error(err.to_string(), messages);
                        set_in_progress.set(InProgressType::None);
                        return;
                    }
                };
            };

            if formatting.get_untracked() {
                match format_func(result, ident.get_untracked().parse().unwrap_or(4)) {
                    Ok(formatted_value) => result = formatted_value,
                    Err(err) => {
                        show_error(err.to_string(), messages);
                        set_in_progress.set(InProgressType::None);
                        return;
                    }
                };
            }

            set_dst.set(result);

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
                match Request::post(format_file_service)
                    .header("content-type", mime_type)
                    .body(&file)
                {
                    Ok(request) => match request.send().await {
                        Ok(response) => match response.binary().await {
                            Ok(bytes) => {
                                let file_name = format!("formatted_{}", file.name());
                                match save_file_to_disk(bytes.to_vec(), &file_name, mime_type) {
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

    let on_escape_click = move |_| {
        let escape_func = escape_func.clone();
        spawn_local(async move {
            set_in_progress.set(InProgressType::Escape);

            match escape_func(src.get_untracked()) {
                Ok(unescaped_value) => set_dst.set(unescaped_value),
                Err(err) => show_error(err.to_string(), messages),
            };
            set_in_progress.set(InProgressType::None);
        });
    };

    let on_copy_click = move |_| {
        copy_to_clipboard(&dst.get());
        show_info(t_string!(i18n, format_page_copied_to_clipboard_msg).to_owned(), messages);
    };

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            // LEFT SIDE
            <div node_ref=left_panel_ref class="sm:flex-none flex flex-col md:flex-row h-[29dvh] md:h-[90dvh] gap-4">
                <div class="md:flex-1 overflow-y-auto flex flex-col gap-4 w-full ">
                    { move || view! {
                        <div class=format!("md:flex-1 h-[30dvh] md:h-auto overflow-y-auto w-full resize-none {}", TEXTAREA_BORDER_CLASSES)>
                            <CodeMirrorEditor
                                element_id=format!("{}-src-code-editor", name)
                                placeholder=Box::new(move || src_placeholder_memo.get())
                                lang=code_lang.read_only()
                                value=src
                                set_value=set_src
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
                                set_local_store_value(&build_store_key(name, "ident"), &value);
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
                            name="Unescape".to_owned()
                            value=unescaping
                            label=move || "Unescape".to_owned()
                        />

                        <Button
                            title=move || "".to_owned()
                            label=move || "".to_owned()
                            label_type=ButtonLabel::RightArrow
                            button_width=ButtonWidth::Lg
                            loading=move || in_progress.get() == InProgressType::Process
                            on_click=on_process_click
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
                local_store_prop_name=move || format!("{}_left_panel_width", name)
                min_ratio={25.0}
                max_ratio={75.0}
                default_ratio={50.0}/>

            // RIGHT SIDE
            <div node_ref=right_panel_ref class="overflow-y-auto flex flex-col gap-4 h-[35dvh] md:h-[90dvh]">
                { move || view! {
                    <div class=format!("flex-1 overflow-y-auto text-black dark:text-white {}", TEXTAREA_BORDER_CLASSES)>
                        <CodeMirrorEditor
                            element_id=format!("{}-code-editor", name)
                            lang=code_lang.read_only()
                            value=dst
                            set_value=set_dst
                            read_only=true
                        />
                    </div>
                    }
                }

                <div class="flex gap-4 justify-between" class:hidden=move || dst.read().is_empty()>
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
                            share_storage.set_text(&dst.read_untracked());
                            navigate(&format!("/share_file?mode={}", name), Default::default());
                        }
                    />
                </div>

            </div>
        </div>
    }
}

fn build_store_key(name: &'static str, key: &'static str) -> String {
    format!("{}_{}", name, key)
}
