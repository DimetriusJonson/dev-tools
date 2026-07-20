#[cfg(feature = "rest_client")]
pub use self::rest_client_page::RestClientPage as RestClientPage;
#[cfg(feature = "rest_client")]
pub mod rest_client_page;
#[cfg(feature = "rest_client")]
pub mod rest_client_response_panel;


#[cfg(not(feature = "rest_client"))]
#[leptos::component]
pub fn RestClientPage() -> impl leptos::IntoView {
    use leptos::view;

    view! {""}
}