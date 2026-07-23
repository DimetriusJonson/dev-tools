use leptos::prelude::*;

use crate::domain::rest_client::ui::request_panel::RequestPanel;

#[component]
pub fn RestClientPage() -> impl IntoView {
    view! {
        <RequestPanel id=1/>
    }
}
