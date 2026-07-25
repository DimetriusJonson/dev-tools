use std::cmp::max;

use leptos::{html::Div, prelude::*};

use crate::{
    common::{local_store::get_local_store_value, ui_utils::get_browser_width},
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
    let (explorer_width, set_explorer_width) = signal(
        get_local_store_value("rc_explorer_width", min_explorer_width.to_string())
            .parse::<i32>()
            .unwrap(),
    );
    let explorer_ref = NodeRef::<Div>::new();

    view! {
        <div class="flex flex-row dark:text-white">
            <RestClientExplorer node_ref=explorer_ref current_request set_current_request width=explorer_width />

            <DragSplitter target_ref=explorer_ref set_width=set_explorer_width local_store_prop_name="rc_explorer_width" min_width={min_explorer_width} max_width={screen_width / 2}/>

            <RequestPanel request_info=current_request set_request_info=set_current_request/>
        </div>
    }
}
