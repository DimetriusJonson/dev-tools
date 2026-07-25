use std::cmp::max;

use leptos::{
    ev,
    html::Div,
    leptos_dom::{self},
    prelude::*,
};

use crate::{
    common::{
        local_store::{get_local_store_value, set_local_store_value},
        ui_utils::get_browser_width,
    },
    domain::rest_client::ui::{
        request_panel::RequestPanel, request_params::RequestInfo,
        rest_client_explorer::RestClientExplorer,
    },
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let screen_width = get_browser_width().unwrap();
    let min_explorer_width = max(256, screen_width / 6);

    let (current_request, set_current_request) = signal(RequestInfo::new_empty());
    let (width, set_width) = signal(
        get_local_store_value("rc_explorer_width", min_explorer_width.to_string()).parse::<i32>().unwrap(),
    );
    let (drag_mode, set_drag_mode) = signal(false);

    let explorer_node_ref = NodeRef::<Div>::new();
    let dragbar_node_ref = NodeRef::<Div>::new();

    let _ = leptos_dom::helpers::window_event_listener(ev::mousemove, move |ev| {
        if drag_mode.get() {
            if let Some(explorer_elem) = explorer_node_ref.get() {
                let rect = explorer_elem.get_bounding_client_rect();
                let new_width = ev.client_x() - rect.left() as i32;

                if new_width > min_explorer_width && new_width < screen_width / 2 {
                    set_width.set(new_width);
                    set_local_store_value("rc_explorer_width", new_width.to_string());
                }
            }
        }
    });

    let _ = leptos_dom::helpers::window_event_listener(ev::mouseup, move |_ev| {
        set_drag_mode.set(false);
    });

    view! {
        <div class="flex flex-row dark:text-white">
            <RestClientExplorer node_ref=explorer_node_ref current_request set_current_request width />

            <div node_ref=dragbar_node_ref class="w-1 bg-gray-700 hover:bg-blue-400/50 cursor-col-resize h-full transition-colors"
                on:mousedown=move |e| {
                    e.prevent_default();
                    set_drag_mode.set(true);
                }
            />

            <RequestPanel request_info=current_request set_request_info=set_current_request/>
        </div>
    }
}
