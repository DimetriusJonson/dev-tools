use std::collections::HashMap;

use leptos::{
    ev,
    html::Div,
    leptos_dom::{self, logging::console_log},
    prelude::*,
};
use web_sys::wasm_bindgen::JsCast;

use crate::{
    common::local_store::{get_local_store_value, set_local_store_value},
    components::ui::button::{Button, ButtonWidth},
    domain::rest_client::ui::{request_params::RequestInfo, request_popup_menu::RequestPopupMenu},
};

#[component]
pub fn RestClientExplorer(
    current_request: ReadSignal<RequestInfo>,
    set_current_request: WriteSignal<RequestInfo>,
) -> impl IntoView {
    let (requests, set_requests) = signal(Vec::<RwSignal<RequestInfo>>::new());
    let (popup_menu_show, set_popup_menu_show) = signal(0);
    let (menu_refs, set_menu_refs) = signal(HashMap::<i32, NodeRef<Div>>::new());

    let on_create_request = move |_| {
        let request = RequestInfo {
            id: generate_request_id(),
            url: format!("http://{}/test_json", window().location().host().unwrap()),
            method: "GET".to_owned(),
        };

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
        if popup_menu_show.get() == 0 {
            return;
        }

        if let Some(target_ref) = menu_refs.read_untracked().get(&popup_menu_show.get_untracked()) {
            if let Some(target_element) = target_ref.get() {
                if let Some(clicked_target) = ev.target() {
                    let clicked_node: &web_sys::Node = clicked_target.unchecked_ref();
                    if !target_element.contains(Some(clicked_node)) {
                       set_popup_menu_show.set(0);
                    }
                }
            }
        }
    });

    view! {
        <div class="flex flex-col gap-y-4 dark:text-white border-r-2 border-gray-700 w-64">
            <div class="p-4">
                <Button
                    label=move || "Create Request".to_owned()
                    class_name="w-full".to_owned()
                    button_width=ButtonWidth::Lg
                    loading=move || false
                    on_click=on_create_request
                    disabled=move || false
                />
            </div>

            { move || { requests.read().iter()
                .map(|request| {
                    let request_cloned = request.get();
                    view! {
                        <div class="group flex w-full h-10 items-center hover:bg-sky-500/50 cursor-pointer p-2"
                            class=(["bg-sky-500/50"], move || request_cloned.id == current_request.read().id)
                            on:click={
                                let request_cloned = request.get();
                                move |_| {
                                    set_current_request.set(request_cloned.clone());
                                }
                            }
                            >
                            <span class="bg-sky-500 p-2 rounded-xl">{request.get().method}</span>
                            <span class="p-2 w-full truncate">{request.get().url}</span>
                            <div class="relative px-2 hidden group-hover:block z-50" node_ref={*menu_refs.read().get(&request_cloned.id).unwrap()}>
                                <Button
                                    label=move || "...".to_owned()
                                    button_width=ButtonWidth::Auto
                                    loading=move || false
                                    on_click=move |_|{
                                        set_popup_menu_show.set(request_cloned.id);
                                    }
                                    disabled=move || false
                                />
                                <RequestPopupMenu class_name="absolute inset-0".to_owned()
                                    show=move || popup_menu_show.get() == request_cloned.id
                                    items=move || {vec![
                                            ("run".to_owned(), "Run request".to_owned()),
                                            ("rename".to_owned(), "Rename".to_owned()),
                                            ("delete".to_owned(), "Delete".to_owned()),
                                            ]}
                                    on_selected=move |val:(String, String)| {
                                        set_popup_menu_show.set(0);
                                        console_log(&val.0);
                                    }
                                    />
                            </div>
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
            let method = get_local_store_value(&format!("{}-rc_method", id), "".to_owned());
            RwSignal::new(RequestInfo { id: *id, url, method })
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
