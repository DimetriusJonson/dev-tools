use crate::common::json_processor::format_json;
use crate::common::xml_processor::format_xml;
use crate::components::layout::drag_splitter::DragSplitter;
use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::layout::tabs::{TabItem, Tabs};
use crate::components::ui::button::{Button, ButtonColor, ButtonHeight, ButtonWidth};
use crate::components::ui::code_mirror_editor::CodeMirrorEditor;
use crate::domain::rest_client::model::request_params::{
    RequestBodyKind, RequestInfo, RequestParams,
};
use crate::domain::rest_client::ui::request_body_form_panel::RequestBodyFormPanel;
use crate::domain::rest_client::ui::request_headers_panel::RequestHeadersPanel;
use crate::domain::rest_client::ui::request_query_panel::RequestQueryPanel;
use crate::domain::rest_client::util::request_store::{
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
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let tab_body_text_ref = NodeRef::<Div>::new();
    let tab_body_form_encoded_ref = NodeRef::<Div>::new();
    let params_ref = NodeRef::<Div>::new();

    let (params_tab_selected, set_params_tab_selected) = signal(0);
    let (body_lang, set_body_lang) = signal("text".to_owned());
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
            if prev.is_none()
                || id != prev.unwrap().id
                || project_id.parse::<i32>().unwrap_or(0) != prev.unwrap().project_id
            {
                let tab_index =
                    get_stored_value(RequestFieldKind::ParamsTab, "0".to_owned(), &project_id, id);
                set_params_tab_selected.set(tab_index.parse().unwrap_or(0));
            }
        },
        false,
    );

    Effect::watch(
        move || body_tab_selected.get(),
        move |value, _prev, _| {
            let body_type = match value {
                1 => RequestBodyKind::Xml,
                2 => RequestBodyKind::Text,
                3 => RequestBodyKind::Formencoded,
                _ => RequestBodyKind::Json,
            };

            match body_type {
                RequestBodyKind::Text => set_body_lang.set("text".to_owned()),
                RequestBodyKind::Json => set_body_lang.set("json".to_owned()),
                RequestBodyKind::Xml => set_body_lang.set("xml".to_owned()),
                _ => (),
            };

            params.read_untracked().set_body_type.set(body_type);
        },
        false,
    );

    let on_format_body = move |_| match params.read_untracked().body_type.get_untracked() {
        RequestBodyKind::Json => {
            let formatted =
                format_json(params.read_untracked().body.read_untracked().as_borrowed(), 4);
            params.read_untracked().set_body.set(formatted);
        }
        RequestBodyKind::Xml => {
            let formatted =
                match format_xml(params.read_untracked().body.read_untracked().as_borrowed(), 4) {
                    Ok(formatted_xml) => formatted_xml,
                    Err(err) => {
                        show_error(err.to_string(), messages);
                        return;
                    }
                };
            params.read_untracked().set_body.set(formatted);
        }
        RequestBodyKind::Text => (),
        RequestBodyKind::Formencoded => (),
    };

    view! {

        <div node_ref=node_ref class="min-h-0 overflow-y-auto flex flex-col gap-2 md:gap-4">
            <div class="flex-1 flex flex-col">
                <div node_ref=params_ref class="flex flex-col">
                    <Tabs class_name="".to_owned()
                        tab_selected=params_tab_selected set_tab_selected=set_params_tab_selected
                        items=move || vec![
                            TabItem::new_simple(t_string!(i18n, rest_client_headers_tab), tab_headers_ref),
                            TabItem::new_simple(t_string!(i18n, rest_client_query_tab), tab_query_ref),
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
                    local_store_prop_name=move || build_request_stored_key(project.read().as_str(), request_info.read().id, "headers_height")
                    min_scr_ration={1.0 / 10.0}
                    max_scr_ration={2.0 / 3.0}
                    default_scr_ration={1.0 / 6.0}
                    allow_mobile=true
                />

                <div class="flex-1 flex flex-col pt-4">
                    <Tabs class_name="".to_owned()
                        tab_selected=body_tab_selected set_tab_selected=set_body_tab_selected
                        items=move || vec![
                            TabItem::new("JSON", RequestBodyKind::Json.content_type(), tab_body_text_ref),
                            TabItem::new("XML", RequestBodyKind::Xml.content_type(), tab_body_text_ref),
                            TabItem::new("Text", RequestBodyKind::Text.content_type(), tab_body_text_ref),
                            TabItem::new("Form-encode", RequestBodyKind::Formencoded.content_type(), tab_body_form_encoded_ref),
                        ] />

                    <div id="code_editor" node_ref=tab_body_text_ref class="group relative flex-1 flex overflow-y-auto pt-4">
                        <CodeMirrorEditor
                            element_id="request-body-code-editor".to_owned()
                            lang=body_lang
                            value=params.read_untracked().body
                            set_value=params.read_untracked().set_body
                         />

                        <Show when=move || params.read_untracked().body_type.get() != RequestBodyKind::Text>
                            <Button
                                label=move || "¶".to_owned()
                                title=move || t_string!(i18n, rest_client_req_body_format_title).to_owned()
                                class_name="absolute right-1 top-1 hidden group-hover:block text-bold w-8 px-2 text-gray-500 hover:text-green-500 z-1000".to_owned()
                                button_width=ButtonWidth::Custom
                                button_height=ButtonHeight::Custom
                                color=ButtonColor::Custom
                                loading=move || false
                                disabled=move || false
                                on_click=on_format_body
                            />
                        </Show>

                    </div>

                    <div node_ref=tab_body_form_encoded_ref class="flex-1 flex flex-col overflow-y-auto pt-4 gap-4">
                        <RequestBodyFormPanel params/>
                    </div>
                </div>
            </div>
        </div>

    }
}
