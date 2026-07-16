use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth};
use crate::i18n::*;

#[component]
pub fn ShareFileViewPage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_query_map();

    let id = move || params.read().get("id").unwrap_or_default();
    let local = move || {
        params.read().get("local").unwrap_or_default().parse::<bool>().unwrap_or_default()
    };

    let share_info_resource = Resource::new(
        move || (id(), local()),
        async move |(id, local)| get_share_info(id, local).await,
    );

    view! {
        <div class="flex flex-col items-center justify-center gap-4 py-12 text-xs md:text-base dark:text-white">

            <Transition
                fallback=move || view! { <div>{t!(i18n, loading_progress)}</div> }
                >
                {move || share_info_resource.get().map(|data| {
                    data.map(|info| {
                        let file_name = info.0.to_owned();
                        let download_file_name = info.0.to_owned();
                        let download_srv_name = if local() {"share_local_file_download"} else {"share_file_download"};

                        view! {
                            <Show when=move || { info.1 }>
                                {
                                    view! {
                                        <div class="items-center justify-center">
                                            <img src={format!("/{}?id={}&thumbnail=true", download_srv_name, id())} alt={info.0.to_owned()}/>
                                        </div>
                                    }.into_view()
                                }
                            </Show>

                            <ButtonLink label=move || t_display!(i18n, share_file_view_download_file, file_name = file_name.to_owned()).to_string() href={format!("/{}?id={}", download_srv_name, id())} button_width=ButtonLinkWidth::Auto
                                color=move || ButtonLinkColor::Primary prop:download=download_file_name />
                        }
                    })
                })}
            </Transition>
        </div>
    }
}

#[server]
pub async fn get_share_info(id: String, local: bool) -> Result<(String, bool), ServerFnError> {
    use crate::common::app_state::ssr::AppState;

    let app_state =
        use_context::<AppState>().ok_or_else(|| ServerFnError::new("App state missing."))?;

    let site_addr = app_state.leptos_options.site_addr;

    let srv_name = if local {"share_local_file_info"} else {"share_file_info"};

    let response =
        reqwest::get(&format!("http://127.0.0.1:{}/{}?id={}", site_addr.port(), srv_name, id))
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
