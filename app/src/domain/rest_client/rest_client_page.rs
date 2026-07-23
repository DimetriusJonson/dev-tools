use leptos::prelude::*;

use crate::domain::rest_client::ui::{
    request_panel::RequestPanel, rest_client_explorer::RestClientExplorer,
};

#[component]
pub fn RestClientPage() -> impl IntoView {
    let (current_request, set_current_request) = signal(None);

    view! {
        <div class="flex flex-row dark:text-white">
            <RestClientExplorer current_request set_current_request />

            <Show when=move || { current_request.read().is_some() }
                fallback=|| view! { <div class="flex-1 flex h-[94dvh] items-center justify-center">{"Select project please."}</div> }
            >
                <RequestPanel request_info={current_request.get().unwrap()}/>
            </Show>

        </div>
    }
}
