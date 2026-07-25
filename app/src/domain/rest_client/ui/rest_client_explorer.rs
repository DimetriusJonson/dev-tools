use std::{collections::HashMap, time::Duration};

use leptos::{
    ev,
    html::{Div, Input},
    leptos_dom::{self},
    prelude::*,
};
use web_sys::wasm_bindgen::JsCast;

use crate::i18n::*;
use crate::{
    common::local_store::{delete_local_store_value, get_local_store_value, set_local_store_value},
    components::ui::{
        button::{Button, ButtonColor, ButtonHeight, ButtonTextSize, ButtonWidth},
        text_input::TextInput,
    },
    domain::rest_client::ui::{request_params::RequestInfo, request_popup_menu::RequestPopupMenu},
};

#[component]
pub fn RestClientExplorer(
    current_request: ReadSignal<RequestInfo>,
    set_current_request: WriteSignal<RequestInfo>,
) -> impl IntoView {
    let i18n = use_i18n();

    let (requests, set_requests) = signal(Vec::<RwSignal<RequestInfo>>::new());
    let (popup_menu_show, set_popup_menu_show) = signal(0);
    let (edit_name_mode, set_edit_name_mode) = signal(false);
    let (edit_name, set_edit_name) = signal("".to_owned());
    let (menu_refs, set_menu_refs) = signal(HashMap::<i32, NodeRef<Div>>::new());
    let edit_name_ref = NodeRef::<Input>::new();

    let on_create_request = move |_| {
        let request = RequestInfo::new(
            generate_request_id(),
            format!("http://{}/test_json", window().location().host().unwrap()),
            "".to_owned(),
            "GET".to_owned(),
        );

        set_requests.write().push(RwSignal::new(request.clone()));
        set_menu_refs.write().insert(request.id, NodeRef::<Div>::new());

        set_current_request.set(request.clone());
        save_requests_ids(&requests.read_untracked());
        set_local_store_value(&format!("{}-rc_url", request.id), request.url);
        set_local_store_value(&format!("{}-rc_method", request.id), request.method);
    };

    let _ = Effect::new(move || {
        set_requests.set(load_requests());
        set_menu_refs.write().clear();
        for request in requests.read_untracked().iter() {
            set_menu_refs.write().insert(request.read_untracked().id, NodeRef::<Div>::new());
        }
    });

    Effect::watch(
        move || current_request.get(),
        move |value, _prev, _| {
            if let Some(req) =
                requests.read_untracked().iter().find(|r| r.read_untracked().id == value.id)
            {
                req.write().url = value.url.to_owned();
                req.write().method = value.method.to_owned();
            }
        },
        false,
    );

    let _ = leptos_dom::helpers::window_event_listener(ev::click, move |ev| {
        if popup_menu_show.get() > 0 {
            if let Some(target_ref) =
                menu_refs.read_untracked().get(&popup_menu_show.get_untracked())
            {
                if let Some(target_element) = target_ref.get() {
                    if let Some(clicked_target) = ev.target() {
                        let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
                        if !target_element.contains(Some(clicked_node)) {
                            set_popup_menu_show.set(0);
                        }
                    }
                }
            }
            return;
        }

        if edit_name_mode.get_untracked() {
            if let Some(target_element) = edit_name_ref.get() {
                if let Some(clicked_target) = ev.target() {
                    let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
                    if !target_element.contains(Some(clicked_node)) {
                        set_edit_name_mode.set(false);
                    }
                }
            }
        }
    });

    view! {
        <div class="flex flex-col gap-y-0 dark:text-white border-r-2 border-gray-700 w-64">
            <div class="p-4">
                <Button
                    label=move || t_display!(i18n, rest_client_explorer_create_request).to_string()
                    class_name="w-full".to_owned()
                    button_width=ButtonWidth::Lg
                    loading=move || false
                    on_click=on_create_request
                    disabled=move || false
                />
            </div>

            { move || { requests.get().into_iter()
                .map(|request| {
                    let request_cloned = request.get();

                    view! {
                        <div class="flex w-full h-10 items-center cursor-pointer p-2"
                            class=(["bg-sky-500/50"], move || request_cloned.id == current_request.read().id)
                            class=(["hover:bg-gray-600/
                            
                            
                            
                            50"], move || request_cloned.id != current_request.read().id)
                            on:click={
                                let request_cloned = request.get();
                                move |_| {
                                    if current_request.read_untracked().id != request_cloned.id {
                                        set_current_request.set(request_cloned.clone());
                                        set_edit_name_mode.set(false);
                                    }
                                }
                            }
                            on:contextmenu={
                                let request_cloned = request.get();
                                move|e| {e.prevent_default();
                                    if current_request.read_untracked().id != request_cloned.id {
                                        set_current_request.set(request_cloned.clone());
                                        set_edit_name_mode.set(false);
                                        set_timeout(move || set_popup_menu_show.set(request_cloned.id), Duration::from_millis(250));
                                    } else {
                                        set_popup_menu_show.set(request_cloned.id);
                                    }
                                }}
                            >

                            <Show when=move || request_cloned.id == current_request.read().id && edit_name_mode.get()
                                fallback={
                                    let request_cloned = request.get();
                                    move || view!{
                                        <span class={format!("rounded-xl h-5 px-2 pb-4 font-medium text-sm {}", get_method_color(&request_cloned.method))}>{request_cloned.method.to_owned()}</span>
                                        <span class="p-2 w-full truncate">{request_cloned.display_name()}</span>

                                        <Show when=move || request_cloned.id == current_request.read().id>
                                            <div class="relative px-2" node_ref={*menu_refs.read().get(&request_cloned.id).unwrap()}>
                                                <Button
                                                    label=move || "...".to_owned()
                                                    class_name="hover:bg-sky-500/80 w-8 h-5 pb-6".to_owned()
                                                    button_width=ButtonWidth::Custom
                                                    button_height=ButtonHeight::Custom
                                                    text_size=ButtonTextSize::Sm
                                                    color=ButtonColor::Custom
                                                    loading=move || false
                                                    on_click={
                                                        let request_cloned = request.get();
                                                        move |_|{
                                                            if current_request.read_untracked().id != request_cloned.id {
                                                                set_current_request.set(request_cloned.clone());
                                                                set_timeout(move || set_popup_menu_show.set(request_cloned.id), Duration::from_millis(250));
                                                            } else {
                                                                set_popup_menu_show.set(request_cloned.id);
                                                            }
                                                        }}
                                                    disabled=move || false
                                                />

                                                <Show when=move || popup_menu_show.get() == request_cloned.id>
                                                    {
                                                        view! {
                                                        <RequestPopupMenu class_name="absolute inset-0 z-50".to_owned()
                                                            items=move || {vec![
                                                                    ("run".to_owned(), t_display!(i18n, rest_client_explorer_run_request).to_string()),
                                                                    ("rename".to_owned(), t_display!(i18n, rest_client_explorer_rename_request).to_string()),
                                                                    ("delete".to_owned(), t_display!(i18n, rest_client_explorer_delete_request).to_string()),
                                                                    ]}
                                                            on_selected={
                                                                let request_cloned=request.get();
                                                                move |val:(String, String)| {
                                                                    match val.0.as_str() {
                                                                        "delete" => {
                                                                            set_requests.write().retain(|r|r.read_untracked().id != request_cloned.id);
                                                                            set_menu_refs.write().remove(&request_cloned.id);

                                                                            set_current_request.set(RequestInfo::new_empty());
                                                                            save_requests_ids(&requests.read_untracked());
                                                                            delete_local_store_value(&format!("{}-rc_url", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_name", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_method", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_body", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_content_type", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_accept", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_accept_lang", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_user_agent", request_cloned.id));
                                                                            delete_local_store_value(&format!("{}-rc_custom_headers", request_cloned.id));
                                                                            set_popup_menu_show.set(0);
                                                                        },
                                                                        "rename" => {
                                                                            set_edit_name_mode.set(true);
                                                                            set_edit_name.set(current_request.read_untracked().display_name());

                                                                            set_timeout(move || {
                                                                                if let Some(input) = edit_name_ref.get() {
                                                                                    input.focus().unwrap();
                                                                                    input.select();
                                                                                    set_popup_menu_show.set(0);
                                                                                }
                                                                            }, Duration::from_millis(250));
                                                                        },
                                                                        "run" => {
                                                                            set_current_request.set(request_cloned.clone_and_run());
                                                                        },
                                                                        _ => ()
                                                                    }
                                                                }
                                                            }
                                                        />
                                                    }}
                                                </Show>
                                            </div>
                                        </Show>
                                    }}>
                                {move || view! {
                                    <TextInput node_ref=edit_name_ref
                                        name="request-name".to_owned()
                                        class_name="w-full".to_owned()
                                        placeholder=move || "Name".to_owned()
                                        input_type="text".to_owned()
                                        value=edit_name
                                        set_value=set_edit_name
                                        on_change=move |v: String| {
                                            requests.read_untracked().iter().filter(|r|r.read_untracked().id == request_cloned.id).for_each(|r|{
                                                r.write().name = v.to_owned();
                                                set_local_store_value(
                                                    &format!("{}-{}", r.get_untracked().id, "rc_name"),
                                                    v.to_owned(),
                                                );
                                            });
                                        }
                                        />
                                }}
                            </Show>

                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}

fn generate_request_id() -> i32 {
    let requests_ids = load_requests_ids();
    if !requests_ids.is_empty()
        && let Some(id) = requests_ids.iter().max()
    {
        return *id + 1;
    }

    1
}

fn load_requests_ids() -> Vec<i32> {
    let requests_ids = get_local_store_value("rc_requests_ids", "".to_owned());
    if !requests_ids.is_empty() {
        requests_ids.split(",").map(|s| s.parse::<i32>().unwrap()).collect()
    } else {
        vec![]
    }
}

fn load_requests() -> Vec<RwSignal<RequestInfo>> {
    load_requests_ids()
        .iter()
        .map(|id| {
            let url = get_local_store_value(&format!("{}-rc_url", id), "".to_owned());
            let name = get_local_store_value(&format!("{}-rc_name", id), "".to_owned());
            let method = get_local_store_value(&format!("{}-rc_method", id), "".to_owned());
            RwSignal::new(RequestInfo::new(*id, url, name, method))
        })
        .collect()
}

fn save_requests_ids(requests: &[RwSignal<RequestInfo>]) {
    let value = requests
        .iter()
        .map(|r| r.read_untracked().id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    set_local_store_value("rc_requests_ids", value);
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
