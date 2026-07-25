use std::cmp::max;

use leptos::{html::Div, prelude::*};

use crate::{
    common::ui_utils::get_browser_width,
    components::layout::drag_splitter::DragSplitter,
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
    let explorer_ref = NodeRef::<Div>::new();

    view! {
        <div class="flex flex-row dark:text-white">
            <RestClientExplorer node_ref=explorer_ref current_request set_current_request />

            <DragSplitter target_ref=explorer_ref local_store_prop_name="rc_explorer_width" 
                min_width={min_explorer_width} max_width={screen_width / 2}
                default_width={min_explorer_width} />

            <RequestPanel request_info=current_request set_request_info=set_current_request/>
        </div>
    }
}
