use crate::{
    components::layout::drag_splitter::DragSplitter,
    domain::rest_client::{
        model::{
            request_params::RequestParams, request_response::RequestResponse,
            rest_client_context::RestClientContext,
        },
        ui::request_params_url::RequestParamsUrl,
    },
    i18n::*,
};
use leptos::{html::Div, prelude::*};

use crate::domain::rest_client::ui::{
    request_params_panel::RequestParamsPanel, request_result_panel::RequestResultPanel,
};

#[component]
pub fn RequestPanel(node_ref: NodeRef<Div>) -> impl IntoView {
    let i18n = use_i18n();
    let rc_context = use_context::<RestClientContext>().expect("Failed get rc_context");

    let (params, _set_params) = signal(RequestParams::new(rc_context.clone()));
    let response = RequestResponse::new(params, rc_context.clone());

    let params_ref = NodeRef::<Div>::new();
    let result_ref = NodeRef::<Div>::new();

    create_request_watcher(params, rc_context.clone(), response.clone());
    create_params_watchers(params, rc_context.clone());

    view! {
        <div node_ref=node_ref class="flex-1 flex">
            <div class="flex-1 flex items-center justify-center"
                class:hidden=move || { rc_context.request.read().id > 0 }>
                {t!(i18n, rest_client_request_not_selected_msg)}
            </div>

            <div class="flex-1 overflow-y-auto flex flex-col gap-2 px-2 py-4 text-xs md:text-base"
                class:hidden=move || { rc_context.request.read().id == 0 }
                >
                <RequestParamsUrl params set_response=response.write_only() />

                <div class="flex-1 overflow-y-auto flex flex-col md:flex-row gap-2 text-xs md:text-base">
                    <RequestParamsPanel node_ref=params_ref params />

                    <DragSplitter
                        class_name="hidden md:block".to_owned()
                        target_ref=params_ref
                        second_target_ref=result_ref
                        local_store_prop_name=move || "params_width".to_owned()
                        min_ratio={10.0}
                        max_ratio={90.0}
                        default_ratio={50.0} />

                    <RequestResultPanel response=response.read_only() params node_ref=result_ref />

                </div>
            </div>
        </div>
    }
}

fn create_request_watcher(
    params: ReadSignal<RequestParams>,
    rc_context: RestClientContext,
    response: RequestResponse,
) {
    Effect::watch(
        move || rc_context.request.get(),
        move |value, prev, _| {
            if prev.is_none()
                || value.id != prev.unwrap().id
                || rc_context.project.read_untracked().parse::<i32>().unwrap_or(0)
                    != prev.unwrap().project_id
            {
                params.read_untracked().url.set(value.url.to_owned());
                params.read_untracked().method.set(value.method.to_owned());

                params.read_untracked().read_from_store(rc_context.clone(), value.id);

                if params.read_untracked().save_response.get_untracked() {
                    response.read_from_store(rc_context.clone(), value.id);
                } else {
                    response.clear();
                }
            }
        },
        false,
    );
}

fn create_params_watchers(params: ReadSignal<RequestParams>, rc_context: RestClientContext) {
    Effect::watch(
        move || params.read_untracked().url.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                rc_context.request.write().url = value.to_owned();
            }
        },
        false,
    );

    Effect::watch(
        move || params.read_untracked().method.get(),
        move |value, prev, _| {
            if prev.is_none() || value != prev.unwrap() {
                rc_context.request.write().method = value.to_owned();
            }
        },
        false,
    );
}
