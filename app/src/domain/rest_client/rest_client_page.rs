use leptos::{html::Div, prelude::*};

use crate::{
    components::layout::drag_splitter::DragSplitter,
    domain::rest_client::{
        model::request_params::RequestInfo,
        ui::{request_panel::RequestPanel, rest_client_explorer::RestClientExplorer},
    },
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let (current_request, set_current_request) = signal(RequestInfo::new_empty());
    let (project, set_project) = signal("".to_owned());

    let explorer_ref = NodeRef::<Div>::new();

    view! {
        <div class="flex flex-row dark:text-white h-screen md:h-[95dvh] text-xs md:text-base">
            <RestClientExplorer node_ref=explorer_ref current_request set_current_request project set_project/>

            <DragSplitter
                class_name="hidden md:block".to_owned()
                target_ref=explorer_ref
                local_store_prop_name=move || "rc_explorer_width".to_owned()
                min_scr_ration={1.0 / 10.0}
                max_scr_ration={1.0 / 2.0}
                default_scr_ration={1.0 / 6.0} />

            <RequestPanel project request_info=current_request set_request_info=set_current_request/>
        </div>
    }
}
