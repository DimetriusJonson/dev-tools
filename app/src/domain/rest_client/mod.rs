#[cfg(feature = "standalone")]
pub use self::rest_client_page::RestClientPage as RestClientPage;
#[cfg(feature = "standalone")]
pub mod rest_client_page;
#[cfg(feature = "standalone")]
pub mod rest_client_response_panel;


#[cfg(not(feature = "standalone"))]
#[leptos::component]
pub fn RestClientPage() -> impl leptos::IntoView {
    use leptos::view;

    view! {""}
}