use leptos::prelude::*;

use crate::domain::rest_client::ui::{
    request_panel::RequestPanel, request_params::RequestInfo, rest_client_explorer::RestClientExplorer,
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let (current_request, set_current_request) = signal(RequestInfo{ id: 0, url: "".to_owned(), method: "".to_owned() });

    view! {
        <div class="flex flex-row dark:text-white">
            <RestClientExplorer current_request set_current_request />

           <Show when=move || { current_request.read().id > 0 }
                fallback=|| view! { <div class="flex-1 flex h-[94dvh] items-center justify-center">{"Select project please."}</div> }
            >
                <RequestPanel request_info=current_request set_request_info=set_current_request/>
            </Show>

        </div>
    }
}
