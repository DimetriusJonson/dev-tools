use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth}};

#[component]
pub fn ShareFileViewPage() -> impl IntoView {
    let params = use_query_map();

    let id = move || params.read().get("id").unwrap_or_default();

    let share_info_resource =
        Resource::new(id, async move |id| get_share_info(id.parse().unwrap_or_default()).await);

    view! {
        <div class="flex flex-col items-center justify-center gap-4 py-12 text-xs md:text-base dark:text-white">

            <Transition
                fallback=move || view! { <div>Загрузка...</div> }
                >
                {move || share_info_resource.get().map(|data| {
                    data.map(|info| {
                        let download_label = format!("Скачать {}", info.0.to_owned());
                        let download_file_name = info.0.to_owned();
                        view! {
                            <Show when=move || { info.1 }>
                                {
                                    view! {
                                        <div class="items-center justify-center">
                                            <img src={format!("/share_file_download?id={}&thumbnail=true", id())} alt={info.0.to_owned()}/>
                                        </div>
                                    }.into_view()
                                }
                            </Show>

                            <ButtonLink label=download_label href={format!("/share_file_download?id={}", id())} button_width=ButtonLinkWidth::Auto
                                color=move || ButtonLinkColor::Primary prop:download=download_file_name />
                        }
                    })
                })}
            </Transition>
        </div>
    }
}

#[server]
pub async fn get_share_info(id: String) -> Result<(String, bool), ServerFnError> {
    use crate::common::app_state::ssr::AppState;

    let app_state = use_context::<AppState>()
            .ok_or_else(|| ServerFnError::new("App state missing."))?;

    let site_addr = app_state.leptos_options.site_addr;

    let response = reqwest::get(&format!("http://127.0.0.1:{}/share_file_info?id={}", site_addr.port(), id))
        .await
        .map_err(|e| ServerFnError::new(format!("Request failed: {e}")))?;

    if response.status() == 200 {
        let response_text = response
            .text()
            .await
            .map_err(|e| ServerFnError::new(format!("Request failed: {e}")))?;

        let parts: Vec<&str> = response_text.split('\n').collect();
        let file_name = parts[0].to_owned();
        let is_image = parts[2].parse::<bool>().unwrap();

        Ok((file_name, is_image))
    } else {
        let response_text = response
            .text()
            .await
            .map_err(|e| ServerFnError::new(format!("Request failed: {e}")))?;

        Err(ServerFnError::ServerError(response_text))
    }

}
