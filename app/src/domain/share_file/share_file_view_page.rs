use gloo_net::http::Request;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth};
use crate::i18n::*;

#[component]
pub fn ShareFileViewPage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_query_map();

    let id = move || params.read().get("id").unwrap_or_default();
    let local =
        move || params.read().get("local").unwrap_or_default().parse::<bool>().unwrap_or_default();

    let share_info_resource = LocalResource::new(move || async move {
        match Request::get("/share_file_info_ex")
            .query([("id", params.read().get("id").unwrap_or_default()), ("local", params.read().get("local").unwrap_or("false".to_owned()))])
            .build()
        {
            Ok(request) => match request.send().await {
                Ok(response) => match response.text().await {
                    Ok(response_text) => {
                        let parts = response_text.split(';').collect::<Vec<&str>>();
                        (parts[0].to_owned(), parts[1].parse::<bool>().unwrap())
                    }
                    Err(err) => {
                        console_log(&format!("Error: {}", err));
                        ("".to_owned(), false)
                    }
                },
                Err(err) => {
                    console_log(&format!("Error: {}", err));
                    ("".to_owned(), false)
                }
            },
            Err(err) => {
                console_log(&format!("Error: {}", err));
                ("".to_owned(), false)
            }
        }
    });

    view! {
        <div class="flex flex-col items-center justify-center gap-4 py-12 text-xs md:text-base dark:text-white">
            {move || share_info_resource.get().map(|info| {
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
            })}
        </div>
    }
}
