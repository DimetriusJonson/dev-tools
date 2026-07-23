use leptos::prelude::*;

use crate::{
    common::local_store::{get_local_store_value, set_local_store_value},
    components::ui::button::{Button, ButtonWidth},
    domain::rest_client::ui::request_params::RequestInfo,
};

#[component]
pub fn RestClientExplorer(
    current_request: ReadSignal<Option<RequestInfo>>,
    set_current_request: WriteSignal<Option<RequestInfo>>,
) -> impl IntoView {
    let (requests, set_requests) = signal(vec![]);

    let on_create_request = move |_| {
        let request = RequestInfo {
            id: generate_request_id(),
            url: "https://test.com/api".to_owned(),
            method: "GET".to_owned(),
        };

        set_requests.write().push(request.clone());
        set_current_request.set(Some(request.clone()));
        save_requests_ids(&requests.read_untracked());
        set_local_store_value(&format!("{}-rc_url", request.id), request.url);
        set_local_store_value(&format!("{}-rc_method", request.id), request.method);
    };

    let _ = Effect::new(move || {
        set_requests.set(load_requests());
    });

    view! {
        <div class="flex flex-col p-0 dark:text-white border-r-2 border-gray-700">

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

            <For
                each=move || requests.get()
                key=|request| request.id
                children=move |request| {
                    let request_cloned = request.clone();
                    view! {
                        <div class="flex w-full h-10 items-center justify-center hover:bg-slate-500/50 cursor-pointer p-2"
                        on:click={
                            move |_| {
                                set_current_request.set(Some(request_cloned.clone()));
                            }
                        }
                        >
                            <span class="bg-sky-500">{request.method}</span><span class="pl-2">{request.url}</span>
                        </div>
                    }
                }
            />
        </div>
    }
}

fn generate_request_id() -> i32 {
    let requests_ids = load_requests_ids();
    if !requests_ids.is_empty() {
        if let Some(id) = requests_ids.iter().max() {
            return *id + 1;
        }
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

fn load_requests() -> Vec<RequestInfo> {
    load_requests_ids()
        .iter()
        .map(|id| {
            let url = get_local_store_value(&format!("{}-rc_url", id), "".to_owned());
            let method = get_local_store_value(&format!("{}-rc_method", id), "".to_owned());
            RequestInfo { id: *id, url, method }
        })
        .collect()
}

fn save_requests_ids(requests: &Vec<RequestInfo>) {
    let value = requests.iter().map(|r| r.id.to_string()).collect::<Vec<String>>().join(",");
    set_local_store_value("rc_requests_ids", value);
}
