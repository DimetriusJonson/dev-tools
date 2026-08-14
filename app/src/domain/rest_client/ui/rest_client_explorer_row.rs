use std::time::Duration;

use leptos::html::Div;
use leptos::{ev, leptos_dom};
use leptos::{html::Input, prelude::*};
use web_sys::wasm_bindgen::JsCast;

use crate::components::layout::message_banner::{Messages, show_error};
use crate::domain::rest_client::model::request_params::RequestCommand;
use crate::domain::rest_client::model::rest_client_context::RestClientContext;
use crate::domain::rest_client::ui::request_popup_menu::RequestPopupMenu;
use crate::domain::rest_client::util::request_store::{
    RequestFieldKind, copy_stored_request, delete_stored_request, generate_request_id,
    set_stored_requests_ids, set_stored_value,
};
use crate::i18n::*;
use crate::{
    components::ui::{
        button::{Button, ButtonColor, ButtonHeight, ButtonTextSize, ButtonWidth},
        text_input::TextInput,
    },
    domain::rest_client::model::request_params::RequestInfo,
};

#[component]
pub fn RestClientExplorerRow(
    request: RwSignal<RequestInfo>,
    requests: ReadSignal<Vec<RwSignal<RequestInfo>>>,
    set_requests: WriteSignal<Vec<RwSignal<RequestInfo>>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let rc_context = use_context::<RestClientContext>().unwrap();

    let (popup_menu_show, set_popup_menu_show) = signal(0);
    let (edit_name_mode, set_edit_name_mode) = signal(false);
    let (edit_name, set_edit_name) = signal("".to_owned());
    let menu_ref = NodeRef::<Div>::new();

    let edit_name_ref = NodeRef::<Input>::new();

    let _ = leptos_dom::helpers::window_event_listener(ev::click, move |ev| {
        if let Some(popup_menu_show) = popup_menu_show.try_get()
            && popup_menu_show > 0
        {
            if let Some(Some(target_element)) = menu_ref.try_get()
                && let Some(clicked_target) = ev.target()
            {
                let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
                if !target_element.contains(Some(clicked_node)) {
                    set_popup_menu_show.set(0);
                }
            }
            return;
        }

        if let Some(edit_name_mode) = edit_name_mode.try_get_untracked()
            && edit_name_mode
            && let Some(Some(target_element)) = edit_name_ref.try_get()
            && let Some(clicked_target) = ev.target()
        {
            let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
            if !target_element.contains(Some(clicked_node)) {
                set_edit_name_mode.set(false);
            }
        }
    });

    view! {
        <div class="flex h-8 sm:h-10 items-center cursor-pointer p-1 sm:p-2 text-xs md:text-base"
            class=(["bg-sky-500/50"], move || request.read_untracked().id == rc_context.request.read().id)
            class=(["hover:bg-gray-600/50"], move || request.read_untracked().id != rc_context.request.read().id)
            on:click=move |_| {
                if rc_context.request.read_untracked().id != request.read_untracked().id {
                    rc_context.request.set(request.get());
                    set_edit_name_mode.set(false);
                }
            }
            on:contextmenu=move|e| {e.prevent_default();
                if rc_context.request.read_untracked().id != request.read_untracked().id {
                    rc_context.request.set(request.get());
                    set_edit_name_mode.set(false);
                    set_timeout(move || set_popup_menu_show.set(request.read_untracked().id), Duration::from_millis(250));
                } else {
                    set_popup_menu_show.set(request.read_untracked().id);
                }
            }
            >

            <Show when=move || request.read().id == rc_context.request.read().id && edit_name_mode.get()
                fallback=move || view!{
                        <span class={format!("rounded-xl h-4 sm:h-5 px-1 sm:px-2 pb-1 sm:pb-4 font-medium text-xs sm:text-sm {}", get_method_color(&request.read().method))}>{request.read().method.to_owned()}</span>
                        <span class="p-1 sm:p-2 w-full truncate">{request.read().display_name()}</span>

                        <Show when=move || request.read().id == rc_context.request.read().id>
                            <div class="relative px-1 sm:px-2" node_ref=menu_ref>
                                <Button
                                    title=move || "".to_owned()
                                    label=move || "...".to_owned()
                                    class_name="hover:bg-sky-500/80 w-8 h-5 pb-6".to_owned()
                                    button_width=ButtonWidth::Custom
                                    button_height=ButtonHeight::Custom
                                    text_size=ButtonTextSize::Sm
                                    color=ButtonColor::Custom
                                    loading=move || false
                                    on_click=move |_|{
                                        if rc_context.request.read_untracked().id != request.read_untracked().id {
                                            rc_context.request.set(request.get());
                                            set_timeout(move || set_popup_menu_show.set(request.read_untracked().id), Duration::from_millis(250));
                                        } else {
                                            set_popup_menu_show.set(request.read_untracked().id);
                                        }
                                    }
                                    disabled=move || false
                                />

                                <Show when=move || popup_menu_show.get() == request.read().id>
                                    <RequestPopupMenu class_name="absolute inset-0 z-50".to_owned()
                                        items=move || {vec![
                                                ("run", t_string!(i18n, rest_client_explorer_run_request), true),
                                                ("copyCUrl", t_string!(i18n, rest_client_explorer_copy_curl), false),
                                                ("copyCUrlWin", t_string!(i18n, rest_client_explorer_copy_curlwin), true),
                                                ("rename", t_string!(i18n, rest_client_explorer_rename_request), false),
                                                ("dublicate", t_string!(i18n, rest_client_explorer_dublicate_request), true),
                                                ("delete", t_string!(i18n, rest_client_explorer_delete_request), false),
                                                ]}
                                        on_selected=move |val:(&'static str, &'static str)| {
                                            match val.0 {
                                                "delete" => {
                                                    set_requests.write().retain(|r|r.read_untracked().id != request.read_untracked().id);

                                                    rc_context.request.set(RequestInfo::new_empty());
                                                    set_stored_requests_ids(rc_context.project.read_only(), &requests.read_untracked());
                                                    delete_stored_request(rc_context.project.read_untracked().as_str(), request.read_untracked().id);
                                                    set_popup_menu_show.set(0);
                                                },
                                                "rename" => {
                                                    set_edit_name_mode.set(true);
                                                    set_edit_name.set(rc_context.request.read_untracked().display_name());

                                                    set_timeout(move || {
                                                        if let Some(input) = edit_name_ref.get() {
                                                            input.focus().unwrap();
                                                            input.select();
                                                            set_popup_menu_show.set(0);
                                                        }
                                                    }, Duration::from_millis(250));
                                                },
                                                "dublicate" => {
                                                    if let Some(orig_request) = requests.read_untracked().iter().find(|r|r.read_untracked().id == request.read_untracked().id) {
                                                        let orig_request = orig_request.get_untracked();
                                                        let request = RequestInfo::new(
                                                            generate_request_id(rc_context.project.read_only()),
                                                            orig_request.project_id,
                                                            orig_request.url.to_owned(),
                                                            orig_request.name.to_owned(),
                                                            orig_request.method.to_owned(),
                                                        );
                                                        let orig_request_id = orig_request.id;

                                                        set_timeout(move || {
                                                            set_requests.write().push(RwSignal::new(request.clone()));

                                                            copy_stored_request(rc_context.project.read_untracked().as_str(), orig_request_id, rc_context.project.read_untracked().as_str(), request.id);
                                                            set_stored_requests_ids(rc_context.project.read_only(), &requests.read_untracked());

                                                            rc_context.request.set(request.clone());
                                                            set_popup_menu_show.set(0);
                                                        }, Duration::from_millis(250));
                                                    }
                                                },
                                                "run" => {
                                                    rc_context.request.write().command = RequestCommand::Run;
                                                },
                                                "copyCUrl" => {
                                                    rc_context.request.write().command = RequestCommand::CopyCUrl;
                                                },
                                                "copyCUrlWin" => {
                                                    rc_context.request.write().command = RequestCommand::CopyCUrlWin;
                                                },
                                                _ => ()
                                            }
                                        }
                                    />
                                </Show>
                            </div>
                        </Show>
                    }>
                {move || view! {
                    <TextInput
                        node_ref=edit_name_ref
                        name="request-name".to_owned()
                        class_name="w-full".to_owned()
                        placeholder=move || "Name".to_owned()
                        input_type="text".to_owned()
                        value=edit_name
                        set_value=set_edit_name
                        on_change=move |value: String| {
                            let val = value.trim().to_lowercase();

                            if val.is_empty() {
                                show_error(t_string!(i18n, rest_client_empty_request_name).to_owned(), messages);
                                return;
                            }

                            if requests.read_untracked().iter().filter(|r|r.read_untracked().id != request.read_untracked().id)
                                .any(|r|r.read_untracked().name.to_lowercase() == val) {
                                    show_error(t_string!(i18n, rest_client_already_exist_request).to_owned(), messages);
                                return;
                            }

                            requests.read_untracked().iter().filter(|r|r.read_untracked().id == request.read_untracked().id).for_each(|r|{
                                r.write().name = value.trim().to_owned();
                                set_stored_value(rc_context.project.read_only(), r.get_untracked().id, RequestFieldKind::Name, value.to_owned());
                            });
                            rc_context.request.write_untracked().name = value.to_owned();
                            set_edit_name_mode.set(false);
                        }
                        on_cancel_change=move |_| {
                            *set_edit_name.write_untracked() = rc_context.request.read_untracked().name.to_owned();
                            set_edit_name_mode.set(false);
                        }
                        />
                }}
            </Show>

        </div>
    }
}

fn get_method_color(method: &str) -> String {
    match method {
        "GET" => "bg-sky-500/50".to_owned(),
        "POST" => "bg-green-500/50".to_owned(),
        "PUT" => "bg-green-400/50".to_owned(),
        "DELETE" => "bg-red-500/50".to_owned(),
        "PATCH" => "bg-green-400/50".to_owned(),
        "HEAD" => "bg-gray-500/50".to_owned(),
        "OPTIONS" => "bg-gray-500/50".to_owned(),
        _ => "bg-sky-500".to_owned(),
    }
}
