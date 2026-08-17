use leptos::{html::Div, prelude::*};

use crate::{
    components::layout::drag_splitter::DragSplitter,
    domain::rest_client::{
        model::{request_info::RequestInfo, rest_client_context::RestClientContext},
        ui::{request_panel::RequestPanel, rest_client_explorer::RestClientExplorer},
    },
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let request = RwSignal::new(RequestInfo::new_empty());
    let project = RwSignal::new("".to_owned());

    let rest_client_context = RestClientContext { project, request };
    provide_context(rest_client_context);

    let explorer_ref = NodeRef::<Div>::new();
    let right_panel_ref = NodeRef::<Div>::new();

    view! {
        <div class="flex flex-row dark:text-white max-w-screen h-screen md:h-[95dvh] text-xs md:text-base">
            <RestClientExplorer node_ref=explorer_ref />

            <DragSplitter
                class_name="hidden md:block".to_owned()
                target_ref=explorer_ref
                second_target_ref=right_panel_ref
                local_store_prop_name=move || "rc_explorer_width".to_owned()
                min_ratio={10.0}
                max_ratio={83.0}
                default_ratio={16.0} 
                allow_hidden=true
                />

            <RequestPanel node_ref=right_panel_ref />
        </div>
    }
}
