
use leptos::{html::Div, prelude::*};

use crate::{
    components::layout::drag_splitter::DragSplitter,
    domain::rest_client::ui::{
        request_panel::RequestPanel, request_params::RequestInfo,
        rest_client_explorer::RestClientExplorer,
    },
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let (current_request, set_current_request) = signal(RequestInfo::new_empty());
    let explorer_ref = NodeRef::<Div>::new();

    view! {
        <div class="flex flex-row dark:text-white">
            <RestClientExplorer node_ref=explorer_ref current_request set_current_request />

            <DragSplitter target_ref=explorer_ref local_store_prop_name="rc_explorer_width" 
                min_scr_ration={1.0 / 10.0} 
                max_scr_ration={1.0 / 2.0}
                default_scr_ration={1.0 / 6.0} />

            <RequestPanel request_info=current_request set_request_info=set_current_request/>
        </div>
    }
}
