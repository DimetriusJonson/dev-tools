use crate::components::layout::drag_splitter::DragSplitter;
use crate::components::layout::tabs::Tabs;
use crate::components::ui::text_area::TextArea;
use crate::domain::rest_client::ui::request_body_form_panel::RequestBodyFormPanel;
use crate::domain::rest_client::ui::request_headers_panel::RequestHeadersPanel;
use crate::domain::rest_client::ui::request_params::{RequestInfo, RequestParams};
use crate::domain::rest_client::ui::request_query_panel::RequestQueryPanel;
use crate::domain::rest_client::ui::request_store::{
    RequestFieldKind, build_request_stored_key, get_stored_value, set_stored_value,
};
use crate::i18n::*;
use leptos::html::Div;
use leptos::prelude::*;

#[component]
pub fn RequestParamsPanel(
    project: ReadSignal<String>,
    request_info: ReadSignal<RequestInfo>,
    body_tab_selected: ReadSignal<usize>,
    set_body_tab_selected: WriteSignal<usize>,
    params: ReadSignal<RequestParams>,
    node_ref: NodeRef<Div>,
) -> impl IntoView {
    let i18n = use_i18n();
    let tab_body_text_ref = NodeRef::<Div>::new();
    let tab_body_form_encoded_ref = NodeRef::<Div>::new();
    let params_ref = NodeRef::<Div>::new();

    let (params_tab_selected, set_params_tab_selected) = signal(0);
    let tab_headers_ref = NodeRef::<Div>::new();
    let tab_query_ref = NodeRef::<Div>::new();

    Effect::watch(
        move || params_tab_selected.get(),
        move |value, _prev, _| {
            set_stored_value(
                project,
                request_info.read_untracked().id,
                RequestFieldKind::ParamsTab,
                value.to_string(),
            )
        },
        false,
    );

    Effect::watch(
        move || request_info.get(),
        move |value, prev, _| {
            let id = value.id;
            let project_id = project.get_untracked();
            if prev.is_none() || id != prev.unwrap().id || project_id.parse::<i32>().unwrap() != prev.unwrap().project_id {
                let tab_index =
                    get_stored_value(RequestFieldKind::ParamsTab, "0".to_owned(), &project_id, id);
                set_params_tab_selected.set(tab_index.parse().unwrap());
            }
        },
        false,
    );

    Effect::watch(
        move || body_tab_selected.get(),
        move |value, _prev, _| match value {
            1 => params.read_untracked().set_body_type.set("formencoded".to_owned()),
            _ => params.read_untracked().set_body_type.set("text".to_owned()),
        },
        false,
    );

    view! {

        <div node_ref=node_ref class="min-h-0 overflow-y-auto flex flex-col gap-2 md:gap-4">
            <div class="flex-1 flex flex-col">
                <div node_ref=params_ref class="flex flex-col">
                    <Tabs class_name="".to_owned()
                        tab_selected=params_tab_selected set_tab_selected=set_params_tab_selected
                        items=move || vec![
                            ("Headers", tab_headers_ref),
                            ("Query", tab_query_ref),
                        ] />

                    <div node_ref=tab_headers_ref class="flex-1 flex flex-col overflow-y-auto gap-y-2 pt-4">
                        <RequestHeadersPanel params />
                    </div>

                    <div node_ref=tab_query_ref class="flex-1 flex flex-col overflow-y-auto pt-4 gap-4">
                        <RequestQueryPanel params request_info />
                    </div>
                </div>

                <DragSplitter
                    class_name="hidden md:block".to_owned()
                    target_ref=params_ref
                    horizontal=true
                    local_store_prop_name=move || build_request_stored_key(project.read_untracked().as_str(), request_info.read_untracked().id, "headers_height")
                    min_scr_ration={1.0 / 10.0}
                    max_scr_ration={2.0 / 3.0}
                    default_scr_ration={1.0 / 6.0}
                    allow_mobile=true
                />

                <div class="flex-1 flex flex-col">
                    <Tabs class_name="".to_owned()
                        tab_selected=body_tab_selected set_tab_selected=set_body_tab_selected
                        items=move || vec![
                            ("Text", tab_body_text_ref),
                            ("Form Encoded", tab_body_form_encoded_ref),
                        ] />

                    <div node_ref=tab_body_text_ref class="flex-1 flex overflow-y-auto">
                        <TextArea
                            name="body".to_owned()
                            class_name="w-full resize-none".to_owned()
                            placeholder=move || {t!(i18n, rest_client_body_placeholder).to_html()}
                            value=params.read_untracked().body
                            set_value=params.read_untracked().set_body
                            on_change=move |_| {}
                        />
                    </div>

                    <div node_ref=tab_body_form_encoded_ref class="flex-1 flex flex-col overflow-y-auto pt-4 gap-4">
                        <RequestBodyFormPanel params/>
                    </div>
                </div>
            </div>
        </div>

    }
}
