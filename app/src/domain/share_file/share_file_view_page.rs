use gloo_net::http::Request;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_query_map;

use crate::components::{
    layout::message_banner::{Messages, show_error},
    ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth},
};

#[component]
pub fn ShareFileViewPage() -> impl IntoView {
    let messages = use_context::<Messages>().expect("Cant get messages context!");
    let params = use_query_map();
    let (file_name, set_file_name) = signal("".to_owned());
    let (is_image, set_is_image) = signal(false);
    let (in_progress, set_in_progress) = signal(true);

    Effect::new(move |_| {
        let id = params.read().get("id").unwrap();
        spawn_local(async move {
            set_in_progress.set(true);
            match Request::get("/share_file_info").query([("id", id)]).build() {
                Ok(request) => match request.send().await {
                    Ok(response) => match response.text().await {
                        Ok(response_text) => {
                            let parts: Vec<&str> = response_text.split('\n').collect();
                            set_file_name.set(parts[0].to_owned());
                            set_is_image.set(parts[2].parse::<bool>().unwrap());
                        }
                        Err(err) => show_error(err.to_string(), messages),
                    },
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            }
            set_in_progress.set(false);

        });
    });

    view! {
        <div class="flex flex-col items-center justify-center gap-4 py-12 text-xs md:text-base dark:text-white">
            { move || view! {
                <Show when=move || { !in_progress.get() } fallback=|| view! { <div>Загрузка...</div> }>
                    <Show when=move || { is_image.get() } fallback=|| view! {  }>
                        <div class="items-center justify-center">
                            <img src={format!("/share_file_download?id={}&thumbnail=true", params.read().get("id").unwrap())} alt={file_name.get()}/>
                        </div>
                    </Show>

                    <ButtonLink label=format!("Скачать {}", file_name.get()) href={format!("/share_file_download?id={}", params.read().get("id").unwrap())} button_width=ButtonLinkWidth::Auto
                        color=move || ButtonLinkColor::Primary prop:download=file_name.get() />
                </Show>
                }
            }

        </div>
    }
}
