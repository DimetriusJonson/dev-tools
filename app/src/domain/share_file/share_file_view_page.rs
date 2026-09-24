use gloo_net::http::Request;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use model::share_file::share_file_info_dto::ShareFileInfoDto;

use crate::components::layout::message_banner::{Messages, show_error};
use crate::components::ui::button_link::{ButtonLink, ButtonLinkColor, ButtonLinkWidth};
use crate::components::ui::code_mirror_editor::CodeMirrorEditor;
use crate::i18n::*;

#[component]
pub fn ShareFileViewPage() -> impl IntoView {
    let i18n = use_i18n();
    let params = use_query_map();
    let messages = use_context::<Messages>().expect("Cant get messages context!");

    let id = move || params.read().get("id").unwrap_or_default();
    let local =
        move || params.read().get("local").unwrap_or_default().parse::<bool>().unwrap_or_default();

    let share_info_resource = LocalResource::new(move || async move {
        match Request::get("/share_file_info_ex")
            .query([
                ("id", params.read().get("id").unwrap_or_default()),
                ("local", params.read().get("local").unwrap_or("false".to_owned())),
            ])
            .build()
        {
            Ok(request) => match request.send().await {
                Ok(response) => match response.json::<ShareFileInfoDto>().await {
                    Ok(dto) => Some(dto),
                    Err(err) => {
                        console_log(&format!("Error: {}", err));
                        None
                    }
                },
                Err(err) => {
                    console_log(&format!("Error: {}", err));
                    None
                }
            },
            Err(err) => {
                console_log(&format!("Error: {}", err));
                None
            }
        }
    });

    let (text, set_text) = signal("".to_owned());
    let (code_lang, set_code_lang) = signal("text".to_owned());

    view! {
        <div class="flex flex-col items-center justify-center gap-4 py-12 text-xs md:text-base dark:text-white">
            {move || share_info_resource.get().map(|info| {
                info.map(|info|{
                    let file_name = info.file_name.to_owned();
                    let download_file_name = info.file_name.to_owned();
                    let download_srv_name = if local() {"share_local_file_download"} else {"share_file_download"};
                    let dowload_url = format!("/{}?id={}", download_srv_name, id());

                    if info.file_size < 100 * 1024 && let Some(lang) = get_mime_code_lang(&info.mime_type) {
                        set_code_lang.set(lang.to_owned());
                        load_text(dowload_url.to_owned(), set_text, messages);
                    } else {
                        set_code_lang.set("".to_owned());
                    }

                    view! {
                        <Show when=move || { info.is_image }>
                            {
                                view! {
                                    <div class="items-center justify-center">
                                        <img src={format!("/{}?id={}&thumbnail=true", download_srv_name, id())} alt={info.file_name.to_owned()}/>
                                    </div>
                                }.into_view()
                            }
                        </Show>

                        <Show when=move || { !code_lang.read_untracked().is_empty() }>
                            {
                                view! {
                                    <div class="min-h-0 overflow-y-auto flex flex-col gap-4 w-full h-[35dvh] md:h-[78dvh] px-4">
                                        <CodeMirrorEditor
                                            element_id="text-code-editor".to_owned()
                                            lang=code_lang
                                            value=text
                                            set_value=set_text
                                            read_only=true
                                        />
                                    </div>
                                }.into_view()
                            }
                        </Show>

                        <ButtonLink label=move || t_display!(i18n, share_file_view_download_file, file_name = file_name.to_owned()).to_string() href={dowload_url.to_owned()} button_width=ButtonLinkWidth::Auto
                            color=move || ButtonLinkColor::Primary prop:download=download_file_name />
                    }
                })
            })}
        </div>
    }
}

fn load_text(url: String, set_text: WriteSignal<String>, messages: Messages) {
    spawn_local(async move {
        set_text.set("".to_owned());
        match Request::get(&url).build() {
            Ok(request) => match request.send().await {
                Ok(response) => match response.text().await {
                    Ok(data) => {
                        set_text.set(data);
                    }
                    Err(err) => show_error(err.to_string(), messages),
                },
                Err(err) => show_error(err.to_string(), messages),
            },
            Err(err) => show_error(err.to_string(), messages),
        }
    })
}

fn get_mime_code_lang(mime_type: &str) -> Option<&str> {
    if mime_type == "text/plain" {
        return Some("text");
    }

    if mime_type.contains("json") {
        return Some("json");
    }

    if mime_type.contains("xml") {
        return Some("xml");
    }

    if mime_type.contains("html") {
        return Some("html");
    }

    None
}