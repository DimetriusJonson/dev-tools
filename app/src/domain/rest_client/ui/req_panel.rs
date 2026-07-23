use leptos::prelude::*;

use crate::domain::rest_client::ui::{
    req_params::RequestParams, req_params_panel::ReqParamsPanel, req_result_panel::ReqResultPanel,
};

#[component]
pub fn ReqPanel(
    params: ReadSignal<RequestParams>,
) -> impl IntoView {
    let (response, set_response) = signal(None);

    view! {
        <div class="flex-1 flex flex-col md:flex-row gap-4 px-2 py-4 text-xs md:text-base">
            <ReqParamsPanel
                params on_result=move|res| {
                    set_response.set(Some(res));
                }
            />

            <ReqResultPanel data=response/>

        </div>
    }
}
